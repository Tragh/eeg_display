use std;

use waveformdrawer;
use waveformdrawer::{WaveformDrawer};

pub struct Ticker {
    timer: std::time::SystemTime
}
impl Ticker {
    pub fn new() -> Ticker{
        Ticker{timer: std::time::SystemTime::now()}
    }
    pub fn ticks(&self) -> u64 {
        let now=self.timer.elapsed().expect("Timer Error.");
        return now.as_secs() * 1000 + now.subsec_nanos() as u64 / 1000000;
    }
}
impl Default for Ticker{
    fn default() -> Ticker {
        Ticker::new()
    }
}




pub struct WaveData{
    pub buffer: Vec<Vec<f32>>,
    pub channels: u32,
    pub sample_rate: u32,
}

pub struct AppState<'a>{
    pub show_file_widgets: bool,
    pub file_selection: Option<std::path::PathBuf>,
    pub waveform_drawers: Vec<WaveformDrawer<'a>>,
    pub ticker: Ticker,
    pub wave_data: Option<WaveData>
}
