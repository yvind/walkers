use crate::{
    center::Center,
    projector::ProjectorType,
    units::{AdjustedPosition, Position},
    zoom::{InvalidZoom, Zoom},
};

/// State of the map widget which must persist between frames.
#[derive(Default, Clone)]
pub struct MapMemory {
    pub(crate) projection_type: ProjectorType,
    pub(crate) center_mode: Center,
    pub(crate) zoom: Zoom,
}

impl MapMemory {
    /// Returns the current zoom level
    pub fn zoom(&self) -> f64 {
        self.zoom.into()
    }

    pub fn zoom_in(&mut self) -> Result<(), InvalidZoom> {
        let zoom = self.zoom();
        self.center_mode = match self.projection_type {
            ProjectorType::Global => self.center_mode.clone().global_zero_offset(zoom),
            ProjectorType::Local => self.center_mode.clone().local_zero_offset(zoom),
        };
        self.zoom.zoom_in()
    }

    /// Try to zoom out, returning `Err(InvalidZoom)` if already at minimum.
    pub fn zoom_out(&mut self) -> Result<(), InvalidZoom> {
        let zoom = self.zoom();
        self.center_mode = match self.projection_type {
            ProjectorType::Global => self.center_mode.clone().global_zero_offset(zoom),
            ProjectorType::Local => self.center_mode.clone().local_zero_offset(zoom),
        };
        self.zoom.zoom_out()
    }

    /// Set exact zoom level
    pub fn set_zoom(&mut self, new_zoom: f64) -> Result<(), InvalidZoom> {
        let zoom = self.zoom();
        self.center_mode = match self.projection_type {
            ProjectorType::Global => self.center_mode.clone().global_zero_offset(zoom),
            ProjectorType::Local => self.center_mode.clone().local_zero_offset(zoom),
        };
        self.zoom = Zoom::try_from(new_zoom)?;
        Ok(())
    }

    /// Center exactly at the given position.
    pub fn center_at(&mut self, pos: Position) {
        self.center_mode = Center::Exact {
            pos: AdjustedPosition::new(pos, Default::default()),
        };
    }

    /// Follow `my_position`.
    pub fn follow_my_position(&mut self) {
        self.center_mode = Center::MyPosition;
    }
}
