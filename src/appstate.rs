use std;

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

#[derive(PartialEq)]
pub enum DataSource {
    NoSource,
    PortAudio,
    WavBuffer,
}


pub struct WaveData{
    pub buffer: Vec<Vec<f32>>,
    pub channels: u32,
    pub sample_rate: u32,
    pub buffer_length: usize
}

pub struct StreamingData{
    pub deque: Vec<f32>,
    pub channels: u32,
    pub sample_rate: u32,
    pub samples_written: usize,
    pub frames_lag: usize,
    pub acceptable_frames_lag: usize
}

pub struct AppData{
    pub data_source: DataSource,
    pub wave_data: Option<WaveData>,
    pub streaming_data: Option<StreamingData>
}

impl AppData{
    pub fn buffer_length(&self)-> Result<usize, &'static str>{
        if self.data_source == DataSource::WavBuffer {
            return Ok(self.wave_data.as_ref().unwrap().buffer_length);
        }
        if self.data_source == DataSource::PortAudio {
            return Ok(self.streaming_data.as_ref().unwrap().samples_written);
        }
        Err("No AppData: DataSource is None")
    }
    pub fn get_sample_rate(&self) -> Result<u32, &'static str> {
        if self.data_source == DataSource::WavBuffer {
            return Ok(self.wave_data.as_ref().unwrap().sample_rate);
        }
        if self.data_source == DataSource::PortAudio {
            return Ok(self.streaming_data.as_ref().unwrap().sample_rate);
        }
        Err("No AppData: DataSource is None")
    }
    /*pub fn readbuffer(&mut self, channel: usize, i: usize) -> f32 {
        if self.data_source == DataSource::WavBuffer {
            return self.wave_data.as_ref().unwrap().buffer[channel][i];
        }
        if self.data_source == DataSource::PortAudio {
            return self.streaming_data.as_ref().unwrap().deque[i];
        }

        assert!(false,"Failure to read buffer: DataSource is None.");0.0
    }*/

    pub fn get_slice(&mut self, channel: usize, ia: usize, ib: usize) -> &[f32] {
        if self.data_source == DataSource::WavBuffer {
            return &self.wave_data.as_ref().unwrap().buffer[channel][ia..ib];
        }
        if self.data_source == DataSource::PortAudio {
            let sd = self.streaming_data.as_mut().unwrap();
            assert!(ib-ia < sd.deque.len(),"Buffer underrun.");
            assert!(sd.deque.len() == sd.samples_written); //remove this assert when you get round to cutting the vector down to stop it growing too much
            if ib - sd.frames_lag > sd.samples_written {
                sd.frames_lag=ib-sd.samples_written;
                println!("Increasing frames lag to {} frames.", sd.frames_lag);
            }
            if sd.frames_lag > sd.acceptable_frames_lag && ib - sd.frames_lag < sd.samples_written {
                sd.frames_lag -= 1;
                println!("Frame lag above acceptable lag of {}, decreasing it to {}.",sd.acceptable_frames_lag,sd.frames_lag);
            }
            let ib = ib - sd.frames_lag;
            let ia = ia - sd.frames_lag;
        //    println!("Reading from location {}", ib);
            return &sd.deque[ia..ib];
        }

        assert!(false,"Failure to read buffer: DataSource is None.");
        &[]
    }
}

#[derive(PartialEq)]
pub enum GuiDisplay{
    FileOpen,
    FilterOptions,
    Nothing,
}

pub struct GuiData{
    pub gui_display: GuiDisplay,
    pub file_selection: Option<std::path::PathBuf>,
    pub cursor_xy: (f32,f32),
}

pub struct FilterData{
    pub red: (f32,f32),
    pub min_red: (f32,f32),
    pub max_red: (f32,f32),
    pub green: (f32,f32),
    pub min_green: (f32,f32),
    pub max_green: (f32,f32),
    pub blue: (f32,f32),
    pub min_blue: (f32,f32),
    pub max_blue: (f32,f32),
    pub amp: f32,
    pub amp_min: f32,
    pub amp_max: f32,
    pub amp_manual: bool,
    pub window_shape: i32,
}
impl Default for FilterData{
    fn default()->FilterData{
        FilterData{
             red: (0.5, 100.0),
             min_red: (0.1, 50.0),
             max_red: (4.0,250.0 ),
             green: (2.0, 2.0),
             min_green: (0.1, 0.1),
             max_green: (5.0, 5.0),
             blue: (127.0, -5.0),
             min_blue: (0.0, -8.0),
             max_blue: (255.0, 8.0),
             amp: 0.0,
             amp_min: -10.0,
             amp_max: 5.0,
             amp_manual: false,
             window_shape: 0,
        }
    }
}

pub struct AppState<'a>{
    pub filter_data: FilterData,
    pub gui_data: GuiData,
    pub waveform_drawers: Vec<WaveformDrawer<'a>>,
    pub ticker: Ticker,
    pub app_data: std::sync::Arc<std::sync::Mutex<AppData>>
}
