#![doc = include_str!("../README.md")]

mod center;
mod download;
pub mod extras;
mod io;
mod map_memory;
mod maps;
mod projector;
pub mod sources;
mod tiles;
mod units;
mod zoom;

pub use download::{HeaderValue, HttpOptions};
pub use maps::{LocalMap, Map, Maps, Plugin};

pub use map_memory::MapMemory;
pub use projector::Projector;
pub use tiles::{HttpTiles, Texture, TextureWithUv, TileId, Tiles};
pub use units::{pos_from_lat_lon, pos_from_lon_lat, Position};
pub use zoom::InvalidZoom;

const TILE_SIZE: u32 = 256;

// zoom level   tile coverage  number of tiles  tile size(*) in degrees
// 0            1 tile         1 tile           360° x 170.1022°
// 1            2 × 2 tiles    4 tiles          180° x 85.0511°
// 2            4 × 4 tiles    16 tiles         90° x [variable]
/// Zoom specifies how many pixels are in the whole map. For example, zoom 0 means that the whole
/// map is just one 256x256 tile, zoom 1 means that it is 2x2 tiles, and so on.
pub(crate) fn total_pixels(zoom: f64) -> f64 {
    2f64.powf(zoom) * (crate::TILE_SIZE as f64)
}

pub(crate) fn local_units_per_point(zoom: f64) -> f64 {
    2_f64.powf(16. - zoom) // at zoom 16 (default) the scale is a unit per point
}
