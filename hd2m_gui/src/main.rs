use iced::{
    widget::{button, column, text, Column},
    Application, Command, Element, Settings, Subscription,
};
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
        String::from("HD2M by preco21")
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
        column![
            text("Muahahaha :)"),
            text("You should register keybindings for this program to work: W, A, S, D for stratagem commands"),
            text("Use \"Mouse 5\" button to activate the macro then use 1..9 keys to instantly run the stratagem"),
        ]
        .padding(20)
        .spacing(20)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }
}

fn main() -> iced::Result {
    App::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(300.0, 250.0),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })
}
