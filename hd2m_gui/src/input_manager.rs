use std::time::Duration;

use enigo::*;
use hd2m_cv::Direction;
use iced::futures::{future, SinkExt};
use tokio::sync::mpsc;

use crate::util::Shutdown;

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<Input>),
    ToggleStratMacro(bool),
    UseStratKey(usize),
}

#[derive(Debug)]
pub enum Input {
    SendDirectionCommand(Vec<Direction>),
}

pub fn input_manager_subscription(shutdown: Shutdown) -> iced::Subscription<Event> {
    struct InputManagerSubscription;
    iced::subscription::channel(
        std::any::TypeId::of::<InputManagerSubscription>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;

            let mut enigo = Enigo::new();

            // Listen for mouse button presses
            let (trigger_evt_tx, mut trigger_evt_rx) = mpsc::channel::<rdev::EventType>(1);
            std::thread::spawn(move || {
                let callback = move |event: rdev::Event| {
                    let _ = trigger_evt_tx.try_send(event.event_type);
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
                            Some(event_type) = trigger_evt_rx.recv() => {
                                match event_type {
                                    rdev::EventType::ButtonPress(button) => match button {
                                        rdev::Button::Unknown(2) => {
                                            let _ = output.send(Event::ToggleStratMacro(true)).await;
                                        }
                                        _ => {}
                                    },
                                    rdev::EventType::ButtonRelease(button) => match button {
                                        rdev::Button::Unknown(2) => {
                                            let _ = output.send(Event::ToggleStratMacro(false)).await;
                                        }
                                        _ => {}
                                    },
                                    rdev::EventType::KeyPress(key) => {
                                        match key {
                                            rdev::Key::Num1 => {
                                                let _ = output.send(Event::UseStratKey(1)).await;
                                            }
                                            rdev::Key::Num2 => {
                                                let _ = output.send(Event::UseStratKey(2)).await;
                                            }
                                            rdev::Key::Num3 => {
                                                let _ = output.send(Event::UseStratKey(3)).await;
                                            }
                                            rdev::Key::Num4 => {
                                                let _ = output.send(Event::UseStratKey(4)).await;
                                            }
                                            rdev::Key::Num5 => {
                                                let _ = output.send(Event::UseStratKey(5)).await;
                                            }
                                            rdev::Key::Num6 => {
                                                let _ = output.send(Event::UseStratKey(6)).await;
                                            }
                                            rdev::Key::Num7 => {
                                                let _ = output.send(Event::UseStratKey(7)).await;
                                            }
                                            rdev::Key::Num8 => {
                                                let _ = output.send(Event::UseStratKey(8)).await;
                                            }
                                            rdev::Key::Num9 => {
                                                let _ = output.send(Event::UseStratKey(9)).await;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Some(action) = receiver.recv() => {
                                inputbot::KeybdKey::DKey.press();
                                inputbot::KeybdKey::DKey.release();
                                match action {
                                    Input::SendDirectionCommand(directions) => {
                                        println!("Sending directions: {:?}", directions);
                                        for dir in directions {
                                            match dir {
                                                // Direction::Up => send(&rdev::EventType::KeyPress(rdev::Key::KeyW)).await,
                                                // Direction::Down => send(&rdev::EventType::KeyPress(rdev::Key::KeyS)).await,
                                                // Direction::Left => send(&rdev::EventType::KeyPress(rdev::Key::KeyA)).await,
                                                // Direction::Right => send(&rdev::EventType::KeyPress(rdev::Key::KeyD)).await,
                                                Direction::Up => {
                                                    inputbot::KeybdKey::WKey.release();
                                                    tokio::time::sleep(Duration::from_millis(50)).await;
                                                    inputbot::KeybdKey::WKey.press();
                                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                                    inputbot::KeybdKey::WKey.release();
                                                },
                                                Direction::Down => {
                                                    inputbot::KeybdKey::SKey.release();
                                                    tokio::time::sleep(Duration::from_millis(50)).await;
                                                    inputbot::KeybdKey::SKey.press();
                                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                                    inputbot::KeybdKey::SKey.release();


                                                },
                                                Direction::Left => {
                                                    inputbot::KeybdKey::AKey.release();
                                                    tokio::time::sleep(Duration::from_millis(50)).await;
                                                    inputbot::KeybdKey::AKey.press();
                                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                                    inputbot::KeybdKey::AKey.release();

                                                },
                                                Direction::Right => {
                                                    inputbot::KeybdKey::DKey.release();
                                                    tokio::time::sleep(Duration::from_millis(50)).await;
                                                    inputbot::KeybdKey::DKey.press();
                                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                                    inputbot::KeybdKey::DKey.release();
                                                },
                                            }
                                        }
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

// async fn send(event_type: &rdev::EventType) {
//     let delay = tokio::time::Duration::from_millis(20);
//     let event_type = event_type.clone();
//     std::thread::spawn(move || match rdev::simulate(&event_type) {
//         Ok(()) => (),
//         Err(SimulateError) => {
//             println!("We could not send {:?}", event_type);
//             println!("error {:?}", SimulateError);
//         }
//     });
//     // Let ths OS catchup (at least MacOS)
//     tokio::time::sleep(delay).await;
// }
