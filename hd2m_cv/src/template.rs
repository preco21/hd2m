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
