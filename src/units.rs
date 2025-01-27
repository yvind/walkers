use std::f64::consts::PI;
use std::ops::{Add, Sub};

use crate::TileId;

/// Position in some coordinates, either latitude and longitude or local projected coordinate system.
#[derive(Debug, Clone, Copy, Default)]
pub struct Position(geo_types::Point);

impl Position {
    /// Construct from latitude and longitude.
    pub fn from_lat_lon(lat: f64, lon: f64) -> Self {
        Self(geo_types::Point::new(lon, lat))
    }

    /// Construct from longitude and latitude. Note that it is common standard to write coordinates
    /// starting with the latitude instead (e.g. `51.104465719934176, 17.075169894118684` is
    /// the [Wrocław's zoo](https://zoo.wroclaw.pl/en/)).
    pub fn from_lon_lat(lon: f64, lat: f64) -> Self {
        Self(geo_types::Point::new(lon, lat))
    }

    pub fn new(x: f64, y: f64) -> Self {
        Self(geo_types::Point::new(x, y))
    }

    pub fn x(&self) -> f64 {
        self.0.x()
    }

    pub fn y(&self) -> f64 {
        self.0.y()
    }

    pub fn lat(&self) -> f64 {
        self.0.y()
    }

    pub fn lon(&self) -> f64 {
        self.0.x()
    }

    pub(crate) fn mercator_normalized(&self) -> (f64, f64) {
        // Project into Mercator (cylindrical map projection).
        let x = self.lon().to_radians();
        let y = self.lat().to_radians().tan().asinh();

        // Scale both x and y to 0-1 range.
        let x = (1. + (x / PI)) / 2.;
        let y = (1. - (y / PI)) / 2.;
        (x, y)
    }

    pub(crate) fn global_bitmap_project(&self, zoom: f64) -> Pixel {
        let total_pixels = crate::total_pixels(zoom);
        let (x, y) = self.mercator_normalized();
        Pixel::new(x * total_pixels, y * total_pixels)
    }

    pub(crate) fn local_bitmap_project(&self, zoom: f64) -> Pixel {
        let units_per_point = crate::local_units_per_point(zoom);

        Pixel::new(self.x() / units_per_point, -self.y() / units_per_point)
    }

    pub(crate) fn tile_id(&self, mut zoom: u8, source_tile_size: u32) -> TileId {
        let (x, y) = self.mercator_normalized();

        // Some sources provide larger tiles, effectively bundling e.g. 4 256px tiles in one
        // 512px one. Walkers uses 256px internally, so we need to adjust the zoom level.
        zoom -= (source_tile_size as f64 / crate::TILE_SIZE as f64).log2() as u8;

        // Map that into a big bitmap made out of web tiles.
        let number_of_tiles = 2u32.pow(zoom as u32) as f64;
        let x = (x * number_of_tiles).floor() as u32;
        let y = (y * number_of_tiles).floor() as u32;

        TileId { x, y, zoom }
    }
}

impl From<geo_types::Point> for Position {
    fn from(value: geo_types::Point) -> Self {
        Self(value)
    }
}

impl From<Position> for geo_types::Point {
    fn from(value: Position) -> Self {
        value.0
    }
}

/// Location projected on the screen or an abstract bitmap.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pixel(geo_types::Point);

impl Pixel {
    pub(crate) fn new(x: f64, y: f64) -> Pixel {
        Pixel(geo_types::Point::new(x, y))
    }

    pub(crate) fn x(&self) -> f64 {
        self.0.x()
    }

    pub(crate) fn y(&self) -> f64 {
        self.0.y()
    }

    pub(crate) fn global_bitmap_unproject(&self, zoom: f64) -> Position {
        let number_of_pixels = crate::total_pixels(zoom);

        let lon = self.x();
        let lon = lon / number_of_pixels;
        let lon = (lon * 2. - 1.) * PI;
        let lon = lon.to_degrees();

        let lat = self.y();
        let lat = lat / number_of_pixels;
        let lat = (-lat * 2. + 1.) * PI;
        let lat = lat.sinh().atan().to_degrees();

        Position::from_lon_lat(lon, lat)
    }

    pub(crate) fn local_bitmap_unproject(&self, zoom: f64) -> Position {
        let units_per_point = crate::local_units_per_point(zoom);

        Position::new(self.x() * units_per_point, -self.y() * units_per_point)
    }
}

impl From<egui::Pos2> for Pixel {
    fn from(value: egui::Pos2) -> Self {
        Pixel::new(value.x as f64, value.y as f64)
    }
}

impl From<Pixel> for egui::Pos2 {
    fn from(val: Pixel) -> Self {
        egui::Pos2::new(val.x() as f32, val.y() as f32)
    }
}

impl From<egui::Vec2> for Pixel {
    fn from(value: egui::Vec2) -> Self {
        Pixel::new(value.x as f64, value.y as f64)
    }
}

impl From<Pixel> for egui::Vec2 {
    fn from(val: Pixel) -> Self {
        egui::Vec2::new(val.x() as f32, val.y() as f32)
    }
}

impl Add<egui::Vec2> for Pixel {
    type Output = Self;

    fn add(self, rhs: egui::Vec2) -> Self::Output {
        Self::new(self.x() + rhs.x as f64, self.y() + rhs.y as f64)
    }
}

impl Sub<egui::Vec2> for Pixel {
    type Output = Self;

    fn sub(self, rhs: egui::Vec2) -> Self::Output {
        Self::new(self.x() - rhs.x as f64, self.y() - rhs.y as f64)
    }
}

impl Add for Pixel {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl Sub for Pixel {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

/// [`Position`] alone is not able to represent detached (e.g. after map gets dragged) position
/// due to insufficient accuracy.
#[derive(Debug, Clone)]
pub(crate) struct AdjustedPosition {
    /// Base geographical position.
    pub(crate) position: Position,

    /// Offset in pixels.
    pub(crate) offset: Pixel,
}

impl AdjustedPosition {
    pub(crate) fn new(position: Position, offset: Pixel) -> Self {
        Self { position, offset }
    }

    pub(crate) fn shift(self, shift: egui::Vec2) -> Self {
        Self {
            position: self.position,
            offset: self.offset + shift,
        }
    }

    pub(crate) fn global_unadjusted_position(&self, zoom: f64) -> Position {
        (self.position.global_bitmap_project(zoom) - self.offset).global_bitmap_unproject(zoom)
    }

    pub(crate) fn local_unadjusted_position(&self, zoom: f64) -> Position {
        (self.position.local_bitmap_project(zoom) - self.offset).local_bitmap_unproject(zoom)
    }

    pub(crate) fn global_zero_offset(self, zoom: f64) -> Self {
        Self {
            position: self.global_unadjusted_position(zoom),
            offset: Default::default(),
        }
    }

    pub(crate) fn local_zero_offset(self, zoom: f64) -> Self {
        Self {
            position: self.local_unadjusted_position(zoom),
            offset: Default::default(),
        }
    }
}

impl From<Position> for AdjustedPosition {
    fn from(position: Position) -> Self {
        Self {
            position,
            offset: Default::default(),
        }
    }
}
