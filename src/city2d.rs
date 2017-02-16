#![allow(dead_code)]

use std;

#[derive(Clone)]
pub struct City2D {
    data: Vec<u8>,
    pub w: u32,
    pub h: u32
}

impl City2D {
    pub fn new(width: u32, height: u32) -> City2D{
        let mut data=Vec::<u8>::new();
        data.resize((width*height*4) as usize,0);
        City2D{
            data: data,
            w: width,
            h: height
        }
    }

    pub fn hbar(&mut self, x: i32, y: i32, w: u32, red: u8, green: u8, blue: u8){
        let offset: u32 = 4*(x as u32 + y as u32 * self.w);
        for i in 0..w{
            self.data[(offset+i*4) as usize]=red;
            self.data[(offset+i*4+1) as usize]=green;
            self.data[(offset+i*4+2) as usize]=blue;
            //self.data[(offset+i*4+3) as usize]=255;
        }

    }

    pub fn copy_to(&self, dest: &mut City2D, source_x: i32, source_y: i32, dest_x: i32, dest_y: i32, width: u32, height: u32){
        let source_offset = (4*(source_x + source_y*self.w as i32)) as usize;
        let dest_offset = (4*(dest_x + dest_y*dest.w as i32)) as usize;
        for j in 0..height{
            //for i in 0..width{
                let source_copy_offset = (4*j*self.w) as usize;
                let dest_copy_offset = (4*j*dest.w ) as usize;
                //dest.data[dest_offset+dest_copy_offset]=self.data[source_offset+source_copy_offset];
                dest.data[(dest_offset+dest_copy_offset)..(dest_offset+dest_copy_offset+4*width as usize)].clone_from_slice(&self.data[(source_offset+source_copy_offset)..(source_offset+source_copy_offset+4*width as usize)]);
            //}
        }
    }

    pub fn into_vec(self) -> Vec<u8>{
        self.data
    }
}
