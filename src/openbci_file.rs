use std;
use regex;


#[allow(dead_code)]
pub struct OpenBCIFile {
    pub path: String,
    pub channels: u32,
    pub sample_rate_hz: u32,
    pub samples: Vec<Vec<f32>>
}

#[allow(dead_code)]
impl OpenBCIFile {
    pub fn new(filename: &str) -> OpenBCIFile {
        use std::io::BufRead;
        let f = std::fs::File::open(&filename).expect("Unabler to open file.");
        let f = std::io::BufReader::new(f);
        let mut channels: i32 = -1;
        let mut samples: Vec<Vec<f32>> = Vec::<Vec<f32>>::new();
        for linew in f.lines() {
            let line = linew.unwrap();
            if line.len() > 0 && line.as_bytes()[0] == '%' as u8 {
                continue;
            }

            let re = regex::Regex::new(r"[ \t,]+").unwrap();
            let fields: Vec<&str> = re.split(&line).collect();
            if channels == -1 { //our first non-header line
                channels = fields.len() as i32 - 5; //ignore i3dex, 3 accelerometers and timestamp to get remaining fields
                samples.resize(channels as usize, Vec::<f32>::new());
            }
            for i in 0..channels {
                samples[i as usize].push(fields[3 + i as usize].parse::<f32>().expect("Found non-float amplitude in file."));
            }

        }
        OpenBCIFile {
            path: String::from(filename),
            channels: channels as u32,
            sample_rate_hz: 0,
            samples: samples
        }
    }


}
