use crate::{
    feature::{CaptureManager, CaptureManagerConfig},
    util::Shutdown,
};
use hd2m_cv::{Direction, TryIntoCv};
use iced::{
    futures::{future, SinkExt},
    subscription, Subscription,
};
use opencv::{self as cv, prelude::*};
use tokio::sync::{mpsc, oneshot};

const TEMPLATE_UP_IMAGE: &'static [u8] = include_bytes!("../resources/up.png");
const TEMPLATE_DOWN_IMAGE: &'static [u8] = include_bytes!("../resources/down.png");
const TEMPLATE_RIGHT_IMAGE: &'static [u8] = include_bytes!("../resources/right.png");
const TEMPLATE_LEFT_IMAGE: &'static [u8] = include_bytes!("../resources/left.png");

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<Input>),
    ResultStratMacro(Vec<Vec<Direction>>),
}

#[derive(Debug)]
pub enum Input {
    RunStratMacro,
}

pub fn capture_process_subscription(shutdown: Shutdown) -> Subscription<Event> {
    struct CaptureSubscription;
    subscription::channel(
        std::any::TypeId::of::<CaptureSubscription>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;

            let mut manager = hd2m_cv::Hd2mCvManager::new(hd2m_cv::Hd2mCvManagerConfig {
                template_up_image: image::load_from_memory_with_format(
                    TEMPLATE_UP_IMAGE,
                    image::ImageFormat::Png,
                )
                .unwrap()
                .to_rgba8(),
                template_down_image: image::load_from_memory_with_format(
                    TEMPLATE_DOWN_IMAGE,
                    image::ImageFormat::Png,
                )
                .unwrap()
                .to_rgba8(),
                template_right_image: image::load_from_memory_with_format(
                    TEMPLATE_RIGHT_IMAGE,
                    image::ImageFormat::Png,
                )
                .unwrap()
                .to_rgba8(),
                template_left_image: image::load_from_memory_with_format(
                    TEMPLATE_LEFT_IMAGE,
                    image::ImageFormat::Png,
                )
                .unwrap()
                .to_rgba8(),
                base_screen_size: (2560, 1440),
                search_options: Some(hd2m_cv::Hd2mCvSearchOptions {
                    threshold: Some(0.9),
                    ..Default::default()
                }),
            })
            .unwrap();

            let (capture_chan_tx, capture_chan_rx) = mpsc::channel(1);
            tokio::spawn({
                let capture_manager = CaptureManager::new(CaptureManagerConfig {
                    window_title: "HELLDIVERS™ 2".to_owned(),
                })
                .unwrap();
                async move {
                    capture_manager.start(capture_chan_rx).await.unwrap();
                    println!("Capture manager done");
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
                                match action {
                                    Input::RunStratMacro => {
                                        tokio::time::sleep(tokio::time::Duration::from_millis(360)).await;

                                        let (cap_tx, cap_rx) = oneshot::channel();
                                        let _ = capture_chan_tx.send(cap_tx).await.unwrap();
                                        let result = cap_rx.await.unwrap();

                                        let size = result.size().unwrap();
                                        let cropped = cv::core::Mat::roi(
                                            &result,
                                            cv::core::Rect::new(
                                                0,
                                                0,
                                                (size.width as f64 * 0.164) as i32,
                                                (size.height as f64 * 0.465) as i32,
                                            ),
                                        ).unwrap().clone_pointee();

                                        manager.use_screen_size(size.width as usize, size.height as usize).unwrap();
                                        let res = manager.run_match_mat(&cropped).unwrap();

                                        // let res = manager.run_match_rgba(&frame.to_rgba8()).unwrap();
                                        println!(
                                            "Res: {:?}",
                                            res.iter()
                                                .map(|e| e.iter().map(|e| e.direction).collect::<Vec<Direction>>())
                                                .collect::<Vec<_>>()
                                        );
                                        // let frame: image::RgbaImage = cropped.clone().try_into_cv().unwrap();
                                        // frame.save("frame.png").unwrap();

                                        let _ = output.send(Event::ResultStratMacro(res.iter()
                                        .map(|e| e.iter().map(|e| e.direction).collect::<Vec<Direction>>())
                                        .collect::<Vec<_>>())).await;
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
