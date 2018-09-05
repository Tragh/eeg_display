use std;
use num;
use appstate::{AppData, FilterData};
use rustfft;

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
    pub x: i32, //x coord of the display
    pub y: i32, //y coord of display
    pub width: u32, //width of the display
    pub height: u32,    //height of the display
    pub milliseconds_per_pixel: f32,  //how much data to display, in ms
    pub dtft_samples: u32, //how many samples to take for the ftft window
    pub dtft_display_samples: u32, //how many of the above samples to display (cuts off high frequency samples)
    pub channel: u32, //which chanel to read from
}

#[allow(dead_code)]
pub struct WaveformDrawer<'a> {
    settings: WaveformDrawerSettings,
    rendered_ticks: u64,
//    image: Option<City2D>,
    vstrips: Vec<VStrip>,
    texture: glium::texture::Texture2d,
    display: &'a glium::Display,
    running: bool,
    start_ticks: u64,
}

impl<'a> WaveformDrawer<'a> {
    pub fn new(display: &glium::Display, settings: WaveformDrawerSettings)->WaveformDrawer{
        let texture = glium::texture::Texture2d::empty(display,settings.width, settings.height).expect("WaveformDrawer unable to create initial texture.");
        texture.as_surface().clear_color(0.0,0.0,0.0,1.0);
        WaveformDrawer{
            //image: Some(City2D::new(settings.width,settings.dtft_display_samples)),
            texture: texture,
            settings: settings,
            rendered_ticks: 0,
            vstrips: Vec::<VStrip>::new(),
            display: display,
            running: false,
            start_ticks: 0
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


        let mut signal = Vec::<num::Complex<f32>>::new();
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
            needed_pixels = std::cmp::min(needed_pixels, settings.width);


            if needed_pixels != 0 {
                signal = vec![num::Complex{re: 0.0, im: 0.0}; dtft_len as usize];
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
                        let sin=(std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).sin();
                        signal[i as usize].re = signal[i as usize].re*sin*sin;
                    }
                },
                2 /*Hamming*/ => {
                    for i in (0)..dtft_len {
                       let cos=(std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
                       signal[i as usize].re = signal[i as usize].re*(0.53836 - 0.46164*cos);
                    }
                },
                3 /*Nuttall*/ => {
                    for i in (0)..dtft_len {
                       let cos2=(2.0*std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
                       let cos4=(4.0*std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
                       let cos6=(6.0*std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
                       signal[i as usize].re = signal[i as usize].re*(0.355768 - 0.487396*cos2 + 0.144232*cos4 - 0.012604*cos6);
                    }
                },
                4 /*Sine*/ => {
                    for i in (0)..dtft_len {
                       let sin=(std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).sin();
                       signal[i as usize].re = signal[i as usize].re*sin;
                    }
                },
                _=>{}
            }

            let mut spectrum = signal.clone();
            let mut fft = rustfft::FFT::new(dtft_len as usize, false);
            fft.process(&signal, &mut spectrum);

            let mut mean_norm : f32 = 0.0;
            for i in 0..dtft_display_len {
                let norm=spectrum[i as usize].norm();
                mean_norm += norm;
            }
            if mean_norm == 0.0 {mean_norm=1.0;}
            mean_norm /= (dtft_display_len/2) as f32;

            let mut vstrip=VStrip::new(settings.height,needed_pixels);
            for i in 0..dtft_display_len {
                let norm_spec_val = if fd.amp_manual {spectrum[i as usize]*fd.amp.exp()} else {spectrum[i as usize]/mean_norm};
                //let norm_spec_val=spectrum[i as usize]/mean_norm;

                let ired=std::cmp::min(   ((norm_spec_val*fd.red.0).norm().atan()*fd.red.1)   as u64,255);
                let igre=std::cmp::min(   ((((norm_spec_val.norm()*fd.green.0)+2.718).ln()-1.0)*fd.green.1)   as u64,255);
                let iblu=std::cmp::min(   (mean_norm*fd.blue.1.exp())   as u64,fd.blue.0 as u64);

                vstrip.write_pixel(i, ired as u8, igre as u8, iblu as u8);
            }
            self.rendered_ticks=ticks; //update the counter now that we're done drawing
            self.vstrips.push(vstrip);
        }

    }



    pub fn generate_and_draw_texture(&mut self, target: &mut glium::Frame){
        let (fb_w,fb_h)=target.get_dimensions();
        if !self.running {return;}
        if self.vstrips.len()!=0 {

            let texture_width=self.settings.width;
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
        let target_width = self.settings.width;
        let target_height = self.settings.height;
        let sfb = tex.as_surface();
        target.blit_from_simple_framebuffer(&sfb,
            &glium::Rect{
                left:0,
                bottom:0,
                width: width,
                height: height},
            &glium::BlitTarget{
                //basically 0,0 is the centre of the screen and texture and distances are in pixels, so we need to transform the coordinates a bit
                left: (self.settings.x - target_width as i32 /2 + fb_w as i32/2) as u32,
                bottom: (self.settings.y - target_height as i32/2 + fb_h as i32/2) as u32,
                width: target_width as i32,
                height: target_height as i32},
            glium::uniforms::MagnifySamplerFilter::Linear);
    }
}
