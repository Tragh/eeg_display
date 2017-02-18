use std;

use portaudio as pa;

use appstate;
use appstate::{AppState, WaveData, Ticker, AppData, StreamingData};

const PA_SAMPLE_RATE: f64 = 44_100.0;
const PA_FRAMES: u32 = 256;
const PA_CHANNELS: i32 = 1;
const PA_INTERLEAVED: bool = true;


pub fn pa_read_from_mic<'a>(app: &mut AppState) -> Result<(), pa::Error>{

    //Used to make sure the PA thread has progressed past the
    //initial setup before we continue with the main thread :-)
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let barrier2=barrier.clone();

    let closure_data=app.app_data.clone();
    let closure_data2=app.app_data.clone();
    std::thread::spawn(move || {

        let pa = pa::PortAudio::new().expect("PortAudio Error when starting.");

        { //write stuff into the appstate
            //let mut audio_dequeue: std::collections::VecDeque<f32> = std::collections::VecDeque::<f32>::new();
            let mut audio_dequeue: Vec<f32> = Vec::<f32>::new();
            let mut streaming_data = StreamingData {
                deque: audio_dequeue,
                channels: PA_CHANNELS as u32,
                sample_rate: PA_SAMPLE_RATE as u32,
                samples_written: 0,
                frames_lag: 0,
                acceptable_frames_lag: 512
            };

            let app_data_arc = closure_data;
            let mut app_data = app_data_arc.lock().unwrap();
            app_data.streaming_data = Some(streaming_data);
            app_data.data_source = appstate::DataSource::PortAudio;
            println!("DataSource is now PortAudio.");
        }


    //    println!("PortAudio:");
    //    println!("version: {}", pa.version());
    //    println!("version text: {:?}", pa.version_text().expect("PortAudio Error"));
    //    println!("host count: {}", pa.host_api_count().expect("PortAudio Error"));
    //
    //    let default_host = pa.default_host_api().expect("PortAudio Error");
    //    println!("default host: {:#?}", pa.host_api_info(default_host).expect("PortAudio Error"));

        let def_input = pa.default_input_device().expect("PortAudio Error");
        let input_info = pa.device_info(def_input).expect("PortAudio Error");
    //    println!("Default input device info: {:#?}", &input_info);

        // Construct the input stream parameters.
        let latency = input_info.default_low_input_latency;
        let input_params = pa::StreamParameters::<f32>::new(def_input, PA_CHANNELS, PA_INTERLEAVED, latency);

        let in_settings = pa::stream::InputSettings::new(input_params,PA_SAMPLE_RATE, PA_FRAMES);



        // A callback to pass to the non-blocking stream.
        let callback = move |pa::stream::InputCallbackArgs { buffer, frames, .. }| {

            assert!(frames == PA_FRAMES as usize);

            let mut app_data = closure_data2.lock().unwrap();
            app_data.streaming_data.as_mut().unwrap().samples_written += frames;
            let ref mut audio_dequeue = app_data.streaming_data.as_mut().unwrap().deque;
            // Put the input on the Deque
            for i in 0..frames {
                audio_dequeue.push(buffer[i]);
            }

            return pa::Continue;
        };
        // Construct a stream with input and output sample types of f32.
        let mut stream = pa.open_non_blocking_stream(in_settings, callback).expect("PortAudio Error");
        stream.start().expect("PortAudio Error");

        barrier2.wait();
        'pa_main:loop{
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        stream.stop().expect("PortAudio Error");
    });

    barrier.wait();
    std::thread::sleep(std::time::Duration::from_millis(100)); //give the PA thread 100ms headstart

    return Ok(());
}