use crate::{
    center::Center,
    units::Position,
    zoom::{InvalidZoom, Zoom},
};

/// State of the map widget which must persist between frames.
#[derive(Default, Clone)]
pub struct MapMemory {
    pub(crate) center_mode: Center,
    pub(crate) zoom: Zoom,
}

impl MapMemory {
    /// Returns the current zoom level
    pub fn zoom(&self) -> f64 {
        self.zoom.into()
    }

    pub fn zoom_in(&mut self) -> Result<(), InvalidZoom> {
        self.zoom.zoom_in()
    }

    /// Try to zoom out, returning `Err(InvalidZoom)` if already at minimum.
    pub fn zoom_out(&mut self) -> Result<(), InvalidZoom> {
        self.zoom.zoom_out()
    }

    /// Set exact zoom level
    pub fn set_zoom(&mut self, zoom: f64) -> Result<(), InvalidZoom> {
        self.zoom = Zoom::try_from(zoom)?;
        Ok(())
    }

    /// Returns exact position if map is detached (i.e. not following `my_position`),
    /// `None` otherwise.
    pub fn detached(&self) -> Option<Position> {
        self.center_mode.position()
    }

    /// Center exactly at the given position.
    pub fn center_at(&mut self, pos: Position) {
        self.center_mode = Center::Exact { pos };
    }

    /// Follow `my_position`.
    pub fn follow_my_position(&mut self) {
        self.center_mode = Center::MyPosition;
    }
}
