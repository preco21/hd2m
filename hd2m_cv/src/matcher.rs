use anyhow::Result;
use image::GrayImage;
use opencv::{self as cv};

use crate::TryFromCv;

#[derive(Debug, Clone)]
pub struct TemplateMatcher {
    baked_template: cv::core::Mat,
}

impl TemplateMatcher {
    pub fn new(template: &cv::core::Mat) -> Result<Self> {
        GrayImage::try_from_cv(&pre_process_mat(template)?)?.save("./canny-edges.png")?;

        Ok(Self {
            baked_template: pre_process_mat(template)?,
        })
    }

    pub fn with_resized(template: &cv::core::Mat, width: i32, height: i32) -> Result<Self> {
        let mut tm = Self::new(template)?;
        tm.resize_template(width, height)?;
        Ok(tm)
    }

    pub fn with_resized_scale(template: &cv::core::Mat, scale: f64) -> Result<Self> {
        let mut tm = Self::new(template)?;
        tm.resize_template_scale(scale)?;
        Ok(tm)
    }

    pub fn mat(&self) -> &cv::core::Mat {
        &self.baked_template
    }

    pub fn match_template(&self, input: &cv::core::Mat) -> Result<TemplateMatcherResult> {
        let mut res = cv::core::Mat::default();
        cv::imgproc::match_template(
            &pre_process_mat(input)?,
            &self.baked_template,
            &mut res,
            cv::imgproc::TM_CCORR_NORMED,
            // INFO: We don't use mask here because somehow without mask, the result is more accurate
            &cv::core::no_array(),
        )?;
        Ok(TemplateMatcherResult::new(res))
    }

    pub fn resize_template(&mut self, width: i32, height: i32) -> Result<()> {
        let mut res = cv::core::Mat::default();
        cv::imgproc::resize(
            // FIXME: should we use the original template or the pre-processed one?
            &self.baked_template,
            &mut res,
            cv::core::Size::new(width, height),
            0.0,
            0.0,
            cv::imgproc::INTER_NEAREST_EXACT,
        )?;
        self.baked_template = res;
        Ok(())
    }

    pub fn resize_template_scale(&mut self, scale: f64) -> Result<()> {
        let mut res = cv::core::Mat::default();
        cv::imgproc::resize(
            &self.baked_template,
            &mut res,
            Default::default(),
            scale,
            scale,
            // This will produce the near-accurate result in terms of pixel patterns as the original template image provided
            // For details: https://stackoverflow.com/questions/5358700/template-match-different-sizes-of-template-and-image
            // See also: https://docs.opencv.org/3.4/da/d54/group__imgproc__transform.html#gga5bb5a1fea74ea38e1a5445ca803ff121aa5521d8e080972c762467c45f3b70e6c
            cv::imgproc::INTER_NEAREST_EXACT,
        )?;
        self.baked_template = res;
        Ok(())
    }
}

fn convert_mat_grayscale(mat: &cv::core::Mat) -> Result<cv::core::Mat> {
    let mut res = cv::core::Mat::default();
    cv::imgproc::cvt_color(&mat, &mut res, cv::imgproc::COLOR_RGBA2GRAY, 0)?;
    Ok(res)
}

fn pre_process_mat(mat: &cv::core::Mat) -> Result<cv::core::Mat> {
    let grayed = convert_mat_grayscale(mat)?;

    let mut smoothed = cv::core::Mat::default();
    cv::imgproc::gaussian_blur(
        &grayed,
        &mut smoothed,
        cv::core::Size::new(3, 3),
        0.0,
        0.0,
        0,
    )?;

    let mut canny_edges = cv::core::Mat::default();
    cv::imgproc::canny(&smoothed, &mut canny_edges, 150.0, 300.0, 3, true)?;

    Ok(canny_edges)
}

#[derive(Debug, Clone)]
pub struct TemplateMatcherResult {
    match_mat: cv::core::Mat,
}

impl TemplateMatcherResult {
    pub(crate) fn new(mat: cv::core::Mat) -> Self {
        Self { match_mat: mat }
    }

    pub fn mat(&self) -> &cv::core::Mat {
        &self.match_mat
    }

    pub fn min_max_loc(&self) -> Result<(f64, f64, cv::core::Point, cv::core::Point)> {
        let mut min_val = 0.0f64;
        let mut max_val = 0.0f64;
        let mut min_loc = cv::core::Point::default();
        let mut max_loc = cv::core::Point::default();
        cv::core::min_max_loc(
            &self.match_mat,
            Some(&mut min_val),
            Some(&mut max_val),
            Some(&mut min_loc),
            Some(&mut max_loc),
            &cv::core::no_array(),
        )?;
        Ok((min_val, max_val, min_loc, max_loc))
    }

    pub fn position(&self) -> Result<cv::core::Point> {
        let (_min_val, _max_val, _min_loc, max_loc) = self.min_max_loc()?;
        Ok(max_loc)
    }
}

impl From<TemplateMatcherResult> for cv::core::Mat {
    fn from(res: TemplateMatcherResult) -> Self {
        res.match_mat
    }
}

impl AsRef<cv::core::Mat> for TemplateMatcherResult {
    fn as_ref(&self) -> &cv::core::Mat {
        &self.match_mat
    }
}
