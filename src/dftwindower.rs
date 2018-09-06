use std;


fn bessel0(x: f32)->f32{
    let mut result:f32=0.0;
    let mut k:f32=0.0;
    let mut term:f32=1.0;
    while{
        result+=term*term;
        k+=1.0;
        term*=x/(2.0*k);
        term>0.0001
    }{}
    result+term*term
}

#[allow(dead_code)]
pub struct DFTWindower{
 samples: u32,
 kaiser_table: Vec<f32>,
}

impl DFTWindower{
    pub fn new(samples: u32)->DFTWindower{
        let mut kaiser_table:Vec<f32> = vec![0.0; samples as usize];
        let b:f32=std::f32::consts::PI;
        let d=bessel0(b);
        for n in 0..(samples){
            let t:f32=2.0*n as f32/(samples as f32 - 1.0) - 1.0;
            kaiser_table[n as usize]=bessel0(b*(1.0-t*t).sqrt())/d;
        }

        DFTWindower{
            samples: samples,
            kaiser_table: kaiser_table,
        }
    }
    pub fn hann(&self, i: u32, dtft_len: u32)->f32{
        let sin=(std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).sin();
        sin*sin
    }
    pub fn hamming(&self, i: u32, dtft_len: u32)->f32{
        let cos=(std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
        (0.53836 - 0.46164*cos)
    }
    pub fn nuttall(&self, i: u32, dtft_len: u32)->f32{
        let cos2=(2.0*std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
        let cos4=(4.0*std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
        let cos6=(6.0*std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).cos();
        (0.355768 - 0.487396*cos2 + 0.144232*cos4 - 0.012604*cos6)
    }
    pub fn sine(&self, i: u32, dtft_len: u32)->f32{
        (std::f32::consts::PI*i as f32/(dtft_len - 1) as f32 ).sin()
    }
    #[allow(unused_variables)]
    pub fn kaiser(&self, i: u32, dtft_len: u32)->f32{
        self.kaiser_table[i as usize]
    }
}
