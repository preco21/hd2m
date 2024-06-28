use crate::{
    convert_mat_to_array2, find_direction_commands, DirectionDescriptor, TemplateMatcher, TryIntoCv,
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
    template_original: DirectionTemplateMatcherSet,
    base_screen_size: (usize, usize),
    current_screen_size: Option<(usize, usize)>,
    template_registry: BTreeMap<(usize, usize), DirectionTemplateMatcherSet>,
    template_search_threshold: f32,
    template_search_chunk_size: usize,
    template_discarding_distance_threshold: f64,
}

impl Hd2mCvManager {
    pub fn new(config: Hd2mCvManagerConfig) -> Result<Self> {
        let up_template = TemplateMatcher::new(&config.template_up_image.try_into_cv()?)?;
        let init_template_size = up_template.mat().size()?;
        let matcher_set = DirectionTemplateMatcherSet::new(
            up_template,
            TemplateMatcher::new(&config.template_down_image.try_into_cv()?)?,
            TemplateMatcher::new(&config.template_right_image.try_into_cv()?)?,
            TemplateMatcher::new(&config.template_left_image.try_into_cv()?)?,
        )?;
        let mut template_registry: BTreeMap<(usize, usize), DirectionTemplateMatcherSet> =
            BTreeMap::new();
        template_registry.insert(
            (
                init_template_size.width as usize,
                init_template_size.height as usize,
            ),
            matcher_set.clone(),
        );
        let search_options = config.search_options.unwrap_or_default();
        Ok(Self {
            template_original: matcher_set,
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

    pub fn run_match_rgba(
        &mut self,
        target: &image::RgbaImage,
    ) -> Result<Vec<Vec<DirectionDescriptor>>> {
        self.run_match_mat(&target.try_into_cv()?)
    }

    pub fn run_match_mat(
        &mut self,
        target: &cv::core::Mat,
    ) -> Result<Vec<Vec<DirectionDescriptor>>> {
        let screen_size = self
            .current_screen_size
            .ok_or(anyhow::anyhow!("Target screen size not registered"))?;

        let matching_template = self
            .template_registry
            .get(&screen_size)
            .ok_or(anyhow::anyhow!(
                "Resized template not found for target size"
            ))?;

        let res_up = matching_template.up.match_template(target)?;
        let res_down = matching_template.down.match_template(target)?;
        let res_right = matching_template.right.match_template(target)?;
        let res_left = matching_template.left.match_template(target)?;

        let arr_up = convert_mat_to_array2(&res_up)?;
        let arr_down = convert_mat_to_array2(&res_down)?;
        let arr_right = convert_mat_to_array2(&res_right)?;
        let arr_left = convert_mat_to_array2(&res_left)?;

        let descriptors = find_direction_commands(
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

        let mut template_up = self.template_original.up.clone();
        template_up.resize_template_scale(inferred_scale)?;
        let mut template_down = self.template_original.down.clone();
        template_down.resize_template_scale(inferred_scale)?;
        let mut template_right = self.template_original.right.clone();
        template_right.resize_template_scale(inferred_scale)?;
        let mut template_left = self.template_original.left.clone();
        template_left.resize_template_scale(inferred_scale)?;

        let template_resized = DirectionTemplateMatcherSet::new(
            template_up,
            template_down,
            template_right,
            template_left,
        )?;

        let rep_template_size = template_resized.up.mat().size()?;
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
pub struct DirectionTemplateMatcherSet {
    pub up: TemplateMatcher,
    pub down: TemplateMatcher,
    pub right: TemplateMatcher,
    pub left: TemplateMatcher,
}

impl DirectionTemplateMatcherSet {
    pub fn new(
        template_up: TemplateMatcher,
        template_down: TemplateMatcher,
        template_right: TemplateMatcher,
        template_left: TemplateMatcher,
    ) -> Result<Self> {
        Ok(Self {
            up: template_up,
            down: template_down,
            right: template_right,
            left: template_left,
        })
    }
}
