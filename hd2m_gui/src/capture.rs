use iced::{futures::SinkExt, subscription, Subscription};
use opencv as cv;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct CaptureProcess {
    id: usize,
    state: State,
}

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
            loop {
                match &mut state {
                    State::Starting => {
                        let (sender, mut receiver) = mpsc::channel(100);
                        let _ = output.send(Event::Ready(sender)).await;
                        state = State::Ready(receiver);
                    }
                    State::Ready(receiver) => {
                        if let Some(action) = receiver.recv().await {
                            match action {
                                Action::TakeScreenshot => {
                                    // let (cap_tx, cap_rx) = oneshot::channel();
                                    // let _ = output.send(Action::TakeScreenshot);
                                    let _ = output.send(Event::ResultTakeScreenshot(
                                        // FIXME: take screenshot
                                        cv::core::Mat::default(),
                                    ));
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
