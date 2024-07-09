use anyhow::Result;
use hd2m_cv::{Direction, TryFromCv, TryIntoCv};
use image::{DynamicImage, GrayImage, RgbaImage};
use opencv as cv;

const MARGIN_Y_ANCHOR: i32 = 22 + 10;
const ANCHOR_BOX: i32 = 25;

const BOX_SIZE: i32 = 70;
const COEFF: f32 = 0.0152777777777778;

fn main() -> Result<()> {
    let start = std::time::Instant::now();

    let source = image::open("./examples/source2.png")?;
    let dst_img = source.clone().to_luma8();
    let source = source.to_rgba8();

    let anchor_img = image::open("./examples/target.png")?.to_rgba8();
    let anchor_mch = hd2m_cv::TemplateMatcher::new(&anchor_img.try_into_cv()?)?;
    let anchor_res = anchor_mch.match_template(&source.try_into_cv()?)?;

    let pos = anchor_res.position()?;

    // Reposition the anchor to the stratgem box
    let pos = cv::core::Point::new(pos.x, pos.y + ANCHOR_BOX + MARGIN_Y_ANCHOR);

    let mut dst_img = dst_img.try_into_cv()?;

    println!("pos: {:?}", pos);

    // cv::imgproc::rectangle(
    //     &mut dst_img,
    //     cv::core::Rect::from_points(pos, cv::core::Point::new(pos.x + 300, pos.y + 500)),
    //     cv::core::VecN([255., 255., 0., 0.]),
    //     2,
    //     cv::imgproc::LINE_8,
    //     0,
    // )?;

    let x = pos.x + 10;
    for i in 0..10 {
        let y = pos.y + i * BOX_SIZE;
        println!("x: {}", x);
        println!("y: {}", y);

        cv::imgproc::rectangle(
            &mut dst_img,
            cv::core::Rect::from_points(
                cv::core::Point::new(x, y),
                cv::core::Point::new(x + 300, y + BOX_SIZE),
            ),
            cv::core::VecN([255., 255., 255., 0.]),
            1,
            cv::imgproc::LINE_8,
            0,
        )?;
    }

    GrayImage::try_from_cv(&dst_img)?.save("./result.png")?;

    println!("Elapsed: {:?}", start.elapsed());

    Ok(())
}
