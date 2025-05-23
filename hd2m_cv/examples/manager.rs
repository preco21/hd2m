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
            threshold: Some(0.6),
            ..Default::default()
        }),
    })?;

    let start = std::time::Instant::now();
    let source_img = image::open("./examples/source.png")?;
    manager.use_screen_size(source_img.width() as usize, source_img.height() as usize)?;

    let res = manager.run_match_rgba(&source_img.to_rgba8())?;
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
