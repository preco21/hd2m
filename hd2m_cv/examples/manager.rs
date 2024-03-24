use anyhow::Result;
use hd2m_cv::Direction;

fn main() -> Result<()> {
    let mut manager = hd2m_cv::Hd2mCvManager::new(hd2m_cv::Hd2mCvManagerConfig {
        template_up_image: image::open("./examples/up.png")?.to_rgba8(),
        template_down_image: image::open("./examples/down.png")?.to_rgba8(),
        template_right_image: image::open("./examples/right.png")?.to_rgba8(),
        template_left_image: image::open("./examples/left.png")?.to_rgba8(),
        base_screen_size: (2560, 1440),
        search_options: Some(hd2m_cv::Hd2mCvSearchOptions {
            threshold: Some(0.979),
            ..Default::default()
        }),
    })?;

    let start = std::time::Instant::now();
    let source_img = image::open("./examples/source2.png")?;

    // let source_img = source_img.resize(2560, 1440, image::imageops::FilterType::Lanczos3);
    let res = manager.run_match(&source_img.to_rgba8())?;
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
