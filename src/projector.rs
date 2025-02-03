use crate::{
    map_memory::MapMemory,
    units::{AdjustedPosition, Position, PositionTrait},
};

/// A Projector relates Positions to screen coordinates
/// two projectors are supported.
#[derive(Default, Clone)]
pub enum ProjectorType {
    /// Global is used for the regular map where Positions are latitude and longitude
    /// and are projected using mercator projection
    #[default]
    Global,
    /// Local is used for local coordinates were Positions are euclidean x and y values in
    /// some arbitrary units and the projection is an affine transformation
    Local,
}

pub struct Projector<'a> {
    clip_rect: egui::Rect,
    memory: &'a mut MapMemory,
    my_position: Position,
}

impl<'a> Projector<'a> {
    pub fn new(memory: &'a mut MapMemory, rect: egui::Rect, my_position: Position) -> Self {
        Self {
            clip_rect: rect,
            memory,
            my_position,
        }
    }

    pub fn project(&self, pos: Position) -> egui::Pos2 {
        let zoom = self.memory.zoom();
        match self.memory.projection_type {
            ProjectorType::Global => {
                let bm_pos = pos.global_bitmap_project(zoom);

                let map_center_projected_position = self
                    .memory
                    .center_mode
                    .global_position(self.my_position, zoom)
                    .global_bitmap_project(zoom);

                let shift = bm_pos - map_center_projected_position;

                self.clip_rect.center() + egui::Vec2::new(shift.x as f32, shift.y as f32)
            }
            ProjectorType::Local => {
                let bm_pos = pos.local_bitmap_project(zoom);

                let map_center_projected_position = self
                    .memory
                    .center_mode
                    .local_position(self.my_position, zoom)
                    .local_bitmap_project(zoom);

                let shift = bm_pos - map_center_projected_position;

                self.clip_rect.center() + egui::Vec2::new(shift.x as f32, shift.y as f32)
            }
        }
    }

    pub fn unproject(&self, screen_pos: egui::Pos2) -> Position {
        let zoom = self.memory.zoom();
        match self.memory.projection_type {
            ProjectorType::Global => {
                let center = self
                    .memory
                    .center_mode
                    .global_position(self.my_position, zoom);

                AdjustedPosition {
                    position: center,
                    offset: Default::default(),
                }
                .shift(-screen_pos.to_vec2())
                .global_unadjusted_position(zoom)
            }
            ProjectorType::Local => {
                let center = self
                    .memory
                    .center_mode
                    .local_position(self.my_position, zoom);

                AdjustedPosition {
                    position: center,
                    offset: Default::default(),
                }
                .shift(-screen_pos.to_vec2())
                .local_unadjusted_position(zoom)
            }
        }
    }

    pub fn scale_pixel_per_meter(&self, pos: Position) -> f32 {
        self.memory.scale_pixel_per_meter(pos)
    }
}
