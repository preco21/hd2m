use crate::{
    convert_image_to_mat_grayscale, convert_tm_mat_to_array2, match_template_with_mask,
    resize_template_scale, DirectionDescriptor,
};
use anyhow::Result;
use cv::core::MatTraitConst;
use opencv as cv;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Hd2mCvManagerConfig {
    pub template_up_image: image::RgbaImage,
    pub template_down_image: image::RgbaImage,
    pub template_right_image: image::RgbaImage,
    pub template_left_image: image::RgbaImage,
    pub base_screen_size: (usize, usize),
    pub search_options: Option<Hd2mCvSearchOptions>,
}

#[derive(Debug, Default)]
pub struct Hd2mCvSearchOptions {
    pub threshold: Option<f32>,
    pub search_chunk_size: Option<usize>,
    pub discarding_distance_threshold: Option<f64>,
}

#[derive(Debug)]
pub struct Hd2mCvManager {
    template_original: DirectionImageTemplate,
    base_screen_size: (usize, usize),
    template_resized_registry: BTreeMap<(usize, usize), DirectionImageTemplate>,
    template_search_threshold: f32,
    template_search_chunk_size: usize,
    template_discarding_distance_threshold: f64,
}

impl Hd2mCvManager {
    pub fn new(config: Hd2mCvManagerConfig) -> Result<Self> {
        let rep_template = convert_image_to_mat_grayscale(&config.template_up_image)?;
        let init_template_size = rep_template.size()?;
        let search_options = config.search_options.unwrap_or_default();
        Ok(Self {
            template_original: DirectionImageTemplate::from_template(
                rep_template,
                convert_image_to_mat_grayscale(&config.template_down_image)?,
                convert_image_to_mat_grayscale(&config.template_right_image)?,
                convert_image_to_mat_grayscale(&config.template_left_image)?,
            ),
            base_screen_size: config.base_screen_size,
            template_resized_registry: BTreeMap::new(),
            template_search_threshold: search_options.threshold.unwrap_or(0.987),
            template_search_chunk_size: search_options
                .search_chunk_size
                .unwrap_or(init_template_size.width as usize),
            template_discarding_distance_threshold: search_options
                .discarding_distance_threshold
                .unwrap_or(init_template_size.width as f64),
        })
    }

    pub fn run_match(&self, target: &cv::core::Mat) -> Result<Vec<Vec<DirectionDescriptor>>> {
        let res_up = match_template_with_mask(&target, &self.template_original.up, None)?;
        let res_down = match_template_with_mask(&target, &self.template_original.down, None)?;
        let res_right = match_template_with_mask(&target, &self.template_original.right, None)?;
        let res_left = match_template_with_mask(&target, &self.template_original.left, None)?;

        let arr_up = convert_tm_mat_to_array2(&res_up)?;
        let arr_down = convert_tm_mat_to_array2(&res_down)?;
        let arr_right = convert_tm_mat_to_array2(&res_right)?;
        let arr_left = convert_tm_mat_to_array2(&res_left)?;

        let descriptors = crate::find_direction_commands(
            &arr_up.view(),
            &arr_down.view(),
            &arr_right.view(),
            &arr_left.view(),
            Some(self.template_search_threshold),
            Some(self.template_search_chunk_size),
            Some(self.template_discarding_distance_threshold),
        )?;

        Ok(descriptors)
    }

    pub fn ensure_screen_size(&mut self, width: usize, height: usize) -> Result<()> {
        if self
            .template_resized_registry
            .contains_key(&(width, height))
        {
            return Ok(());
        }

        let inferred_scale = width as f64 / self.base_screen_size.0 as f64;
        let template_resized = DirectionImageTemplate::with_resized(
            &self.template_original.up,
            &self.template_original.down,
            &self.template_original.right,
            &self.template_original.left,
            inferred_scale,
        )?;
        self.template_resized_registry
            .insert((width, height), template_resized);
        Ok(())
    }
}

#[derive(Debug)]
pub struct DirectionImageTemplate {
    pub up: cv::core::Mat,
    pub down: cv::core::Mat,
    pub right: cv::core::Mat,
    pub left: cv::core::Mat,
}

impl DirectionImageTemplate {
    pub fn from_template(
        template_up: cv::core::Mat,
        template_down: cv::core::Mat,
        template_right: cv::core::Mat,
        template_left: cv::core::Mat,
    ) -> Self {
        Self {
            up: template_up,
            down: template_down,
            right: template_right,
            left: template_left,
        }
    }

    pub fn with_resized(
        template_up: &cv::core::Mat,
        template_down: &cv::core::Mat,
        template_right: &cv::core::Mat,
        template_left: &cv::core::Mat,
        scale: f64,
    ) -> Result<Self> {
        Ok(Self {
            up: resize_template_scale(template_up, scale)?,
            down: resize_template_scale(template_down, scale)?,
            right: resize_template_scale(template_right, scale)?,
            left: resize_template_scale(template_left, scale)?,
        })
    }
}
