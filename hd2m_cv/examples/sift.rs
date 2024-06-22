use std::time::Duration;

use anyhow::Result;
use opencv::{
    self as cv,
    core::{no_array, DMatch, Vector},
    features2d::{BFMatcher, FlannBasedMatcher, ORB_ScoreType, ORB, SIFT},
    flann::{IndexParams, SearchParams, FLANN_INDEX_KDTREE},
    prelude::*,
    types::{PtrOfIndexParams, PtrOfSearchParams, VectorOfDMatch, VectorOfKeyPoint},
};

fn main() -> Result<()> {
    let src_img = cv::imgcodecs::imread("./examples/source2.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let tmpl_img = cv::imgcodecs::imread("./examples/down.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;

    let mut res = cv::core::Mat::default();

    let start = std::time::Instant::now();

    let mut sift = SIFT::create(0, 3, 0.04, 10.0, 1.6, true)?;
    // let mut orb = ORB::create(500, 1.2, 8, 31, 0, 2, ORB_ScoreType::HARRIS_SCORE, 31, 20)?;

    let mut src_keypoints = cv::core::Vector::<cv::core::KeyPoint>::new();
    let mut src_descriptors = Mat::default();
    let mut tmpl_keypoints = cv::core::Vector::<cv::core::KeyPoint>::new();
    let mut tmpl_descriptors = Mat::default();

    sift.detect_and_compute(
        &src_img,
        &Mat::default(),
        &mut src_keypoints,
        &mut src_descriptors,
        false,
    )?;
    sift.detect_and_compute(
        &tmpl_img,
        // &tmpl_img,
        &Mat::default(),
        &mut tmpl_keypoints,
        &mut tmpl_descriptors,
        false,
    )?;

    // // Match descriptors using BFMatcher
    // let mut bf_matcher = BFMatcher::create(cv::core::NORM_L2, true)?;
    // let mut matches = VectorOfDMatch::new();
    // bf_matcher.train_match(
    //     &src_descriptors,
    //     &tmpl_descriptors,
    //     &mut matches,
    //     &no_array(),
    // )?;

    let mut index_params = IndexParams::default()?;
    index_params.set_algorithm(FLANN_INDEX_KDTREE)?;
    index_params.set_int("trees", 5)?;
    let index_params = PtrOfIndexParams::new(index_params);

    let search_params = SearchParams::new_1(50, 0.0, true)?;
    let search_params = PtrOfSearchParams::new(search_params);

    let bf = FlannBasedMatcher::new(&index_params, &search_params)?;
    let mut matches = Vector::<DMatch>::default();
    bf.train_match(
        &tmpl_descriptors,
        &src_descriptors,
        &mut matches,
        &Mat::default(),
    )?;

    println!("matches: {:?}", matches);
    // // Sort matches by distance
    // let mut matches_vec = matches.to_vec();
    // matches_vec.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

    // Take the top matches (you can adjust this number)
    // let num_good_matches = 50;
    // let good_matches = &matches_vec[..num_good_matches];

    // src_keypoints.get(0).unwrap().angle()

    let ratio_thresh = 0.75;
    let mut good_matches = VectorOfDMatch::new();
    for i in 0..matches.len() - 1 {
        if matches.get(i)?.distance < ratio_thresh * matches.get(i + 1)?.distance {
            good_matches.push(matches.get(i)?);
        }
    }

    // Sort matches by distance and take the top matches
    let mut matches_vec = good_matches.to_vec();
    matches_vec.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

    // Take the top matches (you can adjust this number)
    let num_good_matches = 60.min(matches_vec.len());
    let good_matches = &matches_vec[..num_good_matches];

    // Draw rectangles around matching points in the source image
    let mut result_img = src_img.clone();
    for m in good_matches {
        let kp1 = src_keypoints.get(m.query_idx as usize)?;
        let size = 20.0; // Size of the rectangle; adjust as necessary
        let rect = cv::core::Rect::new(
            (kp1.pt().x - size / 2.0).max(0.0) as i32,
            (kp1.pt().y - size / 2.0).max(0.0) as i32,
            size as i32,
            size as i32,
        );
        cv::imgproc::rectangle(
            &mut result_img,
            rect,
            cv::core::Scalar::all(0.0),
            2,
            cv::imgproc::LINE_8,
            0,
        )?;
    }

    // Save the resulting image with rectangles
    cv::imgcodecs::imwrite(
        "result.png",
        &result_img,
        &opencv::types::VectorOfi32::new(),
    )?;

    // Calculate rotation angles
    let mut angles = vec![];
    for m in good_matches {
        let kp1 = src_keypoints.get(m.query_idx as usize)?;
        let kp2 = tmpl_keypoints.get(m.train_idx as usize)?;

        let angle = kp2.angle() - kp1.angle();
        angles.push(angle);
    }

    // Output the matching points and rotation angles
    for (i, m) in good_matches.iter().enumerate() {
        let kp1 = src_keypoints.get(m.query_idx as usize)?;
        let kp2 = tmpl_keypoints.get(m.train_idx as usize)?;

        println!(
            "Match {}: Source Point: ({}, {}), Template Point: ({}, {}), Angle: {}",
            i,
            kp1.pt().x,
            kp1.pt().y,
            kp2.pt().x,
            kp2.pt().y,
            angles[i]
        );
    }
    println!("Elapsed: {:?}", start.elapsed());

    // println!(
    //     "vals: {:?} {:?} {:?} {:?}",
    //     min_val, max_val, min_loc, max_loc
    // );

    // let mut dst_img = source.clone();
    // cv::imgproc::rectangle(
    //     &mut res,
    //     cv::core::Rect::from_points(min_loc, max_loc),
    //     cv::core::VecN([255., 255., 0., 0.]),
    //     -1,
    //     cv::imgproc::LINE_8,
    //     0,
    // )?;

    // cv::imgcodecs::imwrite("./result.png", &res, &cv::core::Vector::new())?;

    Ok(())
}
