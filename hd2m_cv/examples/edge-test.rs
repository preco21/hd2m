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

    let src_img = cv::imgcodecs::imread("./examples/source.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let tmpl_img = cv::imgcodecs::imread("./examples/down.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let start = std::time::Instant::now();

    let mut src_edges = Mat::default();
    let mut template_edges = Mat::default();
    canny(&src_img, &mut src_edges, 150.0, 300.0, 3, true)?;
    canny(&tmpl_img, &mut template_edges, 150.0, 300.0, 3, true)?;

    let mut resized_template_edge = Mat::default();
    cv::imgproc::resize(
        &template_edges,
        &mut resized_template_edge,
        cv::core::Size::default(),
        0.75,
        0.75,
        cv::imgproc::INTER_NEAREST_EXACT,
    )?;

    let mut resize_from_original_canny = Mat::default();
    cv::imgproc::resize(
        &tmpl_img,
        &mut resize_from_original_canny,
        cv::core::Size::default(),
        0.75,
        0.75,
        cv::imgproc::INTER_NEAREST_EXACT,
    )?;
    let mut resize_from_original_canny_edges = Mat::default();
    canny(
        &resize_from_original_canny,
        &mut resize_from_original_canny_edges,
        150.0,
        300.0,
        3,
        true,
    )?;

    println!("Elapsed time: {:?}", start.elapsed());

    let mut gx = Mat::default();
    let mut gy = Mat::default();
    sobel(&src_img, &mut gx, CV_64F, 1, 0, 3, 1.0, 0.0, BORDER_DEFAULT)?;
    sobel(&src_img, &mut gy, CV_64F, 0, 1, 3, 1.0, 0.0, BORDER_DEFAULT)?;

    opencv::imgcodecs::imwrite("edge-result-src.png", &src_edges, &Vector::new())?;
    opencv::imgcodecs::imwrite("edge-result-tmpl.png", &template_edges, &Vector::new())?;
    opencv::imgcodecs::imwrite(
        "edge-result-resize-tmpl.png",
        &resized_template_edge,
        &Vector::new(),
    )?;
    opencv::imgcodecs::imwrite(
        "edge-result-resize-from-original-canny.png",
        &resize_from_original_canny_edges,
        &Vector::new(),
    )?;
    // opencv::imgcodecs::imwrite("edge-result-src-sobel-x.png", &gx, &Vector::new())?;
    // opencv::imgcodecs::imwrite("edge-result-src-sobel-y.png", &gy, &Vector::new())?;

    Ok(())
}
