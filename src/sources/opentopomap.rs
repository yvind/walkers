use super::{Attribution, TileSource};
use crate::tiles::TileId;

/// <https://www.opentopomap.org/about>
pub struct OpenTopoMap;

impl TileSource for OpenTopoMap {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://c.tile.opentopomap.org/{}/{}/{}.png",
            tile_id.zoom, tile_id.x, tile_id.y
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "Mapdata: © OpenStreetMap contributers, SRTM | Map presentation: © OpenTopoMap (CC-BY-SA)",
            url: "https://www.opentopomap.org/about",
            logo_light: None,
            logo_dark: None,
        }
    }
}
