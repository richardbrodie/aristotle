use std::ops::DerefMut;

use crate::text::geom::Point;
use crate::text::FontError;

pub fn horizontal_line<B>(
    start: &Point,
    end: &Point,
    width: usize,
    buffer: &mut B,
) -> Result<(), FontError>
where
    B: DerefMut<Target = [u32]>,
{
    for y in start.y as usize..end.y as usize {
        let py = y * width;
        for x in start.x as usize..end.x as usize {
            let idx = py + x;
            buffer[idx] = 0;
        }
    }
    Ok(())
}
