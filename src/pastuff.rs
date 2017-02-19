use std;

use portaudio as pa;

use appstate;
use appstate::{AppState, StreamingData};

const PA_SAMPLE_RATE: f64 = 44_100.0;
const PA_FRAMES: u32 = 4096;
const PA_CHANNELS: i32 = 1;
const PA_INTERLEAVED: bool = true;


pub fn pa_read_from_mic<'a>(app: &mut AppState) {



    { //write stuff into the appstate
        let audio_dequeue: Vec<f32> = Vec::<f32>::new();
        let streaming_data = StreamingData {
            deque: audio_dequeue,
            channels: PA_CHANNELS as u32,
            sample_rate: PA_SAMPLE_RATE as u32,
            samples_written: 0,
            frames_lag: 0,
            acceptable_frames_lag: 512
        };

        let mut app_data = app.app_data.lock().unwrap();
        app_data.streaming_data = Some(streaming_data);
        app_data.data_source = appstate::DataSource::PortAudio;
        println!("DataSource is now PortAudio.");
    }


    let closure_data=app.app_data.clone();
    std::thread::spawn(move || {


        let pa = pa::PortAudio::new().expect("PortAudio Error");

        println!("PortAudio");
        println!("version: {}", pa.version());
        println!("version text: {:?}", pa.version_text());
        println!("host count: {}", pa.host_api_count().expect("PortAudio Error"));

        let default_host = pa.default_host_api().expect("PortAudio Error");
        println!("default host: {:#?}", pa.host_api_info(default_host));

        let def_input = pa.default_input_device().expect("PortAudio Error");
        let input_info = pa.device_info(def_input).expect("PortAudio Error");
        println!("Default input device info: {:#?}", &input_info);

        // Construct the input stream parameters.
        let latency = input_info.default_high_input_latency;
        let input_params = pa::StreamParameters::<f32>::new(def_input, PA_CHANNELS, PA_INTERLEAVED, latency);
        let settings = pa::InputStreamSettings::new(input_params, PA_SAMPLE_RATE, PA_FRAMES);
        let mut stream = pa.open_blocking_stream(settings).expect("PortAudio Error");

        // We'll use this buffer to transfer samples from the input stream to the output stream.

        stream.start().expect("PortAudio Error");

        fn wait_for_stream<F>(f: F, name: &str) -> u32
            where F: Fn() -> Result<pa::StreamAvailable, pa::error::Error>
        {
            'waiting_for_stream: loop {
                match f() {
                    Ok(available) => match available {
                        pa::StreamAvailable::Frames(frames) => return frames as u32,
                        pa::StreamAvailable::InputOverflowed => println!("Input stream has overflowed"),
                        pa::StreamAvailable::OutputUnderflowed => println!("Output stream has underflowed"),
                    },
                    Err(err) => panic!("An error occurred while waiting for the {} stream: {}", name, err),
                }
            }
        };


        'stream: loop {
            let in_frames = wait_for_stream(|| stream.read_available(), "Read");
            if in_frames > 0 {
                let input_samples = stream.read(in_frames).expect("PortAudio Error");

                let mut app_data = closure_data.lock().unwrap();
                app_data.streaming_data.as_mut().unwrap().samples_written += in_frames as usize;
                let ref mut audio_dequeue = app_data.streaming_data.as_mut().unwrap().deque;

                for i in 0..in_frames {
                    //audio_dequeue.push( (input_samples[(2*i) as usize]+input_samples[(2*i+1) as usize]) /2.0 );
                    audio_dequeue.extend(input_samples.into_iter());
                }
                println!("Read {:?} frames from the input stream.", in_frames);
            } else {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }

        }
    
    });

    std::thread::sleep(std::time::Duration::from_millis(100)); //give the PA thread 100ms headstart

}
