use std::{fs::File, path::Path};

use crate::text::geom::{Point, Rect};

struct Image {
    data: Vec<u8>,
    pos: Point,
    size: Rect,
}
impl Image {
    pub fn load<T: AsRef<Path>>(path: T) -> Self {
        let image_file = File::open(path).unwrap();
        let decoder = png::Decoder::new(image_file);
        let mut reader = decoder.read_info().unwrap();
        let img_info = reader.info();
        let pos = Point::default();
        let size = Rect {
            width: img_info.width,
            height: img_info.height,
        };
        let mut data = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut data).unwrap();
        Self { data, pos, size }
    }
}
