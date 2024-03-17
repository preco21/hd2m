use std::{
    io::{self, Write},
    os::raw::c_void,
    time::Instant,
};

use image::{RgbImage, RgbaImage};
use opencv::{self as cv, prelude::*};
use windows_capture::{
    capture::GraphicsCaptureApiHandler,
    encoder::{ImageEncoder, VideoEncoder, VideoEncoderQuality, VideoEncoderType},
    frame::{Frame, ImageFormat},
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};

// This struct will be used to handle the capture events.
pub struct Capture {
    // The video encoder that will be used to encode the frames.
    encoder: Option<ImageEncoder>,
    // To measure the time the capture has been running
    start: Instant,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = String;

    // The type of error that can occur during capture, the error will be returned from `CaptureControl` and `start` functions.
    type Error = Box<dyn std::error::Error + Send + Sync>;

    // Function that will be called to create the struct. The flags can be passed from settings.
    fn new(message: Self::Flags) -> Result<Self, Self::Error> {
        println!("Got The Flag: {message}");

        let encoder =
            ImageEncoder::new(windows_capture::frame::ImageFormat::Png, ColorFormat::Rgba8);

        Ok(Self {
            encoder: Some(encoder),
            start: Instant::now(),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if self.start.elapsed().as_secs() >= 1 {
            let mat = cv::core::Mat::try_from_cv(frame)?;
            let img: RgbaImage = mat.try_into_cv()?;
            img.save_with_format("test.png", image::ImageFormat::Png)?;

            capture_control.stop();

            // Because there wasn't any new lines in previous prints
            println!();
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture Session Closed");

        Ok(())
    }
}

pub fn start_capture() {
    // Gets The Foreground Window, Checkout The Docs For Other Capture Items
    let primary_monitor = Monitor::primary().expect("There is no primary monitor");

    let settings = Settings::new(
        // Item To Captue
        primary_monitor,
        // Capture Cursor Settings
        CursorCaptureSettings::WithoutCursor,
        // Draw Borders Settings
        DrawBorderSettings::WithoutBorder,
        // The desired color format for the captured frame.
        ColorFormat::Rgba8,
        // Additional flags for the capture settings that will be passed to user defined `new` function.
        "Yea This Works".to_string(),
    )
    .unwrap();

    // Starts the capture and takes control of the current thread.
    // The errors from handler trait will end up here
    Capture::start(settings).expect("Screen Capture Failed");
}
