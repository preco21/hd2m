use std::time::Duration;

use anyhow::Result;
use opencv::{
    self as cv,
    core::{cart_to_polar, no_array, DMatch, Point, Rect, Scalar, Vector, BORDER_DEFAULT, CV_64F},
    features2d::{BFMatcher, FlannBasedMatcher, ORB_ScoreType, ORB, SIFT},
    flann::{IndexParams, SearchParams, FLANN_INDEX_KDTREE},
    imgproc::{
        self, canny, match_shapes, rectangle, sobel, CHAIN_APPROX_SIMPLE, CONTOURS_MATCH_I1,
        RETR_TREE,
    },
    prelude::*,
    types::{
        PtrOfIndexParams, PtrOfSearchParams, VectorOfDMatch, VectorOfKeyPoint,
        VectorOfVectorOfPoint,
    },
};

fn main() -> Result<()> {
    let mut input_img =
        cv::imgcodecs::imread("./examples/source2.png", cv::imgcodecs::IMREAD_COLOR)?;

    let src_img = cv::imgcodecs::imread("./examples/source2.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let tmpl_img = cv::imgcodecs::imread("./examples/down.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let start = std::time::Instant::now();

    let mut src_edges = Mat::default();
    let mut template_edges = Mat::default();
    canny(&src_img, &mut src_edges, 100.0, 200.0, 3, false)?;
    canny(&tmpl_img, &mut template_edges, 100.0, 200.0, 3, false)?;

    // Find contours in the template image
    let mut template_contours = VectorOfVectorOfPoint::new();
    let mut template_hierarchy = Mat::default();
    opencv::imgproc::find_contours(
        &template_edges,
        &mut template_contours,
        RETR_TREE,
        CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    )?;

    println!("Edges: {:?}", template_contours);

    let mut gx = Mat::default();
    let mut gy = Mat::default();
    sobel(&src_img, &mut gx, CV_64F, 1, 0, 3, 1.0, 0.0, BORDER_DEFAULT)?;
    sobel(&src_img, &mut gy, CV_64F, 0, 1, 3, 1.0, 0.0, BORDER_DEFAULT)?;

    // Compute magnitude and direction
    let mut magnitude = Mat::default();
    let mut direction = Mat::default();
    cart_to_polar(&gx, &gy, &mut magnitude, &mut direction, false)?;

    // Prepare to store the results
    #[derive(Debug)]
    struct RectInfo {
        rect: Rect,
        angle: f64,
    }

    let mut results: Vec<RectInfo> = Vec::new();

    // Template matching using contours (approximate approach)
    // for contour in template_contours.clone() {
    //     let bound_rect = imgproc::bounding_rect(&contour)?;

    //     // Run the contour matching on the source image
    //     for i in 0..(src_edges.rows() - bound_rect.height) {
    //         for j in 0..(src_edges.cols() - bound_rect.width) {
    //             let rect = Rect::new(j, i, bound_rect.width, bound_rect.height);
    //             let roi = Mat::roi(&src_edges, rect)?;

    //             let mut roi_contours = VectorOfVectorOfPoint::new();
    //             imgproc::find_contours(
    //                 &roi,
    //                 &mut roi_contours,
    //                 RETR_TREE,
    //                 CHAIN_APPROX_SIMPLE,
    //                 Point::new(0, 0),
    //             )?;

    //             // Check if the contours match (approximate match)
    //             if roi_contours.len() == template_contours.len() {
    //                 let mut match_found = true;
    //                 for k in 0..template_contours.len() {
    //                     if match_shapes(
    //                         &template_contours.get(k)?,
    //                         &roi_contours.get(k)?,
    //                         CONTOURS_MATCH_I1,
    //                         0.0,
    //                     )? > 0.1
    //                     {
    //                         match_found = false;
    //                         break;
    //                     }
    //                 }

    //                 if match_found {
    //                     let angle = *direction
    //                         .at_2d::<f64>(i + bound_rect.height / 2, j + bound_rect.width / 2)?;
    //                     results.push(RectInfo { rect, angle });
    //                 }
    //             }
    //         }
    //     }
    // }

    println!("Results: {:?}", results);
    println!("Elapsed: {:?}", start.elapsed());

    // Draw rectangles and save the result image
    let mut output_image = input_img.clone();
    for result in results {
        rectangle(
            &mut output_image,
            result.rect,
            Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            8,
            0,
        )?;
    }
    opencv::imgcodecs::imwrite("result.png", &output_image, &Vector::new())?;

    Ok(())
}
