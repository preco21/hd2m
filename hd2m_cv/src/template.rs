use anyhow::Result;
use opencv::{self as cv};

pub fn match_template_with_mask(
    source: &cv::core::Mat,
    template: &cv::core::Mat,
    mask: Option<&cv::core::Mat>,
) -> Result<cv::core::Mat> {
    let mask = mask.unwrap_or(template);

    let mut res = cv::core::Mat::default();
    cv::imgproc::match_template(
        source,
        template,
        &mut res,
        cv::imgproc::TM_CCORR_NORMED,
        template,
    )?;

    Ok(res)
}

pub fn resize_template_scale(template: &cv::core::Mat, scale: f64) -> Result<cv::core::Mat> {
    let mut res = cv::core::Mat::default();
    cv::imgproc::resize(
        template,
        &mut res,
        Default::default(),
        scale,
        scale,
        cv::imgproc::INTER_LANCZOS4,
    )?;
    Ok(res)
}
