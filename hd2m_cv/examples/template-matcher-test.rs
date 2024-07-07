use anyhow::Result;
use hd2m_cv::{Direction, TryFromCv, TryIntoCv};
use image::{DynamicImage, GrayImage, RgbaImage};
use opencv as cv;

fn main() -> Result<()> {
    let img_tmp = image::open("./examples/temp.png")?.to_rgba8();
    let matcher = hd2m_cv::TemplateMatcher::new(&img_tmp.try_into_cv()?)?;

    let source = image::open("./examples/source.png")?;
    let dst_img = source.clone().to_luma8();
    let source = source.to_rgba8();

    let start = std::time::Instant::now();
    let res = matcher.match_template(&source.try_into_cv()?)?;

    let mut min_val = 0.0f64;
    let mut max_val = 0.0f64;
    let mut min_loc = cv::core::Point::default();
    let mut max_loc = cv::core::Point::default();

    println!("Elapsed: {:?}", start.elapsed());
    cv::core::min_max_loc(
        &res,
        Some(&mut min_val),
        Some(&mut max_val),
        Some(&mut min_loc),
        Some(&mut max_loc),
        &cv::core::no_array(),
    )?;

    println!(
        "vals: {:?} {:?} {:?} {:?}",
        min_val, max_val, min_loc, max_loc
    );

    let mut dst_img = dst_img.try_into_cv()?;
    cv::imgproc::rectangle(
        &mut dst_img,
        cv::core::Rect::from_points(min_loc, max_loc),
        cv::core::VecN([255., 255., 0., 0.]),
        2,
        cv::imgproc::LINE_8,
        0,
    )?;

    GrayImage::try_from_cv(&dst_img)?.save("./result.png")?;

    println!("Elapsed: {:?}", start.elapsed());

    Ok(())
}
