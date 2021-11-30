use iced::{executor, Application, Command, Element, Subscription};
use iced_video_player::{VideoPlayer, VideoPlayerMessage};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Application {0}")]
    Application(#[from] iced::Error),
}

pub fn handle() {
    App::run(Default::default()).map_err(Error::from)
}

#[derive(Debug)]
enum Message {
    VideoPlayerMessage(VideoPlayerMessage),
}

struct App {
    video: VideoPlayer,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            App {
                video: VideoPlayer::new(&url::Url::parse("file:///C:/my_video.mp4").unwrap())
                    .unwrap(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Video Player")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::VideoPlayerMessage(msg) => self.video.update(msg),
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        self.video.subscription().map(Message::VideoPlayerMessage)
    }

    fn view(&mut self) -> Element<Message> {
        self.video.frame_view().into()
    }
}
