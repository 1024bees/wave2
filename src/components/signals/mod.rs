pub mod sigwindow;
pub mod wavewindow;

#[allow(dead_code)]
/// Module for managing colors -> wave status
pub mod display_wave;
use std::sync::Arc;
use wave2_wavedb::errors::Waverr;
use wave2_wavedb::storage::in_memory::InMemWave;



#[derive(Debug, Clone)]
pub enum Message {
    ///Messages that are shared across wavewindow and signalviewer
    AddWave(Result<Arc<InMemWave>, Arc<Waverr>>),
    SelectedWave(usize), 
    ClearWaves,
    RemoveSelected,

    ///Messages that are only used by wavewindow
    UpdateCursor(u32),
    UpdateBounds((u32, u32)),


    ///Messages that are only used by sigviewer
    CellListPlaceholder,


    ///Messages
    ZoomIn,
    ZoomOut,
    GoToStart,
    GoToEnd,
    Next,
    Prev,
    TIUpdate(Bound, String),
    BoundsUpdate(Bound),
    Noop
}

#[derive(Debug, Clone)]
pub enum Bound {
    Left,
    Right
}
