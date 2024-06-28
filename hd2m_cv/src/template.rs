use anyhow::Result;
use opencv::{self as cv};

pub struct TemplateMatcher {
    pub template_mat: cv::core::Mat,
}

impl TemplateMatcher {
    pub fn new(template: &cv::core::Mat) -> Result<Self> {
        Ok(Self {
            template_mat: canny(template)?,
        })
    }

    pub fn match_template(&self, source: &cv::core::Mat) -> Result<cv::core::Mat> {
        let edged = canny(source)?;
        let mut res = cv::core::Mat::default();
        cv::imgproc::match_template(
            &edged,
            &self.template_mat,
            &mut res,
            cv::imgproc::TM_CCOEFF_NORMED,
            // Use the template as the mask too
            &self.template_mat,
        )?;
        Ok(res)
    }

    pub fn resize_template(&mut self, width: i32, height: i32) -> Result<cv::core::Mat> {
        let mut res = cv::core::Mat::default();
        cv::imgproc::resize(
            &self.template_mat,
            &mut res,
            cv::core::Size::new(width, height),
            0.0,
            0.0,
            cv::imgproc::INTER_NEAREST,
        )?;
        self.template_mat = res;
        Ok(res)
    }

    pub fn resize_template_scale(&mut self, scale: f64) -> Result<()> {
        let mut res = cv::core::Mat::default();
        cv::imgproc::resize(
            &self.template_mat,
            &mut res,
            Default::default(),
            scale,
            scale,
            // This will produce the near-accurate result in terms of pixel patterns as the original template image provided
            // For details: https://stackoverflow.com/questions/5358700/template-match-different-sizes-of-template-and-image
            // See also: https://docs.opencv.org/3.4/da/d54/group__imgproc__transform.html#gga5bb5a1fea74ea38e1a5445ca803ff121aa5521d8e080972c762467c45f3b70e6c
            cv::imgproc::INTER_NEAREST_EXACT,
        )?;
        self.template_mat = res;
        Ok(())
    }
}

fn canny(template: &cv::core::Mat) -> Result<cv::core::Mat> {
    let mut canny_source = cv::core::Mat::default();
    cv::imgproc::canny(&template, &mut canny_source, 150.0, 300.0, 3, true)?;
    Ok(canny_source)
}
