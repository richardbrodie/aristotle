use softbuffer::Surface;
use std::{cmp, num::NonZeroU32, rc::Rc};
use winit::{dpi::PhysicalSize, window::Window};

use crate::text::{
    fonts::Family,
    geom::{Point, Rect},
    typeset::TypesetText,
};

use super::{builder::Builder, Error, Image};

pub struct Canvas {
    surface: Surface<Rc<Window>, Rc<Window>>,
    size: Rect,
}
impl Canvas {
    pub fn new(window: Rc<Window>, size: Rect) -> Result<Self, Error> {
        let context = softbuffer::Context::new(window.clone())?;
        let surface = softbuffer::Surface::new(&context, window)?;
        Ok(Self { surface, size })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<(), Error> {
        self.surface.resize(
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        )?;
        self.size = Rect {
            width: size.width as usize,
            height: size.height as usize,
        };
        Ok(())
    }

    pub fn present(&mut self) -> Result<(), Error> {
        let res = self.surface.buffer_mut()?.present().map_err(|e| e.into());
        res
    }

    pub fn blank(&mut self) -> Result<(), Error> {
        let mut buffer = self.surface.buffer_mut()?;

        // fill every pixel with white
        buffer.fill(0x00ffffff);
        Ok(())
    }

    pub fn draw_line(&mut self, start: &Point, end: &Point) -> Result<(), Error> {
        let mut buffer = self.surface.buffer_mut()?;

        for y in start.y as usize..end.y as usize {
            let py = y * self.size.width as usize;
            for x in start.x as usize..end.x as usize {
                let idx = py + x;
                buffer[idx] = 0;
            }
        }
        Ok(())
    }

    pub fn text(&mut self, family: &Family, text: &TypesetText) -> Result<(), Error> {
        let face = family.face(text.style)?;
        let scale_factor = face.scale_factor(text.point_size);
        let face = face.as_ttf_face()?;
        let mut buffer = self.surface.buffer_mut()?;

        let desc = face.descender() as f32;
        let asc = face.ascender() as f32;
        let h = ((asc - desc) * scale_factor).ceil();

        let mut builder = Builder::new(face.descender(), scale_factor);
        for g in text.glyphs.iter() {
            let w = (g.advance * scale_factor).ceil();
            builder.reset(w as usize, h as usize, -g.bearing);
            if let Some(og) = face.outline_glyph(g.id, &mut builder) {
                builder.rasteriser.for_each_pixel_2d(|x, y, v| {
                    //convert 0-1 range to 0-255
                    let mut byte = (v.clamp(0.0, 1.0) * 255.0) as u8;

                    //if there's no coverage just stop immediately
                    if byte == 0 {
                        return;
                    }

                    let bbox_min_x = (og.x_min as f32 - g.bearing) * scale_factor;
                    let bbox_max_x = (og.x_max as f32 - g.bearing) * scale_factor;
                    let bbox_min_y = (og.y_min as f32 - g.desc) * scale_factor;
                    let bbox_max_y = (og.y_max as f32 - g.desc) * scale_factor;

                    // don't draw pixels we know are outside the bbox
                    if x < bbox_min_x as u32
                        || x > bbox_max_x as u32
                        || y > bbox_max_y as u32
                        || y < bbox_min_y as u32
                    {
                        return;
                    }

                    // don't draw white pixels inside the bbox either
                    if byte == 0 {
                        return;
                    }

                    // invert so that more coverage means less fill
                    byte = 255 - byte;

                    // invert glyph along the y-axis
                    let y = h as u32 - y;

                    // translate xy coords to the glyph position
                    //let xoff = (og.x_min as f32 + min.x) * scale_factor;
                    let xoff = og.x_min as f32 * scale_factor;
                    //let xoff = min.x as f32 * scale_factor;
                    let x = x + (g.pos.x + xoff) as u32;
                    let y = y + g.pos.y as u32;

                    let z = byte as u32;
                    let c = z | z << 8 | z << 16;
                    let idx = x as usize + y as usize * self.size.width as usize;
                    buffer[idx] = c;
                });
            }
        }
        Ok(())
    }

    pub fn image(&mut self, point: &Point, img: &Image) -> Result<(), Error> {
        // clipping
        let (pos_x, pos_y) = (point.x as usize, point.y as usize);
        let rows = cmp::min(img.size.height, self.size.height - pos_y);
        let cols = cmp::min(img.size.width, self.size.width - pos_x);

        let mut buffer = self.surface.buffer_mut()?;

        let ps = img.pixel_size as usize;
        // draw
        (0..rows).for_each(|rownum| {
            let line_start = (pos_y + rownum) * self.size.width + pos_x;
            let img_row_start = (img.size.width * ps) * rownum;
            let img_row_len = img_row_start + cols * ps;
            for (i, pixel) in img.data()[img_row_start..img_row_len]
                .chunks_exact(ps)
                .enumerate()
            {
                let idx = line_start + i;
                // convert to greyscale
                let pixel: u32 = pixel[0] as u32 * 299 / 1000
                    + pixel[1] as u32 * 587 / 1000
                    + pixel[2] as u32 * 114 / 1000;
                buffer[idx] = pixel << 16 | pixel << 8 | pixel;
            }
        });
        Ok(())
    }
}
