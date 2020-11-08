use iced::{
    Application, Column, Command, Container, Element, HorizontalAlignment,
    Length, Row, Settings, Text, PaneGrid, pane_grid
};

use clap::Clap;
use std::cell::RefCell;
use std::rc::Rc;
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

type WrappedContent = Rc<RefCell<Content>>;

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
    panes: pane_grid::State<WrappedContent>,
    sig_viewer: WrappedContent,
    mod_nav: WrappedContent,
    hier_nav: WrappedContent,
}


enum Content {
    SigView(sigwindow::SigViewer),
    ModNav(module_nav::ModNavigator),
    HierNav(hier_nav::HierNav),

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

impl Content {
    fn update(
        &mut self,
        app_message: Message,
    ) {
        match (self, &app_message) {
            (Content::HierNav(hier_mod), Message::HNMessage(message)) => {
                hier_mod.update(message.clone())
            }
            (Content::SigView(sig_view), Message::SVMessage(message)) => {
                sig_view.update(message.clone())
            }
            (Content::ModNav(module_nav), Message::MNMessage(message)) => {
                module_nav.update(message.clone())
            }
            (_,_) => {
                panic!("Incorrect update message and content")
            }
        }
    }

    fn view(
        &mut self
    ) -> Element<Message> {
        match self {
            Content::HierNav(hier_mod) => {
                hier_mod
                    .view()
                    .map(move |message| Message::HNMessage(message))

            }
            Content::SigView(sig_view) => {
                sig_view
                    .view()
                    .map(move |message| Message::SVMessage(message))

            }
            Content::ModNav(module_nav) => {
                module_nav
                    .view()
                    .map(move |message| Message::MNMessage(message))

            }
        }
    }
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
                        let sig_viewer : WrappedContent = Rc::new(RefCell::new(Content::SigView(sigwindow::SigViewer::default())));
                        let mod_nav = Rc::new(RefCell::new(Content::ModNav(module_nav::ModNavigator::default())));
                        let hier_nav = Rc::new(RefCell::new(Content::HierNav(hier_nav::HierNav::default())));
                        let (mut panes, opane) = pane_grid::State::new(sig_viewer);
                        let mod_pane = panes.split(pane_grid::Axis::Vertical, &opane, mod_nav);
                        panes.split(pane_grid::Axis::Horizontal, &(mod_pane.unwrap().0), hier_nav);


                        *self = Wave2::Loaded(State {
                            panes,
                            sig_viewer,
                            mod_nav,
                            hier_nav
                        });
                    }
                    _ => {}
                }
                Command::none()
            }
            Wave2::Loaded(state) => {
                match message {
                    Message::SVMessage(_) => {
                        state
                            .sig_viewer
                            .borrow_mut()
                            .update(message)
                    },
                    Message::HNMessage(_) => {
                        state
                            .hier_nav
                            .borrow_mut()
                            .update(message)
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
                panes,
                sig_viewer,
                mod_nav,
                hier_nav,
            }) => {
                let pane_grid = PaneGrid::new(&mut panes, |pane, content, focus| {
                let is_focused = focus.is_some();
                let title_bar =
                    pane_grid::TitleBar::new(format!("Focused pane"))
                        .padding(10);

                    pane_grid::Content::new(content.borrow_mut().view())
                    .title_bar(title_bar)





                });
                pane_grid.into()

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
