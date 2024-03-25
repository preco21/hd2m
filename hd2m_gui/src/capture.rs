use std::sync::Arc;

use hd2m_cv::TryIntoCv;
use iced::{futures::SinkExt, subscription, Subscription};
use opencv as cv;
use tokio::sync::{mpsc, oneshot};

use crate::feature::{CaptureManager, CaptureManagerConfig};

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Starting,
    Ready(mpsc::Receiver<Action>),
    Errored,
    Restoring,
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<Action>),
    ResultTakeScreenshot(cv::core::Mat),
}

#[derive(Debug)]
pub enum Action {
    TakeScreenshot,
}

pub fn capture_process_subscription() -> Subscription<Event> {
    struct CaptureSubscription;
    subscription::channel(
        std::any::TypeId::of::<CaptureSubscription>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;

            let (capture_chan_tx, capture_chan_rx) = mpsc::channel(1);
            let capture_manager = Arc::new(
                CaptureManager::new(CaptureManagerConfig {
                    window_title: "Code".to_owned(),
                })
                .unwrap(),
            );

            loop {
                match &mut state {
                    State::Starting => {
                        let (sender, mut receiver) = mpsc::channel(1);

                        tokio::spawn({
                            let capture_manager = capture_manager.clone();
                            async move {
                                capture_manager.start(capture_chan_rx).await.unwrap();
                                // FIXME: Close when the capture manager is done
                            }
                        });

                        let _ = output.send(Event::Ready(sender)).await;
                        state = State::Ready(receiver);
                    }
                    State::Ready(receiver) => {
                        if let Some(action) = receiver.recv().await {
                            match action {
                                Action::TakeScreenshot => {
                                    let (cap_tx, cap_rx) = oneshot::channel();
                                    let _ = capture_chan_tx.send(cap_tx).await.unwrap();
                                    let result = cap_rx.await.unwrap();
                                    let cv: image::RgbaImage = result.try_into_cv().unwrap();
                                    cv.save("screenshot.png").unwrap();
                                    println!("got screenshot");
                                    let _ = output.send(Event::ResultTakeScreenshot(result));
                                }
                            }
                        }
                    }
                    State::Errored => {
                        // FIXME: handle errored state
                    }
                    State::Restoring => {
                        // FIXME: handle recover from error
                    }
                }
            }
        },
    )
}
