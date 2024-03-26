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
        mask,
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
        // This will produce the near-same result in terms of pixel patterns as the original template image provided
        // See also: https://docs.opencv.org/3.4/da/d54/group__imgproc__transform.html#gga5bb5a1fea74ea38e1a5445ca803ff121aa5521d8e080972c762467c45f3b70e6c
        cv::imgproc::INTER_NEAREST_EXACT,
    )?;
    Ok(res)
}
