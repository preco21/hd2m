use hd2m_cv::Direction;
use iced::futures::{future, SinkExt};
use tokio::sync::mpsc;

use crate::util::Shutdown;

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<Input>),
    ToggleStratMacro(bool),
}

#[derive(Debug)]
pub enum Input {
    SendCommand(Vec<Direction>),
}

pub fn input_manager_subscription(shutdown: Shutdown) -> iced::Subscription<Event> {
    struct InputManagerSubscription;
    iced::subscription::channel(
        std::any::TypeId::of::<InputManagerSubscription>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;

            // Listen for mouse button presses
            let (trigger_evt_tx, mut trigger_evt_rx) = mpsc::channel::<bool>(1);
            std::thread::spawn(move || {
                let callback = move |event: rdev::Event| match event.event_type {
                    rdev::EventType::ButtonPress(button) => match button {
                        rdev::Button::Unknown(2) => {
                            let _ = trigger_evt_tx.try_send(true);
                        }
                        _ => {}
                    },
                    rdev::EventType::ButtonRelease(button) => match button {
                        rdev::Button::Unknown(2) => {
                            let _ = trigger_evt_tx.try_send(false);
                        }
                        _ => {}
                    },
                    _ => {}
                };
                if let Err(error) = rdev::listen(callback) {
                    println!("Error: {:?}", error)
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
                            Some(flag) = trigger_evt_rx.recv() => {
                                let _ = output.send(Event::ToggleStratMacro(flag)).await;
                            }
                            Some(action) = receiver.recv() => {
                                match action {
                                    Input::SendCommand(directions) => {
                                        println!("{:?}", directions);
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
