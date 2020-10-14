use iced::{
    Application, Command, Container, Element,
    HorizontalAlignment, Length, Settings, Text, Column, Row
};

use clap::Clap;

pub mod components;
use components::*;
use std::path::PathBuf;
use env_logger;

#[derive(Clap, Default)]
#[clap(version = "0.0", author = "Jimmy C <jimmy@1024bees.com>")]
#[cfg(not(target_arch = "wasm32"))]
struct Opts {
    #[clap(short, long, default_value = "~/..conf")]
    config: PathBuf,

    #[clap(short, long)]
    wdbpath: Option<String>,

    #[clap(short, long)]
    vcdpath: Option<String>,
}

impl Opts {
    async fn load(opt: Opts) -> Result<(), std::io::Error> {
        //TODO: Impl me
        Ok(())
    }
}



#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let opts: Opts = Clap::parse();
    let mut settings: Settings<Opts> = Settings {
        flags: opts,
        ..Settings::default()
    };
    env_logger::init();
    Wave2::run(settings);
}

struct State {
    sig_viewer : sigwindow::SigViewer,
    mod_nav : module_nav::ModNavigator,
}

enum Wave2 {
    Loading,
    Loaded(State),
}

#[derive(Debug)]
enum Message {
    // Component messages
    SVMessage(sigwindow::Message),
    MNMessage(module_nav::Message),
    //IoMessage
    Loaded(Result<(), std::io::Error>),
}

impl Application for Wave2 {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = Opts;

    fn new(flags: Opts) -> (Wave2, Command<Self::Message>) {
        (
            Wave2::Loading,
            Command::perform(Opts::load(flags), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("Wave2")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            Wave2::Loading => {
                match message {
                    Message::Loaded(Ok(void)) => {
                        *self = Wave2::Loaded(State {
                            sig_viewer: sigwindow::SigViewer::default(),
                            mod_nav : module_nav::ModNavigator::default(),

                        });
                    }
                    _ => {}
                }
                Command::none()
            }
            Wave2::Loaded(state) => {
                match message {
                    Message::SVMessage(message) => {
                        state.sig_viewer.update(message)
                    }
                    _ => {}
                }
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        match self {
            Wave2::Loading => loading_message(),
            Wave2::Loaded(State { sig_viewer, mod_nav }) => {
                let ww = sig_viewer
                    .view()
                    .map(move |message| Message::SVMessage(message));
                let mod_nav_view = mod_nav
                    .view()
                    .map(move |message| Message::MNMessage(message));
                let all_content = Column::new()
                    .push(
                        Row::new()
                        .push(mod_nav_view))
                    .push(ww);
                all_content.into()

            }
        }
    }
}

fn loading_message() -> Element<'static, Message> {
    Container::new(
        Text::new("Loading...")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .center_x()
    .into()
}
