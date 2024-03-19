use std::time::Duration;

use anyhow::Result;
use hd2m_cv::cv_convert::{IntoCv, TryFromCv, TryIntoCv};
use image::RgbaImage;
use opencv::{self as cv, core::Mat, features2d, prelude::*};

fn main() -> Result<()> {
    let source_img = image::open("./examples/source.png")?;
    let source_img_mat: cv::core::Mat = source_img.to_luma_alpha8().try_into_cv()?;
    let source_img_mat2: cv::core::Mat = source_img.to_rgba8().try_into_cv()?;

    let up_img = image::open("./examples/up.png")?;
    let up_img_mat: cv::core::Mat = up_img.to_luma_alpha8().try_into_cv()?;

    let start = std::time::Instant::now();

    // let mut sift = cv::features2d::SIFT::create(7, 3, 0.02, 10., 1.6, true)?;
    let mut sift = cv::features2d::SIFT::create(10, 3, 0.09, 10., 1.6, false)?;
    // let mut orb = cv::features2d::ORB::create(
    //     500,
    //     1.2,
    //     8,
    //     31,
    //     0,
    //     2,
    //     cv::features2d::ORB_ScoreType::HARRIS_SCORE,
    //     31,
    //     20,
    // )?;

    let mut keypoints1 = cv::core::Vector::<cv::core::KeyPoint>::new();
    let mut descriptors1 = Mat::default();
    let mut keypoints2 = cv::core::Vector::<cv::core::KeyPoint>::new();
    let mut descriptors2 = Mat::default();

    sift.detect_and_compute(
        &source_img_mat,
        &cv::core::no_array(),
        &mut keypoints1,
        &mut descriptors1,
        false,
    )
    .unwrap();

    sift.detect_and_compute(
        &up_img_mat,
        &cv::core::no_array(),
        &mut keypoints2,
        &mut descriptors2,
        false,
    )
    .unwrap();

    println!("Keypoints1: {:?}", keypoints1.len());
    println!("Keypoints2: {:?}", keypoints2.len());

    // Match descriptors between the two images
    // let mut matches = cv::core::Vector::<cv::core::DMatch>::new();
    // let mut matcher = features2d::DescriptorMatcher::create_with_matcher_type(
    //     features2d::DescriptorMatcher_MatcherType::FLANNBASED,
    // )
    // .unwrap();
    // matcher
    //     .match_(&descriptors1, &mut matches, &cv::core::no_array())
    //     .unwrap();
    let mut matches = cv::core::Vector::<cv::core::DMatch>::new();
    let mut matcher = features2d::DescriptorMatcher::create_with_matcher_type(
        features2d::DescriptorMatcher_MatcherType::FLANNBASED,
    )
    .unwrap();
    matcher
        .train_match(
            &descriptors1,
            &descriptors2,
            &mut matches,
            &cv::core::no_array(),
        )
        .unwrap();

    println!("Matches: {:?}", matches.len());

    // Define a vector to store matching points in the target image
    let mut matching_points: Vec<(f32, f32)> = Vec::new();

    // Iterate through matches and extract matching points' positions in the target image
    let mut dst_img = source_img_mat2.clone();

    for match_idx in 0..matches.len() {
        let train_idx = matches.get(match_idx)?.train_idx as usize;
        let keypoint = keypoints1.get(train_idx)?;
        let point = keypoint.pt();
        cv::imgproc::rectangle(
            &mut dst_img,
            cv::core::Rect::from_point_size(
                cv::core::Point::new(point.x as i32, point.y as i32),
                cv::core::Size::new(30, 30),
            ),
            cv::core::VecN([255., 255., 0., 0.]),
            2,
            cv::imgproc::LINE_8,
            0,
        )?;
        matching_points.push((point.x, point.y));
    }

    // Print or use the matching points as needed
    println!("Matching points in target image: {:?}", matching_points);

    let i: RgbaImage = dst_img.try_into_cv()?;
    i.save(format!("./result.png").as_str())?;
    // image::RgbaImage::try_from_cv(, format!("./result{num}.png").as_str())?;

    println!("Elapsed: {:?}", start.elapsed());

    Ok(())
}
