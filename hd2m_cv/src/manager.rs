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
    current_screen_size: Option<(usize, usize)>,
    template_registry: BTreeMap<(usize, usize), DirectionImageTemplate>,
    template_search_threshold: f32,
    template_search_chunk_size: usize,
    template_discarding_distance_threshold: f64,
}

impl Hd2mCvManager {
    pub fn new(config: Hd2mCvManagerConfig) -> Result<Self> {
        let rep_template = convert_image_to_mat_grayscale(&config.template_up_image)?;
        let init_template_size = rep_template.size()?;
        let search_options = config.search_options.unwrap_or_default();
        let template_original = DirectionImageTemplate::from_template(
            rep_template,
            convert_image_to_mat_grayscale(&config.template_down_image)?,
            convert_image_to_mat_grayscale(&config.template_right_image)?,
            convert_image_to_mat_grayscale(&config.template_left_image)?,
        );
        let mut template_registry: BTreeMap<(usize, usize), DirectionImageTemplate> =
            BTreeMap::new();
        template_registry.insert(
            (
                init_template_size.width as usize,
                init_template_size.height as usize,
            ),
            template_original.clone(),
        );
        Ok(Self {
            template_original,
            base_screen_size: config.base_screen_size,
            template_registry,
            template_search_threshold: search_options.threshold.unwrap_or(0.987),
            template_search_chunk_size: search_options
                .search_chunk_size
                .unwrap_or(init_template_size.height as usize + 10),
            current_screen_size: None,
            template_discarding_distance_threshold: search_options
                .discarding_distance_threshold
                .unwrap_or(init_template_size.width as f64 + 3.0),
        })
    }

    pub fn run_match(
        &mut self,
        target: &image::RgbaImage,
    ) -> Result<Vec<Vec<DirectionDescriptor>>> {
        let screen_size = self
            .current_screen_size
            .ok_or(anyhow::anyhow!("Target screen size not registered"))?;

        let target = convert_image_to_mat_grayscale(target)?;
        let matching_template = self
            .template_registry
            .get(&screen_size)
            .ok_or(anyhow::anyhow!(
                "Resized template not found for target size"
            ))?;

        let res_up = match_template_with_mask(&target, &matching_template.up, None)?;
        let res_down = match_template_with_mask(&target, &matching_template.down, None)?;
        let res_right = match_template_with_mask(&target, &matching_template.right, None)?;
        let res_left = match_template_with_mask(&target, &matching_template.left, None)?;

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

    pub fn use_screen_size(&mut self, width: usize, height: usize) -> Result<()> {
        if self.template_registry.contains_key(&(width, height)) {
            return Ok(());
        }

        // Since Helldivers 2 will not scale the ui along the height, we will use the width as the base scale.
        let inferred_scale = width as f64 / self.base_screen_size.0 as f64;
        let template_resized = DirectionImageTemplate::with_resized(
            &self.template_original.up,
            &self.template_original.down,
            &self.template_original.right,
            &self.template_original.left,
            inferred_scale,
        )?;

        let rep_template_size = template_resized.up.size()?;
        self.template_registry
            .insert((width, height), template_resized);
        self.current_screen_size = Some((width, height));
        self.set_search_options(Hd2mCvSearchOptions {
            search_chunk_size: Some(rep_template_size.height as usize * 2),
            discarding_distance_threshold: Some(rep_template_size.width as f64 + 3.0),
            ..Default::default()
        });

        Ok(())
    }

    pub fn set_search_options(&mut self, options: Hd2mCvSearchOptions) {
        self.template_search_threshold =
            options.threshold.unwrap_or(self.template_search_threshold);
        self.template_search_chunk_size = options
            .search_chunk_size
            .unwrap_or(self.template_search_chunk_size);
        self.template_discarding_distance_threshold = options
            .discarding_distance_threshold
            .unwrap_or(self.template_discarding_distance_threshold);
    }
}

#[derive(Debug, Clone)]
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
