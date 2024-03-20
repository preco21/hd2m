use crate::cv_convert::*;
use anyhow::{Error, Result};
use opencv::{self as cv};
use tokio::sync::{mpsc, oneshot};
use windows_capture::{
    capture::GraphicsCaptureApiHandler,
    graphics_capture_api::InternalCaptureControl,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
    window::Window,
};

pub type TriggerCaptureRx = mpsc::Receiver<oneshot::Sender<cv::core::Mat>>;

#[derive(Debug)]
pub struct CaptureManagerConfig {
    pub window_title: String,
}

#[derive(Debug)]
pub struct CaptureManager {
    window_title: String,
}

impl CaptureManager {
    pub fn new(config: CaptureManagerConfig) -> Result<Self> {
        Ok(Self {
            window_title: config.window_title,
        })
    }

    // FIXME: To make it to be able to pass shutdown signals
    pub async fn start(&self, trigger_capture_rx: TriggerCaptureRx) -> Result<()> {
        let window = Window::from_contains_name(&self.window_title)?;
        let settings = Settings::new(
            window,
            CursorCaptureSettings::WithoutCursor,
            DrawBorderSettings::WithoutBorder,
            ColorFormat::Rgba8,
            CaptureConfig { trigger_capture_rx },
        )?;
        let _ = tokio::task::spawn_blocking(move || {
            Capture::start(settings)?;
            Result::<()>::Ok(())
        })
        .await?;
        Ok(())
    }
}

#[derive(Debug)]
struct CaptureConfig {
    pub trigger_capture_rx: TriggerCaptureRx,
}

#[derive(Debug)]
struct Capture {
    trigger_capture_rx: TriggerCaptureRx,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = CaptureConfig;
    type Error = Error;

    fn new(config: CaptureConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            trigger_capture_rx: config.trigger_capture_rx,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut windows_capture::frame::Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let msg = self.trigger_capture_rx.try_recv();
        match msg {
            Ok(cb) => {
                let mat = cv::core::Mat::try_from_cv(frame)?;
                let _ = cb.send(mat);
                Ok(())
            }
            Err(mpsc::error::TryRecvError::Empty) => Ok(()),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                capture_control.stop();
                return Err(Error::msg("Capture trigger channel closed"));
            }
        }
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture Session Closed");
        Ok(())
    }
}
