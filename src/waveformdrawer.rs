use std;
use appstate::{AppData, FilterData};
use rustfft;
use dftwindower::{DFTWindower};

use glium;
use glium::{Surface};

use city2d::City2D;

#[allow(dead_code)]
struct VStrip{
    pub strip: Vec<u8>,
    pub height: u32,
    pub pixels: u32
}

#[allow(dead_code)]
impl VStrip{
    pub fn new(height: u32, pixels: u32) -> VStrip{
        VStrip{
            strip: vec![0 as u8; (height*4) as usize],
            height: height,
            pixels: pixels
        }
    }
    pub fn write_pixel(&mut self, index: u32, red: u8, green: u8, blue: u8){
        let index = index*4;
        self.strip[(index) as usize]=red;
        self.strip[(index+1) as usize]=green;
        self.strip[(index+2) as usize]=blue;
    }
}

#[allow(dead_code)]
pub struct WaveformDrawerSettings {
    pub x: f32, //x coord of the display
    pub y: f32, //y coord of display
    pub width: f32, //width of the display
    pub height: f32,    //height of the display
    pub milliseconds_per_pixel: f32,  //how much data to display, in ms
    pub time_pixels: u32,  //how many time periods to have
    pub dtft_samples: u32, //how many samples to take for the ftft window
    pub dtft_display_samples: u32, //how many of the above samples to display (cuts off high frequency samples)
    pub channel: u32, //which chanel to read from
}

#[allow(dead_code)]
pub struct WaveformDrawer<'a> {
    pub settings: WaveformDrawerSettings,
    rendered_ticks: u64,
//    image: Option<City2D>,
    vstrips: Vec<VStrip>,
    texture: glium::texture::Texture2d,
    texture_w: u32,
    texture_h: u32,
    display: &'a glium::Display,
    running: bool,
    start_ticks: u64,
    dft_windower: DFTWindower,
}

impl<'a> WaveformDrawer<'a> {
    pub fn new(display: &glium::Display, settings: WaveformDrawerSettings)->WaveformDrawer{
        let texture_width= settings.time_pixels;
        let texture_height= settings.dtft_display_samples;
        let dtft_samples = settings.dtft_samples;
        let texture = glium::texture::Texture2d::empty(display,texture_width, texture_height).expect("WaveformDrawer unable to create initial texture.");
        texture.as_surface().clear_color(0.0,0.0,0.0,1.0);
        WaveformDrawer{
            //image: Some(City2D::new(settings.width,settings.dtft_display_samples)),
            texture: texture,
            texture_w: texture_width,
            texture_h: texture_height,
            settings: settings,
            rendered_ticks: 0,
            vstrips: Vec::<VStrip>::new(),
            display: display,
            running: false,
            start_ticks: 0,
            dft_windower: DFTWindower::new(dtft_samples),
        }
    }


    pub fn start(&mut self, ticks: u64){
        self.start_ticks=ticks;
        self.running=true;
    }




    pub fn update_stft(&mut self, ticks: u64, app_data: &std::sync::Arc<std::sync::Mutex<AppData>>, fd: &FilterData){
        if !self.running {return;}
        let ticks = ticks-self.start_ticks;
        let settings=&mut self.settings;

        let mut signal = Vec::<rustfft::num_complex::Complex<f32>>::new();
        let dtft_len: u32;
        let dtft_display_len: u32;
        let mut needed_pixels: u32;
        { //lock the data mutex here
            let data_arc = app_data.clone();
            let mut data = data_arc.lock().unwrap();
            let sample_point: u64 = ticks * data.get_sample_rate().unwrap() as u64 / 1000; //what point (index) in the data are we at


            //if we're too near the begining to do a DTFT or we're past the end of the data then we draw our texture and return
            if sample_point as usize >= data.buffer_length().unwrap() {
                return;
            }


            dtft_len = std::cmp::min(sample_point, settings.dtft_samples as u64) as u32; //how many points to sample for the DTFT
            dtft_display_len = std::cmp::min(sample_point, settings.dtft_display_samples as u64) as u32; //how many points to sample for the DTFT

            //how many pixels (width) these samples will take up
            needed_pixels=((ticks - self.rendered_ticks) as f32 / settings.milliseconds_per_pixel) as u32;
            needed_pixels = std::cmp::min(needed_pixels, self.texture_w);


            if needed_pixels != 0 {
                signal = vec![rustfft::num_complex::Complex{re: 0.0, im: 0.0}; dtft_len as usize];
                let slice = data.get_slice(settings.channel as usize, (sample_point-dtft_len as u64) as usize,(sample_point) as usize);
                for i in (0)..dtft_len {
                    signal[i as usize].re=slice[i as usize];
                }
            }
        } //unlock data mutex here

        if needed_pixels != 0 {

            match fd.window_shape {
                0 /*none*/ => {},
                1 /*Hann*/ => {
                    for i in (0)..dtft_len {
                        signal[i as usize].re = signal[i as usize].re*self.dft_windower.hann(i,dtft_len);
                    }
                },
                2 /*Hamming*/ => {
                    for i in (0)..dtft_len {
                       signal[i as usize].re = signal[i as usize].re*self.dft_windower.hamming(i,dtft_len);
                    }
                },
                3 /*Nuttall*/ => {
                    for i in (0)..dtft_len {
                       signal[i as usize].re = signal[i as usize].re*self.dft_windower.nuttall(i,dtft_len);
                    }
                },
                4 /*Sine*/ => {
                    for i in (0)..dtft_len {
                       signal[i as usize].re = signal[i as usize].re*self.dft_windower.sine(i,dtft_len);
                    }
                },
                5 /*Kaiser*/ => {
                    for i in (0)..dtft_len {
                       signal[i as usize].re = signal[i as usize].re*self.dft_windower.kaiser(i,dtft_len);
                    }
                },
                _=>{}
            }

            let mut spectrum = signal.clone();
            let mut fft_planner = rustfft::FFTplanner::new(false);
            let fft = fft_planner.plan_fft(dtft_len as usize);
            fft.process(&mut signal, &mut spectrum);

            let mut mean_norm : f32 = 0.0;
            for i in 0..dtft_display_len {
                let norm=spectrum[i as usize].norm();
                mean_norm += norm;
            }
            if mean_norm == 0.0 {mean_norm=1.0;}
            mean_norm /= (dtft_display_len/2) as f32;

            let mut vstrip=VStrip::new(settings.dtft_display_samples,needed_pixels);
            for i in 0..dtft_display_len {
                let norm_spec_val = if fd.amp_manual {spectrum[i as usize]*fd.amp.exp()} else {spectrum[i as usize]/mean_norm};
                //let norm_spec_val=spectrum[i as usize]/mean_norm;

                let ired=std::cmp::min(   ((norm_spec_val*fd.red.0).norm().atan()*fd.red.1)   as u64,255);
                //let igre=std::cmp::min(   ((((norm_spec_val.norm()*fd.green.0)+2.718).ln()-1.0)*fd.green.1)   as u64,255);
                let igre=std::cmp::min(   (norm_spec_val.norm()*fd.green.0.exp() + (1.0+norm_spec_val.norm()).ln() * fd.green.1.exp() )   as u64,255);
                let iblu=std::cmp::min(   (mean_norm*fd.blue.1.exp())   as u64,fd.blue.0 as u64);

                vstrip.write_pixel(dtft_display_len-i-1, ired as u8, igre as u8, iblu as u8);
            }
            self.rendered_ticks=ticks; //update the counter now that we're done drawing
            self.vstrips.push(vstrip);
        }

    }



