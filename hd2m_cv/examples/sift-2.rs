use std::{default, time::Duration};

use anyhow::Result;
use opencv::{
    self as cv,
    core::{no_array, DMatch, Rect, Scalar, Vector},
    features2d::{BFMatcher, FlannBasedMatcher, ORB_ScoreType, ORB, SIFT},
    flann::{IndexParams, SearchParams, FLANN_INDEX_KDTREE},
    prelude::*,
    types::{
        PtrOfIndexParams, PtrOfSearchParams, VectorOfDMatch, VectorOfKeyPoint, VectorOfPoint2f,
        VectorOfVectorOfDMatch,
    },
};

fn main() -> Result<()> {
    let mut input_img =
        cv::imgcodecs::imread("./examples/source2.png", cv::imgcodecs::IMREAD_COLOR)?;
    let src_img = cv::imgcodecs::imread("./examples/source2.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let tmpl_img = cv::imgcodecs::imread("./examples/down.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;

    let start = std::time::Instant::now();

    let mut orb = ORB::create(0, 1.0, 8, 31, 0, 2, ORB_ScoreType::HARRIS_SCORE, 31, 20)?;
    let mut orb = SIFT::create_def()?;
    let mut orb = SIFT::create(0, 3, 0.04, 10.0, 1.6, true)?;

    let mut keypoints_template = VectorOfKeyPoint::new();
    let mut descriptors_template = Mat::default();
    orb.detect_and_compute(
        &tmpl_img,
        // &tmpl_img,
        &Mat::default(),
        &mut keypoints_template,
        &mut descriptors_template,
        false,
    )?;

    let mut keypoints_img = VectorOfKeyPoint::new();
    let mut descriptors_img = Mat::default();
    orb.detect_and_compute(
        &src_img,
        &Mat::default(),
        &mut keypoints_img,
        &mut descriptors_img,
        false,
    )?;

    let mut index_params = IndexParams::default()?;
    index_params.set_algorithm(FLANN_INDEX_KDTREE)?;
    index_params.set_int("trees", 5)?;
    let index_params = PtrOfIndexParams::new(index_params);

    let search_params = SearchParams::new_1(10, 0.0, true)?;
    let search_params = PtrOfSearchParams::new(search_params);

    let bf = FlannBasedMatcher::new(&index_params, &search_params)?;
    let mut matches = Vector::<DMatch>::default();
    bf.train_match(
        &descriptors_template,
        &descriptors_img,
        &mut matches,
        &Mat::default(),
    )?;

    let mut best_match = VectorOfVectorOfDMatch::new();
    let k = 2; // Finding the optimal two points.

    // This line is valid until here.
    bf.knn_train_match(
        &descriptors_template,
        &descriptors_img,
        &mut best_match,
        k,
        &Mat::default(),
        false,
    )?;

    // Filtering good key points.
    let mut result = VectorOfVectorOfDMatch::new();

    for line in &best_match {
        let mut list = VectorOfDMatch::new();

        for singe in line {
            // The lower the value, the higher the similarity.
            if singe.distance < 0.7 {
                list.push(singe);
            }
        }

        result.push(list);
    }

    println!("best_match: {:?}", best_match);

    // if best_match.len() >= 4 {
    //     let mut src_pts = VectorOfPoint2f::new();
    //     let mut dst_pts = VectorOfPoint2f::new();
    //     for key_point in best_match {
    //         for elem in key_point {
    //             let query_idx = keypoints_template.get(elem.query_idx as usize)?;
    //             src_pts.push(query_idx.pt());

    //             let train_idx = keypoints_img.get(elem.train_idx as usize)?;
    //             dst_pts.push(train_idx.pt());
    //         }
    //     }

    //     // Random sampling   also is 5
    //     let mut h = cv::imgproc::find_homography(&src_pts, &dst_pts, &mut no_array(), RANSAC, 5f64)?;

    //     let weight = h.size()?.width;
    //     let height = h.size()?.height;

    //     let mut pts = VectorOfPoint2f::new();
    //     pts.push(Point2f::new(0f32, 0f32));
    //     pts.push(Point2f::new(0f32, (height - 1) as f32));
    //     pts.push(Point2f::new((weight - 1) as f32, (height - 1) as f32));
    //     pts.push(Point2f::new((weight - 1) as f32, 0f32));

    //     // This line throws an error
    //     // perspective_transform(&pts, &mut h, &no_array())?;

    //     // polylines_def(&mut dst_mat, &pts, true, Scalar::from((0, 0, 255)))?;
    //     //
    //     // // 绘制关键点
    //     // let mut net_mat = Mat::default();
    //     // draw_matches_knn_def(
    //     //     &src_mat,
    //     //     &keypoints_template,
    //     //     &dst_mat,
    //     //     &keypoints_img,
    //     //     &result,
    //     //     &mut net_mat,
    //     // )?;
    //     //
    //     // imshow("ssd", &h)?;

    //     /// wait_key(100000)?;
    // } else {
    //     println!("array len must >=4");
    //     exit(0)
    // }

    // // Save the output image
    // let output_image_path = "res.png";
    // opencv::imgcodecs::imwrite(output_image_path, &input_img, &opencv::core::Vector::new())?;
    println!("Elapsed: {:?}", start.elapsed());

    Ok(())
}
