use std::{
    io::{self, Write},
    os::raw::c_void,
    time::Instant,
};

use hd2m_search::cv_convert::*;
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
    // The type of flags used to get the values from the settings.
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

    // Called every time a new frame is available.
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        print!(
            "\rRecording for: {} seconds",
            self.start.elapsed().as_secs()
        );
        io::stdout().flush()?;

        // Send the frame to the video encoder
        // self.encoder.as_mut().unwrap().encode(
        //     frame.buffer().unwrap().as_raw_buffer(),
        //     1920,
        //     1080,
        // )?;
        // self.encoder.as_mut().unwrap().send_frame(frame)?;

        // Note: The frame has other uses too for example you can save a single for to a file like this:
        // frame.save_as_image("frame.png", ImageFormat::Png)?;
        // Or get the raw data like this so you have full control:
        let mut data = frame.buffer()?;
        // Stop the capture after 6 seconds
        if self.start.elapsed().as_secs() >= 1 {
            // Finish the encoder and save the video.
            // self.encoder.take().unwrap().finish()?;
            let width = data.width();
            let height = data.height();
            // let mat = cv::core::Mat::from_slice_rows_cols(
            //     data.as_raw_nopadding_buffer()?,
            //     height as usize,
            //     width as usize,
            // )?;
            // let mat = cv::core::Mat::from_slice(data.as_raw_nopadding_buffer()?)?;
            println!("width: {}, height: {}", width, height);

            // Frame -> Mat
            // image -> Mat
            // (optional) Mat -> image
            // Mat -> ndarray

            let original_buf = data.as_raw_buffer().to_vec();
            unsafe {
                let buf = original_buf.clone();
                let mat = cv::core::Mat::new_rows_cols_with_data(
                    width as i32,
                    height as i32,
                    cv::core::CV_8UC4,
                    buf.as_ptr() as *mut c_void,
                    cv::core::Mat_AUTO_STEP,
                )?;

                let img: RgbImage = mat.try_into_cv()?;

                img.save_with_format("test.png", image::ImageFormat::Png)?;

                // let img = RgbaImage::from_raw(width, height, mat.data_bytes()?.to_owned()).unwrap();
                // img.save_with_format("test.png", image::ImageFormat::Png)?;

                // let img = RgbaImage::from_raw(width, height, original_buf).unwrap();
                // img.save_with_format("test2.png", image::ImageFormat::Png)?;
                // img.save("test.png")?;
            }

            capture_control.stop();

            // Because there wasn't any new lines in previous prints
            println!();
        }

        Ok(())
    }

    // Optional handler called when the capture item (usually a window) closes.
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
