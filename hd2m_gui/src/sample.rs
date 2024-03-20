// https://discourse.iced.rs/t/questions-about-subscriptions/91/5

use std::fmt::Display;

use connection::InputMessage;
use iced::{
    executor,
    futures::channel::mpsc,
    widget::{button, column, container, progress_bar, row, text},
    Application, Command, Element, Length, Renderer, Settings, Subscription, Theme,
};

#[derive(Default)]
struct App {
    sender: Option<mpsc::Sender<InputMessage>>,
    state: ConnectionState,
    result: Option<String>,
    progress: Option<u32>,
}

#[derive(Debug, Default)]
enum ConnectionState {
    #[default]
    Pending,
    Connected,
    Disconnected,
}

impl ConnectionState {
    fn is_connected(&self) -> bool {
        match self {
            ConnectionState::Connected => true,
            _ => false,
        }
    }
}

impl Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Pending => write!(f, "Pending"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Disconnected => write!(f, "Disconnected"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Connect,
    Disconnect,
    GetParams,
    Cancel,
    ConnectionEvent(connection::Event),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Iced Mavlink Connection Question".to_string()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connect => self.send_message_to_subscription(InputMessage::Connect),
            Message::Disconnect => self.send_message_to_subscription(InputMessage::Disconnect),
            Message::GetParams => self.send_message_to_subscription(InputMessage::GetParameters),
            Message::Cancel => self.send_message_to_subscription(InputMessage::Cancel),

            Message::ConnectionEvent(event) => match event {
                connection::Event::ConnectionPending(sender) => {
                    self.state = ConnectionState::Disconnected;
                    self.sender = Some(sender);
                }
                connection::Event::Connected => {
                    self.state = ConnectionState::Connected;
                    println!("Connection started");
                }
                connection::Event::Disconnected => {
                    self.state = ConnectionState::Disconnected;
                    println!("Connection closed");
                }
                connection::Event::Progress(progress) => self.progress = Some(progress),
                connection::Event::Result(value) => self.result = Some(value),
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Message, Renderer<Theme>> {
        let controls = {
            let connect = button("Connect")
                .on_press_maybe((!self.state.is_connected()).then(|| Message::Connect));
            let disconnect = button("Disconnect")
                .on_press_maybe(self.state.is_connected().then(|| Message::Disconnect));

            let get_params = button("Get params")
                .on_press_maybe(self.state.is_connected().then(|| Message::GetParams));

            let cancel = button("Cancel").on_press(Message::Cancel);

            row![connect, disconnect, get_params, cancel].spacing(10)
        };

        let status = {
            let state = text(self.state.to_string());
            let progress = progress_bar(0.0..=9.0, self.progress.unwrap_or(0) as f32).width(200);
            let result = text(self.result.as_ref().unwrap_or(&"None yet".to_string()));
            row![state, progress, result].spacing(10)
        };

        container(column![controls, status].spacing(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        connection::connection().map(Message::ConnectionEvent)
    }
}

impl App {
    fn send_message_to_subscription(&mut self, message: InputMessage) {
        self.sender.as_mut().unwrap().try_send(message).unwrap()
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}

mod connection {

    use std::{sync::Arc, thread, time::Duration};

    use iced::{
        futures::{channel::mpsc, SinkExt, StreamExt},
        subscription, Subscription,
    };
    use tokio::time::timeout;

    type Connection = Arc<String>;

    enum State {
        Disconnected,
        Connected { connection: Connection },
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        ConnectionPending(mpsc::Sender<InputMessage>),
        Connected,
        Disconnected,
        Progress(u32),
        Result(String),
    }

    pub enum InputMessage {
        Connect,
        GetParameters,
        Cancel,
        Disconnect,
        None,
    }

    pub fn connection() -> Subscription<Event> {
        struct Connect;
        subscription::channel(
            std::any::TypeId::of::<Connect>(),
            100,
            |mut output| async move {
                // Start the subscription future and return the sender to send messages into the subscription back to the application
                let (sender, mut receiver) = mpsc::channel(100);
                let _ = output.send(Event::ConnectionPending(sender)).await;

                let mut state = State::Disconnected;

                loop {
                    match &mut state {
                        State::Disconnected => {
                            if let InputMessage::Connect = receiver.select_next_some().await {
                                state = State::Connected {
                                    connection: Arc::new("Connection".to_string()),
                                };
                                let _ = output.send(Event::Connected).await;
                            }
                        }

                        State::Connected { connection } => {
                            match receive_input_message(&mut receiver).await {
                                InputMessage::Connect => panic!("Already connected"),
                                InputMessage::Disconnect => {
                                    state = State::Disconnected;
                                    let _ = output.send(Event::Disconnected).await;
                                }
                                InputMessage::GetParameters => {
                                    let result =
                                        get_parameters(connection.clone(), output.clone()).await;
                                    let _ = output.send(Event::Result(result)).await;
                                }
                                InputMessage::None => {
                                    // Here I poll the connection and report status back to application
                                }
                                InputMessage::Cancel => {
                                    // How do a cancel the `get_parameters` task?
                                }
                            }
                        }
                    }
                }
            },
        )
    }

    async fn receive_input_message(rx: &mut mpsc::Receiver<InputMessage>) -> InputMessage {
        // Try to receive a message. If there isn't one, every second default to a `None` message which triggers polling the connection
        timeout(Duration::from_secs(1), rx.select_next_some())
            .await
            .unwrap_or(InputMessage::None)
    }

    async fn get_parameters(connection: Connection, mut output: mpsc::Sender<Event>) -> String {
        tokio::task::spawn_blocking({
            let conn = connection.clone();
            move || {
                for i in 0..10 {
                    println!("Using connection {}", conn);
                    thread::sleep(Duration::from_secs(1));
                    let _ = output.send(Event::Progress(i)); // Cannot await here because the block is not async
                }
                "Result that took a long time to download or compute or whatever".to_string()
            }
        })
        .await
        .unwrap()
    }
}
