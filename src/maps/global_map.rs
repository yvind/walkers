use egui::{PointerButton, Response, Sense, Ui, UiBuilder, Vec2, Widget};

use crate::{
    center::Center,
    map_memory::MapMemory,
    projector::{Projector, ProjectorType},
    tiles::flood_fill_tiles,
    units::{AdjustedPosition, Position, PositionTrait},
    Plugin, Tiles,
};

/// The actual map widget. Instances are to be created on each frame, as all necessary state is
/// stored in [`Tiles`] and [`MapMemory`].
pub struct Map<'a, 'b, 'c> {
    tiles: Option<&'c mut dyn Tiles>,
    memory: &'a mut MapMemory,
    my_position: Position,
    plugins: Vec<Box<dyn Plugin + 'b>>,

    zoom_gesture_enabled: bool,
    drag_gesture_enabled: bool,
    zoom_speed: f64,
    double_click_to_zoom: bool,
    double_click_to_zoom_out: bool,
    zoom_with_ctrl: bool,
}

impl<'a, 'b, 'c> Map<'a, 'b, 'c> {
    pub fn new(
        tiles: Option<&'c mut dyn Tiles>,
        memory: &'a mut MapMemory,
        my_position: Position,
    ) -> Self {
        memory.projection_type = ProjectorType::Global;

        Self {
            tiles,
            memory,
            my_position,
            plugins: Vec::default(),
            zoom_gesture_enabled: true,
            drag_gesture_enabled: true,
            zoom_speed: 2.0,
            double_click_to_zoom: false,
            double_click_to_zoom_out: false,
            zoom_with_ctrl: true,
        }
    }

