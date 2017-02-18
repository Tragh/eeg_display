use std;
use num;
use appstate;
use appstate::{AppState, WaveData, Ticker, AppData};
use rustfft;

use glium;
use glium::{Surface};

use city2d;
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
    pub dtft_samples: u32, //how many ms to take for fft window
    pub channel: u32, //which chanel to read from
}

#[allow(dead_code)]
pub struct WaveformDrawer<'a> {
    settings: WaveformDrawerSettings,
    rendered_ticks: u64,
    image: Option<City2D>,
    vstrips: Vec<VStrip>,
    texture: glium::texture::Texture2d,
    display: &'a glium::backend::glutin_backend::GlutinFacade,
    running: bool,
    start_ticks: u64,
}

impl<'a> WaveformDrawer<'a> {
    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade, settings: WaveformDrawerSettings)->WaveformDrawer{
        let texture = glium::texture::Texture2d::empty(display,settings.width, settings.height).expect("WaveformDrawer unable to create initial texture.");
        texture.as_surface().clear_color(0.0,0.0,0.0,1.0);
        WaveformDrawer{
            image: Some(City2D::new(settings.width,settings.height)),
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




    pub fn update_stft(&mut self, ticks: u64, app_data: &std::sync::Arc<std::sync::Mutex<AppData>>){
        if !self.running {return;}
        let ticks = ticks-self.start_ticks;
        let settings=&mut self.settings;

        {
            let data_arc = app_data.clone();
            let mut data = data_arc.lock().unwrap();
            let sample_point = ticks as u32 * data.get_sample_rate().unwrap()/1000; //what point (index) in the data are we at


            //if we're too near the begining to do a DTFT or we're past the end of the data then we draw our texture and return
            if sample_point as usize >= data.buffer_length().unwrap() {
                return;
            }


            let dtft_len = std::cmp::min(sample_point, settings.dtft_samples); //how many points to sample for the DTFT

            //how many pixels (width) these samples will take up
            let needed_pixels=((ticks - self.rendered_ticks) as f32 / settings.milliseconds_per_pixel) as u32;
            let needed_pixels = std::cmp::min(needed_pixels, settings.width);



            if needed_pixels != 0 {
                let mut signal = vec![num::Complex{re: 0.0, im: 0.0}; dtft_len as usize];
                let mut spectrum = signal.clone();
                let slice = data.get_slice(settings.channel as usize, (sample_point-dtft_len) as usize,(sample_point) as usize);
                for i in (0)..dtft_len {
                    signal[i as usize].re=slice[i as usize];
                }
                let mut fft = rustfft::FFT::new(dtft_len as usize, false);
                fft.process(&signal, &mut spectrum);

                let mut mean_norm : f32 = 0.0;
                for i in 0..dtft_len/2 {
                    let norm=spectrum[i as usize].norm();
                    mean_norm += norm;
                }

                mean_norm /= dtft_len as f32;

                let mut vstrip=VStrip::new(settings.height,needed_pixels);
                for i in 0..dtft_len/2 {
                    let norm_spec_val=spectrum[i as usize]/mean_norm;

                    let ired=norm_spec_val.norm().atan()*150.0;
                    let igre=((norm_spec_val.norm()+2.718).ln()-1.0)*50.0;
                    let iblu=std::cmp::min((mean_norm/10.0) as u64,255);

                    vstrip.write_pixel(i, ired as u8, igre as u8, iblu as u8);
                }
                self.rendered_ticks=ticks; //update the counter now that we're done drawing
                self.vstrips.push(vstrip);
            }

        } //end scope for data mutex


    }



    pub fn generate_and_draw_texture(&mut self, target: &mut glium::Frame, win_w: u32, win_h: u32){
        if !self.running {return;}
        if self.vstrips.len()!=0 {

            let wfwidth=self.settings.width;
            let wfheight=self.settings.height;

            //the section below glues together all the vstrips to create the right hand side of the graph
            let mut width = 0;
            for vstrip in &self.vstrips {width+=vstrip.pixels;}
            let mut image=City2D::new(width, wfheight);

            let mut x: i32 = 0;
            for vstrip in &self.vstrips{
                for i in 0..wfheight as i32 {
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
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_vec(), dims);
            let texture = glium::texture::Texture2d::new(self.display, image).unwrap();
            {
                let width = texture.get_width();
                let height = texture.get_height().unwrap();
                let target_width = self.texture.get_width();
                let target_height = self.texture.get_height().unwrap();

                let new_waveform_texture: glium::texture::Texture2d = glium::texture::Texture2d::empty(self.display,wfwidth, wfheight).unwrap();
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
        let sfb = tex.as_surface();
        target.blit_from_simple_framebuffer(&sfb,
            &glium::Rect{
                left:0,
                bottom:0,
                width: width,
                height: height},
            &glium::BlitTarget{
                //basically 0,0 is the centre of the screen and texture and distances are in pixels, so we need to transform the coordinates a bit
                left: (self.settings.x - self.settings.width as i32 /2 + win_w as i32/2) as u32,
                bottom: (self.settings.y - self.settings.height as i32/2 + win_h as i32/2) as u32,
                width: width as i32,
                height: height as i32},
            glium::uniforms::MagnifySamplerFilter::Nearest);
    }
}
