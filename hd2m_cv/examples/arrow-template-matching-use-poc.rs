use std::time::Duration;

use anyhow::Result;
use hd2m_cv::{
    convert_image_to_mat_grayscale, convert_tm_mat_to_array2, find_direction_commands,
    match_template_with_mask, Direction, TryIntoCv,
};
use image::{RgbImage, RgbaImage};
use ndarray::*;
use opencv::{self as cv, prelude::*};

fn main() -> Result<()> {
    let source_img = image::open("./examples/source2.png")?;
    // let source_img = source_img.resize(2560, 1440, image::imageops::FilterType::Lanczos3);
    let source_mat = convert_image_to_mat_grayscale(&source_img.to_rgba8())?;
    let output_source_mat: cv::core::Mat = source_img.to_rgba8().try_into_cv()?;

    let up_img = image::open("./examples/up.png")?;
    let up_mat = convert_image_to_mat_grayscale(&up_img.to_rgba8())?;
    let up_match_result = match_template_with_mask(&source_mat, &up_mat, None)?;
    let up_tm_array = convert_tm_mat_to_array2(&up_match_result)?;

    let down_img = image::open("./examples/down.png")?;
    let down_mat = convert_image_to_mat_grayscale(&down_img.to_rgba8())?;
    let down_match_result = match_template_with_mask(&source_mat, &down_mat, None)?;
    let down_tm_array = convert_tm_mat_to_array2(&down_match_result)?;

    let right_img = image::open("./examples/right.png")?;
    let right_mat = convert_image_to_mat_grayscale(&right_img.to_rgba8())?;
    let right_match_result = match_template_with_mask(&source_mat, &right_mat, None)?;
    let right_tm_array = convert_tm_mat_to_array2(&right_match_result)?;

    let left_img = image::open("./examples/left.png")?;
    let left_mat = convert_image_to_mat_grayscale(&left_img.to_rgba8())?;
    let left_match_result = match_template_with_mask(&source_mat, &left_mat, None)?;
    let left_tm_array = convert_tm_mat_to_array2(&left_match_result)?;

    println!(
        "source image width/height: {:?}",
        (source_img.width(), source_img.height())
    );

    // report_min_max_log(&up_match_result, &output_source_mat, 1)?;
    // report_min_max_log(&down_match_result, &output_source_mat, 2)?;
    // report_min_max_log(&right_match_result, &output_source_mat, 3)?;
    // report_min_max_log(&left_match_result, &output_source_mat, 4)?;

    let start = std::time::Instant::now();
    let res = find_direction_commands(
        &up_tm_array.view(),
        &down_tm_array.view(),
        &right_tm_array.view(),
        &left_tm_array.view(),
        Some(0.987),
        Some(30),
        Some(20.0),
    )?;

    let mut dst_img: Mat = source_img.try_into_cv()?;
    for (i, row) in res.iter().enumerate() {
        for im in row.iter() {
            let color = match im.direction {
                hd2m_cv::Direction::Up => cv::core::VecN([0., 255., 0., 255.]),
                hd2m_cv::Direction::Down => cv::core::VecN([0., 0., 255., 255.]),
                hd2m_cv::Direction::Right => cv::core::VecN([255., 0., 0., 255.]),
                hd2m_cv::Direction::Left => cv::core::VecN([0., 255., 255., 255.]),
            };
            cv::imgproc::rectangle(
                &mut dst_img,
                cv::core::Rect::from_point_size(
                    cv::core::Point::new(im.position.x as i32, im.position.y as i32),
                    cv::core::Size::new(23, 23),
                ),
                color,
                2,
                cv::imgproc::LINE_8,
                0,
            )?;
        }
    }
    let i: RgbImage = dst_img.try_into_cv()?;
    // let i: RgbaImage = dst_img.try_into_cv()?;
    i.save(format!("./result.png").as_str())?;

    println!(
        "Res: {:?}",
        res.iter()
            .map(|e| e.iter().map(|e| e.direction).collect::<Vec<Direction>>())
            .collect::<Vec<_>>()
    );
    println!();
    println!("Elapsed: {:?}", start.elapsed());

    Ok(())
}

fn report_min_max_log(
    mat: &cv::core::Mat,
    source: &cv::core::Mat,
    num: usize,
) -> Result<(f64, f64, cv::core::Point, cv::core::Point)> {
    let mut min_val = 0.0f64;
    let mut max_val = 0.0f64;
    let mut min_loc = cv::core::Point::default();
    let mut max_loc = cv::core::Point::default();

    cv::core::min_max_loc(
        &mat,
        Some(&mut min_val),
        Some(&mut max_val),
        Some(&mut min_loc),
        Some(&mut max_loc),
        &cv::core::no_array(),
    )?;

    let mut dst_img = source.clone();
    cv::imgproc::rectangle(
        &mut dst_img,
        cv::core::Rect::from_point_size(max_loc, cv::core::Size::new(30, 30)),
        cv::core::VecN([255., 255., 0., 0.]),
        2,
        cv::imgproc::LINE_8,
        0,
    )?;

    println!(
        "MinMaxLoc vals: min={:?}, max={:?}, min_loc={:?}, max_loc={:?}",
        min_val, max_val, min_loc, max_loc
    );

    let i: RgbaImage = dst_img.try_into_cv()?;
    i.save(format!("./result{num}.png").as_str())?;

    Ok((min_val, max_val, min_loc, max_loc))
}
