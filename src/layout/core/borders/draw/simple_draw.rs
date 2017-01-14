
use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point};
use rustwlc::render::{write_pixels, wlc_pixel_format};
use cairo::{Context, ImageSurface, Format, Operator, Status, SolidPattern};
use cairo::prelude::{SurfaceExt};

use super::super::borders::Borders;
use super::base::{BaseDraw, Drawable, DrawErr};
use super::color::Color;

/// Draws the borders simply, with a solid color at the same thickness.
pub struct SimpleDraw {
    base: BaseDraw,
    color: Color
}

impl SimpleDraw {
    pub fn new(base: BaseDraw, color: Color) -> Self {
        SimpleDraw {
            base: base,
            color: color
        }
    }
}

impl Drawable for SimpleDraw {
    fn draw(self, mut border_g: Geometry) -> Result<Borders, DrawErr> {
        let original_g = self.base.borders().geometry;
        border_g.origin.x -= self.base.borders().thickness as i32 / 2;
        border_g.origin.y -= self.base.borders().thickness as i32 / 2;
        border_g.size.w += self.base.borders().thickness;
        border_g.size.h += self.base.borders().thickness;

        error!("Drawing at {:#?}", border_g);
        let mut base = self.base;
        base.clear();
        base.set_color_source(self.color);
        // This draws _relatively_ compared to the rest of Way Cooler
        // Thus, 0,0 is top left of the buffer, not of the entire window.
        base.rectangle(border_g.origin.x as f64,
                       border_g.origin.y as f64,
                       border_g.size.w as f64,
                       border_g.size.h as f64);
        base = try!(base.check_cairo());
        base.fill();
        base = try!(base.check_cairo());
        Ok(base.finish(original_g))
    }
}