    pub fn generate_and_draw_texture(&mut self, target: &mut glium::Frame){
        let (fb_w,fb_h)=target.get_dimensions();
        if !self.running {return;}
        if self.vstrips.len()!=0 {

            let texture_width=self.settings.time_pixels;
            let texture_height=self.settings.dtft_display_samples;

            //the section below glues together all the vstrips to create the right hand side of the graph
            let mut width = 0;
            for vstrip in &self.vstrips {width+=vstrip.pixels;}
            let mut image=City2D::new(width, texture_height);

            let mut x: i32 = 0;
            for vstrip in &self.vstrips{
                for i in 0..texture_height as i32 {
                    image.hbar(x, i, vstrip.pixels,
                        vstrip.strip[(i*4) as usize],
                        vstrip.strip[(i*4+1) as usize],
                        vstrip.strip[(i*4+2) as usize]);
                }
                x += vstrip.pixels as i32;
            }
            self.vstrips.clear();

            //image is now the tiny slice at the right hand side that we need to glue on to the texture
            //now we need to slide the texture along so we have room to glue this bit on
            //then glue it on to create the final texture
            let dims = (image.w, image.h);
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_vec(), dims);
            let texture = glium::texture::Texture2d::new(self.display, image).unwrap();
            {
                let width = texture.get_width();
                let height = texture.get_height().unwrap();
                let target_width = self.texture.get_width();
                let target_height = self.texture.get_height().unwrap();

                let new_waveform_texture: glium::texture::Texture2d = glium::texture::Texture2d::empty(self.display, texture_width, texture_height).unwrap();
                {
                    let sfb = new_waveform_texture.as_surface();
                    sfb.blit_from_simple_framebuffer(&self.texture.as_surface(),
                        &glium::Rect{
                            left:width,
                            bottom:0,
                            width: target_width-width,
                            height: height},
                        &glium::BlitTarget{
                            left: 0,
                            bottom:0,
                            width: (target_width-width) as i32,
                            height: target_height as i32},
                        glium::uniforms::MagnifySamplerFilter::Nearest);

                    sfb.blit_from_simple_framebuffer(&texture.as_surface(),
                        &glium::Rect{
                            left:0,
                            bottom:0,
                            width: width,
                            height: height},
                        &glium::BlitTarget{
                            left:target_width-width,
                            bottom:0,
                            width: width as i32,
                            height: height as i32},
                        glium::uniforms::MagnifySamplerFilter::Nearest);
                }
                self.texture=new_waveform_texture;
            }
        }

        //now we have the final texture, we can draw it!
        let tex=&self.texture;
        let width = tex.get_width();
        let height = tex.get_height().unwrap();
        let target_width = self.settings.width * fb_w as f32 / 100.0;
        let target_height = self.settings.height * fb_h as f32 / 100.0;
        let target_x = fb_w as f32 *(self.settings.x - self.settings.width/2.0 +50.0)/100.0;
        let target_y = fb_h as f32 *(self.settings.y - self.settings.height/2.0 +50.0)/100.0;
        let sfb = tex.as_surface();
        target.blit_from_simple_framebuffer(&sfb,
            &glium::Rect{
                left:0,
                bottom:0,
                width: width,
                height: height},
            &glium::BlitTarget{
                //basically 0,0 is the centre of the screen and texture and distances are in pixels, so we need to transform the coordinates a bit
                left: target_x as u32,
                bottom: target_y as u32,
                width: target_width as i32,
                height: target_height as i32},
            glium::uniforms::MagnifySamplerFilter::Linear);
    }
}
