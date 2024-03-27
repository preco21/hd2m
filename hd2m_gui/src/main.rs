use iced::{
    widget::{button, column, text, Column},
    window, Application, Command, Element, Settings, Subscription,
};
use tokio::sync::{mpsc, oneshot};
use util::Shutdown;

mod capture;
mod feature;
mod input_manager;
mod util;

#[derive(Debug)]
struct App {
    is_macro_active: bool,
    input_manager_tx: Option<mpsc::Sender<input_manager::Input>>,
    capture_tx: Option<mpsc::Sender<capture::Input>>,
    shutdown_token: Shutdown,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

#[derive(Debug, Clone)]
enum Message {
    IcedEvent(iced::Event),
    HandleInputManagerEvent(input_manager::Event),
    HandleCaptureEvent(capture::Event),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let (shutdown_complete_tx, _shutdown_complete_rx) = mpsc::channel(1);
        let shutdown_token = Shutdown::new(shutdown_tx, shutdown_complete_tx.clone());

        let app = Self {
            is_macro_active: false,
            input_manager_tx: None,
            capture_tx: None,
            shutdown_token,
            shutdown_rx: Some(shutdown_rx),
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("HD2M by preco21")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(event) => match event {
                iced::Event::Window(id, event) => match event {
                    iced::window::Event::CloseRequested => {
                        if let Some(shutdown_rx) = self.shutdown_rx.take() {
                            // Notify the shutdown token that the application is shutting down
                            drop(shutdown_rx);
                            return window::close(id);
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Message::HandleCaptureEvent(event) => match event {
                capture::Event::Ready(sender) => {
                    self.capture_tx = Some(sender);
                    println!("Capture ready");
                }
                capture::Event::ResultTakeScreenshot(mat) => {
                    println!("Got screenshot");
                }
            },
            Message::HandleInputManagerEvent(event) => match event {
                input_manager::Event::Ready(sender) => {
                    self.input_manager_tx = Some(sender);
                }
                input_manager::Event::ToggleStratMacro(flag) => {
                    self.is_macro_active = flag;
                    if flag {
                        if let Some(capture) = &self.capture_tx {
                            let _ = capture.send(capture::Input::TakeScreenshot);
                        }
                    }
                }
            },
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::event::listen().map(Message::IcedEvent),
            input_manager::input_manager_subscription(self.shutdown_token.clone())
                .map(Message::HandleInputManagerEvent),
            capture::capture_process_subscription(self.shutdown_token.clone())
                .map(Message::HandleCaptureEvent),
        ])
    }

    fn view(&self) -> Element<Message> {
        column![
            text("Muahahaha :)"),
            text("You should register keybindings for this program to work: W, A, S, D for stratagem commands"),
            text("Use \"Mouse 5\" button to activate the macro then use 1..9 keys to instantly run the stratagem"),
            text(format!("Macro is currently: {}", if self.is_macro_active { "Active" } else { "Inactive" })),
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
            size: iced::Size::new(300.0, 300.0),
            resizable: false,
            exit_on_close_request: false,
            ..Default::default()
        },
        ..Default::default()
    })
}