    pub fn with_plugin(mut self, plugin: impl Plugin + 'b) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Set whether map should perform zoom gesture.
    ///
    /// Zoom is typically triggered by the mouse wheel while holding <kbd>ctrl</kbd> key on native
    /// and web, and by pinch gesture on Android.
    pub fn zoom_gesture(mut self, enabled: bool) -> Self {
        self.zoom_gesture_enabled = enabled;
        self
    }

    /// Set whether map should perform drag gesture.
    pub fn drag_gesture(mut self, enabled: bool) -> Self {
        self.drag_gesture_enabled = enabled;
        self
    }

    /// Change how far to zoom in/out.
    /// Default value is 2.0
    pub fn zoom_speed(mut self, speed: f64) -> Self {
        self.zoom_speed = speed;
        self
    }

    /// Set whether to enable double click primary mouse button to zoom
    pub fn double_click_to_zoom(mut self, enabled: bool) -> Self {
        self.double_click_to_zoom = enabled;
        self
    }

    /// Set whether to enable double click secondary mouse button to zoom out
    pub fn double_click_to_zoom_out(mut self, enabled: bool) -> Self {
        self.double_click_to_zoom_out = enabled;
        self
    }

    /// Sets the zoom behavior
    ///
    /// When enabled zoom is done with mouse wheel while holding <kbd>ctrl</kbd> key on native
    /// and web. Panning is done with mouse wheel without <kbd>ctrl</kbd> key
    ///
    /// When disabled, zooming can be done without holding <kbd>ctrl</kbd> key
    /// but panning with mouse wheel is disabled
    ///
    /// Has no effect on Android
    pub fn zoom_with_ctrl(mut self, enabled: bool) -> Self {
        self.zoom_with_ctrl = enabled;
        self
    }
}

impl Map<'_, '_, '_> {
    /// Handle zoom and drag inputs, and recalculate everything accordingly.
    /// Returns `false` if no gesture handled.
    fn handle_gestures(&mut self, ui: &mut Ui, response: &Response) -> bool {
        let mut zoom_delta = ui.input(|input| input.zoom_delta()) as f64;

        if self.double_click_to_zoom
            && ui.ui_contains_pointer()
            && response.double_clicked_by(PointerButton::Primary)
        {
            zoom_delta = 2.0;
        }

        if self.double_click_to_zoom_out
            && ui.ui_contains_pointer()
            && response.double_clicked_by(PointerButton::Secondary)
        {
            zoom_delta = 0.0;
        }

        if !self.zoom_with_ctrl && zoom_delta == 1.0 {
            // We only use the raw scroll values, if we are zooming without ctrl,
            // and zoom_delta is not already over/under 1.0 (eg. a ctrl + scroll event or a pinch zoom)
            // These values seem to correspond to the same values as one would get in `zoom_delta()`
            zoom_delta = ui.input(|input| (1.0 + input.smooth_scroll_delta.y / 200.0)) as f64
        };

        let mut changed = false;

        // Zooming and dragging need to be exclusive, otherwise the map will get dragged when
        // pinch gesture is used.
        if !(0.99..=1.01).contains(&zoom_delta)
            && ui.ui_contains_pointer()
            && self.zoom_gesture_enabled
        {
            // Displacement of mouse pointer relative to widget center
            let offset = response.hover_pos().map(|p| p - response.rect.center());

            let pos = self
                .memory
                .center_mode
                .global_position(self.my_position, self.memory.zoom());

            // While zooming, we want to keep the location under the mouse pointer fixed on the
            // screen. To achieve this, we first move the location to the widget's center,
            // then adjust zoom level, finally move the location back to the original screen
            // position.
            if let Some(offset) = offset {
                self.memory.center_mode = Center::Exact {
                    pos: (AdjustedPosition::from(pos).shift(-offset))
                        .global_zero_offset(self.memory.zoom()),
                };
            }

            // Shift by 1 because of the values given by zoom_delta(). Multiple by zoom_speed(defaults to 2.0),
            // because then it felt right with both mouse wheel, and an Android phone.
            self.memory
                .zoom
                .zoom_by((zoom_delta - 1.) * self.zoom_speed);

            // Recalculate the AdjustedPosition's offset, since it gets invalidated by zooming.
            self.memory.center_mode = self
                .memory
                .center_mode
                .clone()
                .global_zero_offset(self.memory.zoom());

            if let Some(offset) = offset {
                self.memory.center_mode = self.memory.center_mode.clone().shift(offset);
            }

            changed = true;
        } else if self.drag_gesture_enabled {
            changed = self
                .memory
                .center_mode
                .recalculate_drag(response, self.my_position);
        }

        // Only enable panning with mouse_wheel if we are zooming with ctrl. But always allow touch devices to pan
        let panning_enabled = ui.input(|i| i.any_touches()) || self.zoom_with_ctrl;

        if ui.ui_contains_pointer() && panning_enabled {
            // Panning by scrolling, e.g. two-finger drag on a touchpad:
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
            if scroll_delta != Vec2::ZERO {
                let pos = self
                    .memory
                    .center_mode
                    .global_position(self.my_position, self.memory.zoom());
                self.memory.center_mode = Center::Exact {
                    pos: AdjustedPosition::from(pos).shift(scroll_delta),
                };
            }
        }

        changed
    }
}

impl Widget for Map<'_, '_, '_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (rect, mut response) =
            ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

        let mut moved = self.handle_gestures(ui, &response);
        moved |= self.memory.center_mode.update_movement();

        if moved {
            response.mark_changed();
            ui.ctx().request_repaint();
        }

        let zoom = self.memory.zoom();
        let map_center = self
            .memory
            .center_mode
            .global_position(self.my_position, zoom);
        let painter = ui.painter().with_clip_rect(rect);

        if let Some(tiles) = self.tiles {
            let mut meshes = Default::default();
            flood_fill_tiles(
                painter.clip_rect(),
                map_center.tile_id(self.memory.zoom.round(), tiles.tile_size()),
                map_center.global_bitmap_project(zoom),
                zoom,
                tiles,
                &mut meshes,
            );

            for shape in meshes.drain().filter_map(|(_, mesh)| mesh) {
                painter.add(shape);
            }
        }

        let projector = Projector::new(self.memory, rect, self.my_position);
        for (idx, plugin) in self.plugins.into_iter().enumerate() {
            let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect).id_salt(idx));
            plugin.run(&mut child_ui, &response, &projector);
        }

        response
    }
}
