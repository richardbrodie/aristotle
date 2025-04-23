use std::io::Read;

use crate::text::geom::Rect;

use super::Error;

#[derive(Debug)]
pub struct Image {
    data: Vec<u8>,
    data_len: usize,
    pub pixel_size: usize,
    pub size: Rect,
}
impl Image {
    pub fn open<R: Read>(data: R) -> Result<Self, Error> {
        // open png
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info()?;
        let img_info = reader.info();

        // get dimensions and pixel depth
        let pixel_size = img_info.color_type.samples();
        let size = Rect {
            width: img_info.width as usize,
            height: img_info.height as usize,
        };

        // read data to buffer
        let mut data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut data)?;

        Ok(Self {
            data,
            data_len: info.buffer_size(),
            pixel_size,
            size,
        })
    }
    pub fn rescale(&self, scale: f32) -> Self {
        let ps = self.pixel_size;
        let new_width = (self.size.width as f32 * scale) as usize;
        let new_height = (self.size.height as f32 * scale) as usize;
        let mut new_data = vec![0; new_width * new_height * ps];
        for nh in 0..new_height {
            let oh = (nh as f32 / scale).floor() as usize;
            for nw in 0..new_width {
                let ow = (nw as f32 / scale).floor() as usize;
                let oidx = (oh * self.size.width * ps) + (ow * ps);
                let nidx = (nh * new_width * ps) + (nw * ps);
                let pixels = &self.data[oidx..oidx + ps];
                new_data[nidx..nidx + ps].copy_from_slice(pixels);
            }
        }
        let data_len = new_data.len();
        return Self {
            data: new_data,
            data_len,
            size: Rect {
                width: new_width,
                height: new_height,
            },
            pixel_size: self.pixel_size,
        };
    }
    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_len]
    }
}
