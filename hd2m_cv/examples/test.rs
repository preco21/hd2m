use std::time::Duration;

use anyhow::Result;
use opencv::{self as cv, prelude::*};

fn main() -> Result<()> {
    let source = cv::imgcodecs::imread("./source.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;
    let template = cv::imgcodecs::imread("./template.png", cv::imgcodecs::IMREAD_GRAYSCALE)?;

    let mut res = cv::core::Mat::default();

    let start = std::time::Instant::now();
    cv::imgproc::match_template(
        &source,
        &template,
        &mut res,
        cv::imgproc::TM_CCOEFF_NORMED,
        &cv::core::no_array(),
    )?;

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

    let mut dst_img = source.clone();
    cv::imgproc::rectangle(
        &mut res,
        cv::core::Rect::from_points(min_loc, max_loc),
        cv::core::VecN([255., 255., 0., 0.]),
        -1,
        cv::imgproc::LINE_8,
        0,
    )?;

    cv::imgcodecs::imwrite("./result.png", &res, &cv::core::Vector::new())?;

    Ok(())
}

// $env:path+=";C:\Users\preco\vcpkg\installed\x64-windows\tools\llvm"
// $env:OPENCV_INCLUDE_PATHS = "C:\Users\preco\vcpkg\installed\x64-windows\include"
// $env:OPENCV_LINK_PATHS = "C:\Users\preco\vcpkg\installed\x64-windows\lib"
// $env:OPENCV_LINK_LIBS="opencv_core4,opencv_imgcodecs4,opencv_imgproc4,opencv_bioinspired4,opencv_dnn4,opencv_stitching4,zlib"
// $env:OPENCV_MSVC_CRT="static"
// $env:VCPKGRS_DYNAMIC=0
// $env:VCPKG_LIBRARY_LINKAGE=static

// $env:OPENCV_LINK_LIBS="opencv_core481,opencv_videoio481,opencv_imgcodecs481,opencv_imgproc481,ippiw,ittnotify,ippicvmt,zlib"
// $env:OPENCV_LINK_PATHS="D:/opt/opencv/x64/vc17/staticlib"
// $env:OPENCV_INCLUDE_PATHS="D:/opt/opencv/include"
// $env:OPENCV_MSVC_CRT="static"

// VCPKG를 STATIC LINK로 설치해야 했음

// https://github.com/spoorn/media-to-ascii/blob/a01017fbb3dc883a89b0a1aff237c69b072542b2/.github/workflows/build.yml

// 여기에서 필요한 기능을 가진 STATIC LINK 라이브러리 전부 가져와야 함
// C:\Users\preco\vcpkg\installed\x64-windows\lib
