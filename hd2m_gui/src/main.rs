use hd2m_cv::cv_convert::*;
use hd2m_cv::screen_capture::{CaptureManager, CaptureManagerConfig};
use iced::{widget::text, Application, Command, Element, Settings, Subscription};
use image::RgbaImage;
use opencv::{self as cv};
use std::cell::RefCell;
use tokio::sync::{mpsc, oneshot};

mod capture;
mod message;
mod shutdown;

#[derive(Default)]
struct App {
    capture_proc_tx: Option<mpsc::Sender<capture::Action>>,
}

#[derive(Debug, Clone)]
enum Message {
    None,
    ExternalMessageReceived(i32),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;

    fn new(flags: AppFlags) -> (Self, Command<Message>) {
        let (tick_tx, mut tick_rx) = mpsc::channel(1);
        let app = App {
            receiver: RefCell::new(Some(flags.receiver)),
            tick_tx,
            capture_trigger: mpsc::channel(1),
            num: 0,
        };
        (
            app,
            Command::perform(
                async move {
                    let (capture_chan_tx, capture_chan_rx) = mpsc::channel(1);

                    let mgr = CaptureManager::new(CaptureManagerConfig {
                        window_title: "Code".to_owned(),
                    })
                    .unwrap();

                    tokio::spawn(async move {
                        mgr.start(capture_chan_rx).await.unwrap();
                    });

                    tokio::spawn(async move {
                        loop {
                            tick_rx.recv().await;
                            let now = std::time::Instant::now();
                            let (cap_tx, cap_rx) = oneshot::channel();
                            let _ = capture_chan_tx.send(cap_tx).await;
                            let result = cap_rx.await.unwrap();
                            let cv: RgbaImage = result.try_into_cv().unwrap();
                            println!("screenshot took: {:?}", now.elapsed());
                            cv.save("screenshot.png").unwrap();
                        }
                    });
                },
                |_| Message::None,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("External Message Example")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ExternalMessageReceived(num) => {
                self.num = num;
            }
            Message::None => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::unfold(
            "led changes",
            (self.receiver.take(), self.tick_tx.clone()),
            move |(mut receiver, tick_tx)| async move {
                let num = receiver.unwrap().recv().await.unwrap();
                tick_tx.send(()).await.unwrap();
                (Message::ExternalMessageReceived(num), (receiver, tick_tx))
            },
        )

        // iced::subscription::unfold("screen capture", 100, move |mut output| async move {
        //     let (tick_tx, tick_rx) = mpsc::channel(1);

        //     let (capture_chan_tx, capture_chan_rx) = mpsc::channel(1);

        //     let mgr = CaptureManager::new(CaptureManagerConfig {
        //         window_title: "Code".to_owned(),
        //     })
        //     .unwrap();

        //     tokio::spawn(async move {
        //         mgr.start(capture_chan_rx).await.unwrap();
        //     });

        //     loop {
        //         tick_rx.recv().await;
        //         let (cap_tx, cap_rx) = oneshot::channel();
        //         let _ = capture_chan_tx.send(cap_tx).await;
        //         let result = cap_rx.await.unwrap();
        //         let cv: RgbaImage = result.try_into_cv().unwrap();
        //         cv.save("screenshot.png").unwrap();
        //     }
        // })
    }

    fn view(&self) -> Element<Message> {
        text(self.num).into()
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
