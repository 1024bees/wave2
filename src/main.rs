use iced::{
    pane_grid, Application, Column, Command, Container, Element,
    HorizontalAlignment, Length, PaneGrid, Row, Settings, Text,
};

use clap::Clap;
use std::sync::Arc;
pub mod components;
mod config;
use components::hier_nav::hier_nav;
use components::{menu_bar, module_nav, sigwindow};
use config::menu_update;
use env_logger;
use log::{info, warn};
use std::path::PathBuf;
use wave2_wavedb::api::WdbAPI;
use wave2_wavedb::errors::Waverr;
use wave2_wavedb::inout::wave_loader::load_vcd;
use wave2_wavedb::wavedb::WaveDB;

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

pub struct State {
    panes: pane_grid::State<Content>,
    sv_pane: pane_grid::Pane,
    mn_pane: pane_grid::Pane,
    hn_pane: pane_grid::Pane,
    menu_bar: menu_bar::MenuBar,
    wdb_api: Option<Arc<WdbAPI>>,
}

impl State {
    fn set_file_pending(&mut self, pending: bool) {
        self.menu_bar.set_pending_file(pending);
    }

    fn open_file_pending(&mut self) -> bool {
        self.menu_bar.get_pending_file()
    }
}

// This may be bad design, but I've resigned for the pane_grid::State
// to own all Component level state.
//
// In the future it will be better to have content own the iced widget state and have an
// Rc<RefCell<>> wrapping application state
///
/// Component level content;
/// Content wraps each component level state, and can be referenced via get_mut() using each pane
/// as a key
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
pub enum PaneMessage {
    //Dragged(pane_grid::DragEvent),
    Resize(pane_grid::ResizeEvent),
}

#[derive(Debug)]
pub enum Message {
    // Component messages
    SVMessage(sigwindow::Message),
    MNMessage(module_nav::Message),
    HNMessage(hier_nav::Message),
    MBMessage(menu_bar::Message),
    //IoMessage
    Loaded(Result<(), std::io::Error>),
    LoadWDB(Result<Arc<WdbAPI>, Waverr>),
    //Pane Messages
    PaneMessage(PaneMessage),
}

impl Content {
    fn update(&mut self, app_message: Message) {
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
            (_, _) => panic!("Incorrect update message and content"),
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Content::HierNav(hier_mod) => hier_mod
                .view()
                .map(move |message| Message::HNMessage(message)),
            Content::SigView(sig_view) => sig_view
                .view()
                .map(move |message| Message::SVMessage(message)),
            Content::ModNav(module_nav) => module_nav
                .view()
                .map(move |message| Message::MNMessage(message)),
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
                        let sig_viewer =
                            Content::SigView(sigwindow::SigViewer::default());
                        let mod_nav = Content::ModNav(
                            module_nav::ModNavigator::default(),
                        );
                        let hier_nav =
                            Content::HierNav(hier_nav::HierNav::default());
                        let (mut panes, sv_pane) =
                            pane_grid::State::new(sig_viewer);
                        let (mn_pane, _) = panes
                            .split(pane_grid::Axis::Vertical, &sv_pane, mod_nav)
                            .unwrap();
                        panes.swap(&mn_pane, &sv_pane);
                        let (hn_pane, _) = panes
                            .split(
                                pane_grid::Axis::Horizontal,
                                &mn_pane,
                                hier_nav,
                            )
                            .unwrap();
                        //TODO: do some like uhhh... cleaning up here
                        //      should probably initialize sizes of panes, etc

                        let menu_bar = menu_bar::MenuBar::default();
                        *self = Wave2::Loaded(State {
                            panes,
                            sv_pane,
                            mn_pane,
                            hn_pane,
                            menu_bar,
                            wdb_api: None,
                        });
                    }
                    _ => {}
                }
                Command::none()
            }
            Wave2::Loaded(state) => {
                match message {
                    Message::MBMessage(menu_message) => {
                        return menu_update(state, menu_message)
                    }
                    Message::SVMessage(_) => state
                        .panes
                        .get_mut(&state.sv_pane)
                        .unwrap()
                        .update(message),
                    Message::HNMessage(_) => state
                        .panes
                        .get_mut(&state.hn_pane)
                        .unwrap()
                        .update(message),
                    Message::MNMessage(_) => state
                        .panes
                        .get_mut(&state.mn_pane)
                        .unwrap()
                        .update(message),

                    Message::LoadWDB(payload) => match payload {
                        Ok(wdb_api) => {
                            state.wdb_api = Some(wdb_api);
                            state.set_file_pending(false);
                            info!("Loaded message successfully");
                            state
                                .panes
                                .get_mut(&state.hn_pane)
                                .unwrap()
                                .update(Message::HNMessage(
                                    hier_nav::Message::SetHier(
                                        state.wdb_api.as_ref().unwrap().get_hier_map().clone(),
                                    ),
                                ));
                        }
                        Err(waverr) => {
                            state.set_file_pending(false);
                            warn!(
                                "{}",
                                format!("VCD not loaded! err is {:?}", waverr)
                            )
                        }
                    },
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
                panes, menu_bar, ..
            }) => {
                //all_content.into()
                let pane_grid = PaneGrid::new(panes, |pane, content, focus| {
                    let is_focused = focus.is_some();
                    let title_bar =
                        pane_grid::TitleBar::new(format!("Focused pane"))
                            .padding(10);

                    pane_grid::Content::new(content.view()).title_bar(title_bar)
                })
                .width(Length::Fill)
                .height(Length::Fill)
                //FIXME: causes int overflow in the glow backend
                //.on_drag(|pane_data| Message::PaneMessage(PaneMessage::Dragged(pane_data)))
                .on_resize(10, |resize_data| {
                    Message::PaneMessage(PaneMessage::Resize(resize_data))
                });

                let menu_bar_view =
                    menu_bar.view().map(|message| Message::MBMessage(message));

                Row::new().push(menu_bar_view).push(pane_grid).into()
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
