use iced::{
    pane_grid, Application, Clipboard, Column, Command, Container, Element, HorizontalAlignment,
    Length, PaneGrid, Settings, Text,
};

use clap::Clap;
use std::sync::Arc;
pub mod components;
mod config;
use components::hier_nav::hier_nav;
use components::icon_bar;
use components::signals::wavewindow;
use components::{
    menu_bar, module_nav,
    signals::{self, sigwindow},
    style,
};
use config::menu_update;
use env_logger;
use log::warn;
use std::path::PathBuf;
use wave2_wavedb::api::WdbApi;
use wave2_wavedb::errors::Waverr;
use wave2_wavedb::inout::wave_loader::load_vcd_from_path;

#[derive(Clap, Default)]
#[clap(version = "0.0", author = "Jimmy C <jimmy@1024bees.com>")]
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
struct Opts {
    #[clap(short, long, default_value = "~/..conf")]
    config: PathBuf,

    #[clap(short, long)]
    wdbpath: Option<PathBuf>,

    #[clap(short, long)]
    vcdpath: Option<PathBuf>,
}

impl Opts {
    fn load(opt: Opts) -> (Wave2, Command<Message>) {
        match opt {
            Opts {
                vcdpath: Some(path),
                ..
            } => (
                Wave2::Loading,
                Command::perform(
                    async { Ok(load_vcd_from_path(path).await) },
                    Message::Loaded,
                ),
            ),
            _ => (
                Wave2::Loading,
                Command::perform(async { Ok(None) }, Message::Loaded),
            ),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let opts: Opts = Clap::parse();
    let settings: Settings<Opts> = Settings {
        flags: opts,
        ..Settings::default()
    };
    env_logger::init();
    Wave2::run(settings).expect("Fatal error during initialization");
}

pub struct State {
    panes: pane_grid::State<Content>,
    sv_pane: pane_grid::Pane,
    mn_pane: pane_grid::Pane,
    hn_pane: pane_grid::Pane,
    ww_pane: pane_grid::Pane,
    focused_pane: Option<pane_grid::Pane>,
    menu_bar: menu_bar::GlobalMenuBar,
    icon_bar: icon_bar::IconBar,
    wdb_api: Option<Arc<WdbApi>>,
}

impl State {
    fn set_file_pending(&mut self, pending: bool) {
        self.menu_bar.set_pending_file(pending);
    }

    fn open_file_pending(&mut self) -> bool {
        self.menu_bar.get_pending_file()
    }

    fn get_api(&self) -> Arc<WdbApi> {
        self.wdb_api.as_ref().unwrap().clone()
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
    WaveWindow(wavewindow::WaveWindowState),
}

impl ToString for Content {
    fn to_string(&self) -> String {
        match self {
            Content::SigView(_) => String::from("Signal viewer"),
            Content::ModNav(_) => String::from("Signal navigator"),
            Content::HierNav(_) => String::from("Hierarchy navigator"),
            Content::WaveWindow(_) => String::from("Waves"),
        }
    }
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
    MNMessage(module_nav::Message),
    HNMessage(hier_nav::Message),
    SignalsMessage(signals::Message),
    MBMessage(menu_bar::Message),
    IBMessage(signals::Message),
    //IoMessage
    Loaded(Result<Option<Arc<WdbApi>>, std::io::Error>),
    LoadWDB(Result<Arc<WdbApi>, Waverr>),
    //Pane Messages
    PaneMessage(PaneMessage),
}

impl Content {
    fn update(&mut self, app_message: Message) {
        match (self, &app_message) {
            (Content::HierNav(hier_mod), Message::HNMessage(message)) => {
                hier_mod.update(message.clone())
            }
            (Content::SigView(sig_view), Message::SignalsMessage(message)) => {
                sig_view.update(message.clone())
            }
            (Content::WaveWindow(wavewindow), Message::SignalsMessage(message)) => {
                wavewindow.update(message.clone())
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
                .map(move |message| Message::SignalsMessage(message)),
            Content::ModNav(module_nav) => module_nav
                .view()
                .map(move |message| Message::MNMessage(message)),
            Content::WaveWindow(ww) => ww
                .view()
                .map(move |message| Message::SignalsMessage(message)),
        }
    }
}

impl Application for Wave2 {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = Opts;

    fn new(flags: Opts) -> (Wave2, Command<Self::Message>) {
        Opts::load(flags)
    }

    fn title(&self) -> String {
        String::from("Wave2")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        fn update_signals_logic(state: &mut State, inner_message: signals::Message) {
            state
                .panes
                .get_mut(&state.sv_pane)
                .unwrap()
                .update(Message::SignalsMessage(inner_message.clone()));
            state
                .panes
                .get_mut(&state.ww_pane)
                .unwrap()
                .update(Message::SignalsMessage(inner_message.clone()));
        }

        match self {
            Wave2::Loading => {
                match message {
                    Message::Loaded(Ok(wavedb)) => {
                        let sig_viewer = Content::SigView(sigwindow::SigViewer::default());
                        let mod_nav = Content::ModNav(module_nav::ModNavigator::default());
                        let hier_nav = Content::HierNav(hier_nav::HierNav::default());
                        let wavewindow =
                            Content::WaveWindow(wavewindow::WaveWindowState::default());
                        let (mut panes, sv_pane) = pane_grid::State::new(sig_viewer);

                        let (mn_pane, split) = panes
                            .split(pane_grid::Axis::Vertical, &sv_pane, mod_nav)
                            .unwrap();
                        panes.swap(&mn_pane, &sv_pane);
                        panes.resize(&split, 0.2);
                        let (ww_pane, ww_split) = panes
                            .split(pane_grid::Axis::Vertical, &sv_pane, wavewindow)
                            .unwrap();
                        panes.resize(&ww_split, 0.3);
                        let (hn_pane, _) = panes
                            .split(pane_grid::Axis::Horizontal, &mn_pane, hier_nav)
                            .unwrap();
                        panes.swap(&hn_pane, &mn_pane);
                        //TODO: do some like uhhh... cleaning up here
                        //      should probably initialize sizes of panes, etc

                        let menu_bar = menu_bar::GlobalMenuBar::default();
                        let icon_bar = icon_bar::IconBar::default();
                        *self = Wave2::Loaded(State {
                            panes,
                            sv_pane,
                            mn_pane,
                            hn_pane,
                            ww_pane,
                            menu_bar,
                            icon_bar,
                            focused_pane: None,
                            wdb_api: None,
                        });
                        if wavedb.is_some() {
                            Command::perform(async move { Ok(wavedb.unwrap()) }, Message::LoadWDB)
                        } else {
                            Command::none()
                        }
                    }
                    _ => Command::none(),
                }
            }
            Wave2::Loaded(state) => {
                match message {
                    Message::MBMessage(menu_message) => return menu_update(state, menu_message),
                    Message::SignalsMessage(inner_message) => {
                        state.focused_pane = Some(state.sv_pane);
                        update_signals_logic(state, inner_message);
                    }
                    Message::HNMessage(hn_message) => {
                        match hn_message {
                            hier_nav::Message::SendModule(module_idx) => {
                                // we have to process this message within HierNav
                                state
                                    .panes
                                    .get_mut(&state.hn_pane)
                                    .unwrap()
                                    .update(Message::HNMessage(hn_message.clone()));
                                return Command::perform(
                                    WdbApi::get_module_signals(state.get_api(), module_idx),
                                    move |vector| {
                                        Message::MNMessage(module_nav::Message::SignalUpdate(
                                            vector,
                                        ))
                                    },
                                );
                            }
                            _ => {
                                state.focused_pane = Some(state.hn_pane);
                                state
                                    .panes
                                    .get_mut(&state.hn_pane)
                                    .unwrap()
                                    .update(Message::HNMessage(hn_message))
                            }
                        }
                    }
                    Message::IBMessage(ib_message) => match ib_message {
                        signals::Message::TIUpdate(..) | signals::Message::BoundsUpdate(_) => {
                            state.icon_bar.update(ib_message)
                        }
                        _ => {
                            update_signals_logic(state, ib_message);
                        }
                    },
                    Message::MNMessage(mn_message) => match mn_message {
                        module_nav::Message::AddSig(signal_item) => {
                            return Command::perform(
                                WdbApi::get_signal(state.get_api(), signal_item),
                                move |wave| {
                                    Message::SignalsMessage(signals::Message::AddWave(wave))
                                },
                            );
                        }

                        _ => {
                            state.focused_pane = Some(state.mn_pane);
                            state
                                .panes
                                .get_mut(&state.mn_pane)
                                .unwrap()
                                .update(Message::MNMessage(mn_message))
                        }
                    },
                    Message::LoadWDB(payload) => match payload {
                        Ok(wdb_api) => {
                            state.wdb_api = Some(wdb_api);
                            state.set_file_pending(false);

                            state.panes.get_mut(&state.hn_pane).unwrap().update(
                                Message::HNMessage(hier_nav::Message::SetHier(
                                    state.wdb_api.as_ref().unwrap().get_hier_map().clone(),
                                )),
                            );
                            return Command::perform(
                                WdbApi::bounds(state.get_api()),
                                move |bounds| {
                                    Message::SignalsMessage(signals::Message::UpdateBounds(bounds))
                                },
                            );
                        }
                        Err(waverr) => {
                            state.set_file_pending(false);
                            warn!("{}", format!("VCD not loaded! err is {:?}", waverr))
                        }
                    },
                    Message::PaneMessage(pane_message) => match pane_message {
                        PaneMessage::Resize(pane_grid::ResizeEvent { split, ratio }) => {
                            state.panes.resize(&split, ratio);
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
                panes,
                menu_bar,
                focused_pane,
                icon_bar,
                ..
            }) => {
                //all_content.into()

                let pane_grid = PaneGrid::new(panes, |pane, content| {
                    //let title_bar = pane_grid::TitleBar::new(Text::new(format!("Focused pane"))).padding(3);
                    let is_focused = focused_pane
                        .as_ref()
                        .map(|focused| focused.clone() == pane)
                        .unwrap_or(false);

                    pane_grid::Content::new(content.view()).style(style::Pane { is_focused })
                    //.title_bar(title_bar)
                })
                .width(Length::Fill)
                .height(Length::Fill)
                //FIXME: causes int overflow in the glow backend
                //.on_drag(|pane_data| Message::PaneMessage(PaneMessage::Dragged(pane_data)))
                .on_resize(10, |resize_data| {
                    Message::PaneMessage(PaneMessage::Resize(resize_data))
                })
                .spacing(3);

                let menu_bar_view = menu_bar.view().map(Message::MBMessage);
                let icon_bar_view = icon_bar.view().map(Message::IBMessage);
                Column::new()
                    .push(menu_bar_view)
                    .push(icon_bar_view)
                    .push(pane_grid)
                    .into()
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
