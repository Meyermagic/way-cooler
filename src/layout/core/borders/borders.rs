use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use rustwlc::{Geometry, Size, WlcOutput};
use rustwlc::render::{calculate_stride};
use cairo::{ImageSurface, Format};

use ::registry;
use ::render::{Color, Renderable};

/// The borders of a container.
///
/// This type just deals with rendering,
#[derive(Clone)]
pub struct Borders {
    /// The surface that contains the bytes we give to wlc to draw.
    surface: ImageSurface,
    /// The geometry where the buffer is written.
    ///
    /// Should correspond with the geometry of the container.
    pub geometry: Geometry,
    /// The output where the buffer is written to.
    output: WlcOutput,
    /// The specific color these borders should be colored.
    ///
    /// If unspecified, the default is used
    color: Option<Color>,
    /// The title used for the title border.
    title: String
}

impl Renderable for Borders {
    fn new(mut geometry: Geometry, output: WlcOutput) -> Option<Self> {
        let thickness = Borders::thickness();
        if thickness == 0 {
            return None
        }
        // Add the thickness to the geometry.
        geometry.origin.x -= thickness as i32;
        geometry.origin.y -= thickness as i32;
        geometry.origin.y -= Borders::title_offset() as i32;
        geometry.size.w += thickness;
        geometry.size.h += thickness;
        geometry.size.h += Borders::title_offset();
        let Size { w, h } = geometry.size;
        let stride = calculate_stride(w) as i32;
        let data: Vec<u8> = iter::repeat(0).take(h as usize * stride as usize).collect();
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer,
                                                    drop_data,
                                                    Format::ARgb32,
                                                    w as i32,
                                                    h as i32,
                                                    stride);
        Some(Borders {
            // TODO Make configurable
            title: "Hello World!".into(),
            surface: surface,
            geometry: geometry,
            output: output,
            color: None
        })
    }

    fn get_surface(&mut self) -> &mut ImageSurface {
        &mut self.surface
    }

    fn get_geometry(&self) -> Geometry {
        self.geometry
    }

    fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    fn get_output(&self) -> WlcOutput {
        self.output
    }

    /// Updates/Creates the underlying geometry for the surface/buffer.
    ///
    /// This causes a reallocation of the buffer, do not call this
    /// in a tight loop unless you want memory fragmentation and
    /// bad performance.
    fn reallocate_buffer(mut self, mut geometry: Geometry) -> Option<Self>{
        // Add the thickness to the geometry.
        let thickness = Borders::thickness();
        if thickness == 0 {
            return None;
        }
        geometry.origin.x -= thickness as i32;
        geometry.origin.y -= thickness as i32;
        geometry.origin.y -= Borders::title_offset() as i32;
        geometry.size.w += thickness;
        geometry.size.h += thickness;
        geometry.size.h += Borders::title_offset();
        //warn!("Allocate @ {:#?}", geometry);
        let Size { w, h } = geometry.size;
        if w == self.geometry.size.w && h == self.geometry.size.h {
            return Some(self);
        }
        let stride = calculate_stride(w) as i32;
        let data: Vec<u8> = iter::repeat(0).take(h as usize * stride as usize).collect();
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer,
                                                    drop_data,
                                                    Format::ARgb32,
                                                    w as i32,
                                                    h as i32,
                                                    stride);
        self.geometry = geometry;
        self.surface = surface;
        Some(self)
    }
}

impl Borders {
    pub fn title(&self) -> &str {
        &*self.title
    }

    /// Gets the offset from the title border.
    ///
    /// E.g: this offset is the y value that the top border (that doesn't
    /// doesn't contain the title text) should start at.
    pub fn title_offset() -> u32 {
        // TODO make this a more appropriate value, maybe configurable w/ pango
        50
    }

    pub fn thickness() -> u32 {
        registry::get_data("border_size")
            .map(registry::RegistryGetData::resolve).and_then(|(_, data)| {
                Ok(data.as_f64().map(|num| {
                    if num <= 0.0 {
                        0u32
                    } else {
                        num as u32
                    }
                }).unwrap_or(0u32))
            }).unwrap_or(0u32)
    }

    /// Fetches the default color from the registry.
    ///
    /// If the value is unset, black borders are returned.
    pub fn default_color() -> Color {
        let val = registry::get_data("border_color")
            .map(registry::RegistryGetData::resolve).and_then(|(_, data)| {
                Ok(data.as_f64().map(|num| {
                    if num <= 0.0 {
                        0u32
                    } else {
                        num as u32
                    }
                }).unwrap_or(0u32))
            }).unwrap_or(0u32);
        val.into()
    }

    /// Gets the active border color, if one is set.
    pub fn active_color() -> Option<Color> {
        let val = registry::get_data("active_border_color")
            .map(registry::RegistryGetData::resolve).and_then(|(_, data)| {
                Ok(data.as_f64().map(|num| {
                    if num <= 0.0 {
                        0u32
                    } else {
                        num as u32
                    }
                }).unwrap_or(0u32))
            }).ok();
        val.map(|c| c.into())
    }

    /// Gets the color for these borders.
    ///
    /// If a specific one is unset, then the default color is returned.
    pub fn color(&self) -> Color {
        self.color.unwrap_or_else(Borders::default_color)
    }

    /// Sets or clears the specific color for these borders.
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color
    }

    pub fn get_output(&self) -> WlcOutput {
        self.output
    }
}

impl Debug for Borders {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Borders")
            .field("geometry", &self.geometry as &Debug)
            .finish()
    }
}

impl PartialEq for Borders {
    fn eq(&self, other: &Borders) -> bool {
        self.geometry == other.geometry
    }
}

impl Eq for Borders {}

unsafe impl Send for Borders {}
unsafe impl Sync for Borders {}

#[allow(dead_code)]
fn drop_data(_: Box<[u8]>) { }
