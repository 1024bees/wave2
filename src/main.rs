use iced::{
    Application, Column, Command, Container, Element, HorizontalAlignment,
    Length, Row, Settings, Text,
};

use clap::Clap;

pub mod components;
use components::{module_nav,sigwindow};
use components::hier_nav::hier_nav;
use env_logger;
use std::path::PathBuf;
use wave2_wavedb::errors::Waverr;
use wave2_wavedb::inout::wave_loader::load_vcd;



#[derive(Clap, Default)]
#[clap(version = "0.0", author = "Jimmy C <jimmy@1024bees.com>")]
#[cfg(not(target_arch = "wasm32"))]
struct Opts {
    #[clap(short, long, default_value = "~/..conf")]
    config: PathBuf,

    #[clap(short, long)]
    wdbpath: Option<PathBuf>,

    #[clap(short, long)]
    vcdpath: Option<PathBuf>,
}

impl Opts {
    async fn load(opt: Opts) -> Result<(), std::io::Error> {
        let Opts {
            config,
            wdbpath,
            vcdpath,

        } = opt;
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
    sig_viewer: sigwindow::SigViewer,
    mod_nav: module_nav::ModNavigator,
    hier_nav: hier_nav::HierNav,
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
    HNMessage(hier_nav::Message),
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
                            mod_nav: module_nav::ModNavigator::default(),
                            hier_nav: hier_nav::HierNav::default(),
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
                    },
                    Message::HNMessage(message) => {
                        state.hier_nav.update(message)
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
            Wave2::Loaded(State {
                sig_viewer,
                mod_nav,
                hier_nav,
            }) => {
                let ww = sig_viewer
                    .view()
                    .map(move |message| Message::SVMessage(message));
                let mod_nav_view = mod_nav
                    .view()
                    .map(move |message| Message::MNMessage(message));
                let hierarchy_view = hier_nav
                    .view()
                    .map(move |message| Message::HNMessage(message));

                let all_content =
                    Column::new().push(Row::new().push(mod_nav_view)).push(ww);
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
