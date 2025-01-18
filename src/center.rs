use egui::{Response, Vec2};

use crate::{
    projector::{Projector, ProjectorTrait},
    units::{AdjustedPosition, Position},
};


/// Position at the map's center. Initially, the map follows `my_position` argument which typically
/// is meant to be fed by a GPS sensor or other geo-localization method. If user drags the map,
/// it becomes "detached" and stays this way until [`MapMemory::center_mode`] is changed back to
/// [`Center::MyPosition`].
#[derive(Clone, Default)]
pub(crate) enum Center {
    /// Centered at `my_position` argument of the [`Map::new()`] function.
    #[default]
    MyPosition,

    /// Centered exactly here
    Exact { pos: Position },

    /// Map is currently being dragged.
    Moving { pos: Position, direction: Vec2 },

    /// Map is currently moving due to inertia, and will slow down and stop after a short while.
    Inertia {
        pos: Position,
        direction: Vec2,
        amount: f32,
    },
}

impl Center {
    pub(crate) fn recalculate_drag(&mut self, response: &Response, my_position: Position) -> bool {
        if response.dragged_by(egui::PointerButton::Primary) {
            *self = Center::Moving {
                pos: self.position().unwrap_or(my_position),
                direction: response.drag_delta(),
            };
            true
        } else if response.drag_stopped() {
            if let Center::Moving { pos, direction } = &self {
                *self = Center::Inertia {
                    pos: pos.to_owned(),
                    direction: *direction,
                    amount: 1.0,
                };
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn update_movement(&mut self, projector: &Projector) -> bool {
        match self {
            Center::Moving { pos, direction } => {
                let delta = *direction;

                *pos = projector.shift(pos.to_owned(), delta);

                true
            }
            Center::Inertia {
                pos,
                direction,
                amount,
            } => {
                if amount <= &mut 0.0 {
                    *self = Center::Exact {
                        pos: pos.to_owned(),
                    }
                } else {
                    let delta = *direction * *amount;

                    projector.shift(pos.to_owned(), delta);
                };
                true
            }
            _ => false,
        }
    }

    /// Returns exact position if map is detached (i.e. not following `my_position`),
    /// `None` otherwise.
    pub(crate) fn detached(&self, projector: &impl ProjectorTrait) -> Option<Position> {
        self.adjusted_position().map(|p| projector.position(p))
    }

    /// Get the real position at the map's center.
    pub fn position(&self, my_position: Position, projector: &impl ProjectorTrait) -> Position {
        self.detached(projector).unwrap_or(my_position)
    }

    pub(crate) fn adjusted_position(&self) -> Option<AdjustedPosition> {
        match self {
            Center::MyPosition => None,
            Center::Exact { pos } | Center::Moving { pos, .. } | Center::Inertia { pos, .. } => {
                Some(pos.to_owned())
            }
        }
    }
}
