use crate::geom::Point;

pub struct Builder {
    pos: Point,
    offset: Point,
    scale: f32,
    last_move: Option<Point>,
    pub rasteriser: ab_glyph_rasterizer::Rasterizer,
}
impl Builder {
    pub fn new(desc: i16, scale: f32) -> Self {
        Builder {
            pos: Point::new(0.0, 0.0),
            last_move: None,
            offset: Point::new(0.0, (-desc as f32) * scale),
            scale,
            rasteriser: ab_glyph_rasterizer::Rasterizer::new(0, 0),
        }
    }
    pub fn reset(&mut self, w: usize, h: usize, o: f32) {
        self.offset.x = o * self.scale;
        self.rasteriser.reset(w, h);
    }
}

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pos = Point::new(x, y) * self.scale + self.offset;
        self.last_move = Some(self.pos);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = Point::new(x, y) * self.scale + self.offset;
        self.rasteriser.draw_line(self.pos.into(), p.into());
        self.pos = p;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let p1 = Point::new(x1, y1) * self.scale + self.offset;
        let p2 = Point::new(x2, y2) * self.scale + self.offset;
        self.rasteriser
            .draw_quad(self.pos.into(), p1.into(), p2.into());
        self.pos = p2;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        let p1 = Point::new(x1, y1) * self.scale + self.offset;
        let p2 = Point::new(x2, y2) * self.scale + self.offset;
        let p3 = Point::new(x3, y3) * self.scale + self.offset;
        self.rasteriser
            .draw_cubic(self.pos.into(), p1.into(), p2.into(), p3.into());
        self.pos = p3;
    }

    fn close(&mut self) {
        if let Some(m) = self.last_move.take() {
            self.rasteriser.draw_line(self.pos.into(), m.into());
        }
    }
}
