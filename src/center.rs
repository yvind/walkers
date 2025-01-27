use egui::{Response, Vec2};

use crate::units::{AdjustedPosition, Position};

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
    Exact { pos: AdjustedPosition },

    /// Map is currently being dragged.
    Moving {
        pos: AdjustedPosition,
        direction: Vec2,
    },

    /// Map is currently moving due to inertia, and will slow down and stop after a short while.
    Inertia {
        pos: AdjustedPosition,
        direction: Vec2,
        amount: f32,
    },
}

impl Center {
    pub(crate) fn recalculate_drag(&mut self, response: &Response, my_position: Position) -> bool {
        if response.dragged_by(egui::PointerButton::Primary) {
            *self = Center::Moving {
                pos: self
                    .get_adjusted_position()
                    .unwrap_or(AdjustedPosition::new(my_position, Default::default())),
                direction: response.drag_delta(),
            };
            true
        } else if response.drag_stopped() {
            if let Center::Moving { pos, direction } = &self {
                *self = Center::Inertia {
                    pos: pos.clone(),
                    direction: *direction,
                    amount: 1.0,
                };
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn update_movement(&mut self) -> bool {
        match self {
            Center::Moving { pos, direction } => {
                let delta = *direction;

                *pos = pos.clone().shift(delta);

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

                    *pos = pos.clone().shift(delta);
                    *amount -= 0.03;
                };
                true
            }
            _ => false,
        }
    }

    pub(crate) fn get_adjusted_position(&self) -> Option<AdjustedPosition> {
        match self {
            Center::MyPosition => None,
            Center::Exact { pos } | Center::Moving { pos, .. } | Center::Inertia { pos, .. } => {
                Some(pos.to_owned())
            }
        }
    }

    /// Shift position by given number of pixels, if detached.
    pub(crate) fn shift(self, offset: Vec2) -> Self {
        match self {
            Center::MyPosition => Center::MyPosition,
            Center::Exact { pos } => Center::Exact {
                pos: pos.shift(offset),
            },
            Center::Moving { pos, direction } => Center::Moving {
                pos: pos.shift(offset),
                direction,
            },
            Center::Inertia {
                pos,
                direction,
                amount,
            } => Center::Inertia {
                pos: pos.shift(offset),
                direction,
                amount,
            },
        }
    }

    pub(crate) fn global_zero_offset(self, zoom: f64) -> Center {
        match self {
            Center::MyPosition => Center::MyPosition,
            Center::Exact { pos } => Center::Exact {
                pos: pos.global_zero_offset(zoom),
            },
            Center::Moving { pos, direction } => Center::Moving {
                pos: pos.global_zero_offset(zoom),
                direction,
            },
            Center::Inertia {
                pos,
                direction,
                amount,
            } => Center::Inertia {
                pos: pos.global_zero_offset(zoom),
                direction,
                amount,
            },
        }
    }

    pub(crate) fn local_zero_offset(self, zoom: f64) -> Center {
        match self {
            Center::MyPosition => Center::MyPosition,
            Center::Exact { pos } => Center::Exact {
                pos: pos.local_zero_offset(zoom),
            },
            Center::Moving { pos, direction } => Center::Moving {
                pos: pos.local_zero_offset(zoom),
                direction,
            },
            Center::Inertia {
                pos,
                direction,
                amount,
            } => Center::Inertia {
                pos: pos.local_zero_offset(zoom),
                direction,
                amount,
            },
        }
    }

    pub(crate) fn global_position(&self, my_position: Position, zoom: f64) -> Position {
        match self.get_adjusted_position() {
            Some(adj_pos) => adj_pos.global_unadjusted_position(zoom),
            None => my_position,
        }
    }

    pub(crate) fn local_position(&self, my_position: Position, zoom: f64) -> Position {
        match self.get_adjusted_position() {
            Some(adj_pos) => adj_pos.local_unadjusted_position(zoom),
            None => my_position,
        }
    }
}
