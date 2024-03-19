use std::time::Duration;

use anyhow::Result;
use hd2m_cv::cv_convert::{IntoCv, TryFromCv, TryIntoCv};
use image::RgbaImage;
use opencv::{self as cv, prelude::*};

fn main() -> Result<()> {
    let source_img = image::open("./examples/source2.png")?;
    let source_img_mat: cv::core::Mat = source_img.to_luma_alpha8().try_into_cv()?;
    let source_img_mat2: cv::core::Mat = source_img.to_rgba8().try_into_cv()?;

    let up_img = image::open("./examples/up.png")?;
    let down_img = image::open("./examples/down.png")?;
    let right_img = image::open("./examples/right.png")?;
    let left_img = image::open("./examples/left.png")?;

    let up_img_mat: cv::core::Mat = up_img.to_luma_alpha8().try_into_cv()?;
    let down_img_mat: cv::core::Mat = down_img.to_luma_alpha8().try_into_cv()?;
    let right_img_mat: cv::core::Mat = right_img.to_luma_alpha8().try_into_cv()?;
    let left_img_mat: cv::core::Mat = left_img.to_luma_alpha8().try_into_cv()?;

    println!("source_img_mat: {:?}", source_img_mat);

    let start = std::time::Instant::now();

    let temp = match_template(&up_img_mat, &source_img_mat)?;
    find_min_max_log(&temp, &source_img_mat2, 1)?;
    let temp = match_template(&down_img_mat, &source_img_mat)?;
    find_min_max_log(&temp, &source_img_mat2, 2)?;
    let temp = match_template(&right_img_mat, &source_img_mat)?;
    find_min_max_log(&temp, &source_img_mat2, 3)?;
    let temp = match_template(&left_img_mat, &source_img_mat)?;
    find_min_max_log(&temp, &source_img_mat2, 4)?;

    println!("Elapsed: {:?}", start.elapsed());

    // cv::imgcodecs::imwrite("./result.png", &res, &cv::core::Vector::new())?;

    Ok(())
}

fn match_template(template: &cv::core::Mat, source: &cv::core::Mat) -> Result<cv::core::Mat> {
    let mut res = cv::core::Mat::default();
    cv::imgproc::match_template(
        source,
        template,
        &mut res,
        cv::imgproc::TM_CCOEFF_NORMED,
        &cv::core::no_array(),
    )?;
    Ok(res)
}

fn find_min_max_log(
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

    // let mut dst_img = source.clone();
    // cv::imgproc::rectangle(
    //     &mut dst_img,
    //     cv::core::Rect::from_point_size(max_loc, cv::core::Size::new(30, 30)),
    //     cv::core::VecN([255., 255., 0., 0.]),
    //     2,
    //     cv::imgproc::LINE_8,
    //     0,
    // )?;

    // let i: RgbaImage = dst_img.try_into_cv()?;
    // i.save(format!("./result{num}.png").as_str())?;
    // image::RgbaImage::try_from_cv(, format!("./result{num}.png").as_str())?;

    cv::imgcodecs::imwrite(
        format!("./result{num}.png").as_str(),
        &source,
        &cv::core::Vector::new(),
    )?;

    println!(
        "MinMaxLoc vals: min={:?}, max={:?}, min_loc={:?}, max_loc={:?}",
        min_val, max_val, min_loc, max_loc
    );

    Ok((min_val, max_val, min_loc, max_loc))
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
