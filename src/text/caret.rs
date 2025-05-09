use crate::text::{fonts::FontStyle, geom::Point};

use super::{TextError, TypesetConfig};

#[derive(Debug, Default, Clone, Copy)]
pub struct Caret {
    scaled_height: f32,
    space_width: f32,
    horizontal_margin: f32,
    vertical_margin: f32,
    page_width: f32,
    page_height: f32,
    point: Point,
}
impl Caret {
    pub fn new(config: &TypesetConfig) -> Result<Self, TextError> {
        let face = config.family.face(FontStyle::default())?;
        let scaled_height = face.scaled_height(config.point_size)?;
        let space_width = face.space_width(config.point_size)?;
        let point = Point::new(
            config.horizontal_margin.into(),
            config.vertical_margin.into(),
        );
        Ok(Self {
            scaled_height,
            space_width,
            horizontal_margin: config.horizontal_margin as f32,
            vertical_margin: config.vertical_margin as f32,
            page_width: config.page_width as f32,
            page_height: config.page_height as f32,
            point,
        })
    }
    pub fn point(&self) -> Point {
        self.point
    }
    pub fn scaled_height(&self) -> f32 {
        self.scaled_height
    }
    pub fn newline(&mut self, lines: f32) {
        self.point = Point::new(
            self.horizontal_margin,
            self.point.y + self.scaled_height * lines,
        );
    }

    pub fn reset_location(&mut self) {
        self.point = Point::new(self.horizontal_margin, self.vertical_margin);
    }
    pub fn advance(&mut self, hadv: f32) {
        self.point.x += hadv;
    }
    pub fn space(&mut self) {
        self.point.x += self.space_width;
    }

    pub fn overflows_horizontally(&self, hadv: f32) -> bool {
        self.point.x + hadv + self.horizontal_margin > self.page_width
    }

    pub fn overflows_vertically(&self, vadv: f32) -> bool {
        self.point.y + self.scaled_height + (vadv * self.scaled_height)
            >= self.page_height - self.vertical_margin
    }
}
