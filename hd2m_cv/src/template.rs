use anyhow::Result;
use opencv::{self as cv};

#[derive(Debug, Clone)]
pub struct TemplateMatcher {
    original_template: cv::core::Mat,
}

impl TemplateMatcher {
    pub fn new(template: &cv::core::Mat) -> Result<Self> {
        Ok(Self {
            original_template: convert_mat_grayscale(template)?,
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
        &self.original_template
    }

    pub fn match_template(&self, source: &cv::core::Mat) -> Result<cv::core::Mat> {
        let grayed = convert_mat_grayscale(source)?;
        let mut res = cv::core::Mat::default();
        cv::imgproc::match_template(
            &grayed,
            &self.original_template,
            &mut res,
            cv::imgproc::TM_CCOEFF_NORMED,
            // INFO: We don't use mask here because somehow without mask, the result is more accurate
            &cv::core::no_array(),
        )?;
        Ok(res)
    }

    pub fn resize_template(&mut self, width: i32, height: i32) -> Result<()> {
        let mut res = cv::core::Mat::default();
        cv::imgproc::resize(
            &self.original_template,
            &mut res,
            cv::core::Size::new(width, height),
            0.0,
            0.0,
            cv::imgproc::INTER_NEAREST_EXACT,
        )?;
        self.original_template = res;
        Ok(())
    }

    pub fn resize_template_scale(&mut self, scale: f64) -> Result<()> {
        let mut res = cv::core::Mat::default();
        cv::imgproc::resize(
            &self.original_template,
            &mut res,
            Default::default(),
            scale,
            scale,
            // This will produce the near-accurate result in terms of pixel patterns as the original template image provided
            // For details: https://stackoverflow.com/questions/5358700/template-match-different-sizes-of-template-and-image
            // See also: https://docs.opencv.org/3.4/da/d54/group__imgproc__transform.html#gga5bb5a1fea74ea38e1a5445ca803ff121aa5521d8e080972c762467c45f3b70e6c
            cv::imgproc::INTER_NEAREST_EXACT,
        )?;
        self.original_template = res;
        Ok(())
    }
}

fn convert_mat_grayscale(mat: &cv::core::Mat) -> Result<cv::core::Mat> {
    let mut res = cv::core::Mat::default();
    cv::imgproc::cvt_color(&mat, &mut res, cv::imgproc::COLOR_RGBA2GRAY, 0)?;
    Ok(res)
}
