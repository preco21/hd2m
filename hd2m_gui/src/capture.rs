use crate::{
    feature::{CaptureManager, CaptureManagerConfig},
    util::Shutdown,
};
use hd2m_cv::TryIntoCv;
use iced::{
    futures::{future, SinkExt},
    subscription, Subscription,
};
use opencv as cv;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<Input>),
    ResultTakeScreenshot(cv::core::Mat),
}

#[derive(Debug)]
pub enum Input {
    TakeScreenshot,
}

pub fn capture_process_subscription(shutdown: Shutdown) -> Subscription<Event> {
    struct CaptureSubscription;
    subscription::channel(
        std::any::TypeId::of::<CaptureSubscription>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;

            let (capture_chan_tx, capture_chan_rx) = mpsc::channel(1);
            tokio::spawn({
                let capture_manager = CaptureManager::new(CaptureManagerConfig {
                    window_title: "Code".to_owned(),
                })
                .unwrap();
                async move {
                    capture_manager.start(capture_chan_rx).await.unwrap();
                    // FIXME: Close when the capture manager is done
                }
            });

            loop {
                match &mut state {
                    State::Starting => {
                        let (sender, receiver) = mpsc::channel(1);
                        let _ = output.send(Event::Ready(sender)).await;
                        state = State::Ready(receiver);
                    }
                    State::Ready(receiver) => {
                        tokio::select! {
                            Some(action) = receiver.recv() => {
                                println!("got action: {:?}", action);
                                match action {
                                    Input::TakeScreenshot => {
                                        let (cap_tx, cap_rx) = oneshot::channel();
                                        let _ = capture_chan_tx.send(cap_tx).await.unwrap();
                                        let result = cap_rx.await.unwrap();
                                        let cv: image::RgbaImage = result.clone().try_into_cv().unwrap();
                                        cv.save("screenshot.png").unwrap();
                                        println!("got screenshot");
                                        let _ = output.send(Event::ResultTakeScreenshot(result));
                                    }
                                }
                            }
                            _ = shutdown.recv_shutdown() => {
                                state = State::Closed;
                            }
                        }
                    }
                    State::Closed => future::pending().await,
                }
            }
        },
    )
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Starting,
    Ready(mpsc::Receiver<Input>),
    Closed,
}
