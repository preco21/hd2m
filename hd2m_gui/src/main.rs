use iced::{widget::text, Application, Command, Element, Settings, Subscription};
use tokio::sync::{mpsc, oneshot};

mod capture;
mod feature;
mod util;

#[derive(Default)]
struct App {
    capture_proc_tx: Option<mpsc::Sender<capture::Action>>,
}

#[derive(Debug, Clone)]
enum Message {
    None,
    CaptureProcess(capture::Event),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let app = Self {
            capture_proc_tx: None,
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("External Message Example")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CaptureProcess(event) => match event {
                capture::Event::Ready(sender) => {
                    self.capture_proc_tx = Some(sender);
                }
                capture::Event::ResultTakeScreenshot(mat) => {}
            },
            Message::None => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::Subscription::batch(vec![
            capture::capture_process_subscription().map(Message::CaptureProcess)
        ])
    }

    fn view(&self) -> Element<Message> {
        text(123).into()
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
