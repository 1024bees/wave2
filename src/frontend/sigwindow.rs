use wave2_wavedb::{InMemWave};
use crate::frontend::display_wave::WaveDisplayOptions;
use crate::frontend::wavewindow;
use wave2_custom_widgets::cell_list;
use iced::{button, scrollable, text_input, Align, Column, TextInput};


struct DisplayedWave {
    wave_content : InMemWave,
    display_conf : Option<WaveDisplayOptions>
}

enum WaveOptions {
    Delete
}


pub struct SigViewer {
    waves_state : cell_list::State<WaveOptions>,
    wavewindow : wavewindow::WaveWindowState,
    live_waves : Vec<DisplayedWave>,
    scroll: scrollable::State,
}



