mod global_map;
mod local_map;

pub use global_map::Map;
pub use local_map::LocalMap;

use crate::Projector;

/// Plugins allow drawing custom shapes on the map. After implementing this trait for your type,
/// you can add it to the map with [`Map::with_plugin`]
pub trait Plugin {
    /// Function called at each frame.
    ///
    /// The provided [`Ui`] has its [`Ui::max_rect`] set to the full rect that was allocated
    /// by the map widget. Implementations should typically use the provided [`Projector`] to
    /// compute target screen coordinates and use one of the various egui methods to draw at these
    /// coordinates instead of relying on [`Ui`] layout system.
    ///
    /// The provided [`Response`] is the response of the map widget itself and can be used to test
    /// if the mouse is hovering or clicking on the map.
    fn run(self: Box<Self>, ui: &mut egui::Ui, response: &egui::Response, projector: &Projector);
}

/// Wrap your map in the Maps enum to be able to return
/// the two different maps from different if branches.
/// Interactable in the exact same way as the "naked" maps
pub enum Maps<'a, 'b, 'c> {
    Map(Map<'a, 'b, 'c>),
    LocalMap(LocalMap<'a, 'b>),
}

impl egui::Widget for Maps<'_, '_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            Maps::Map(map) => map.ui(ui),
            Maps::LocalMap(local_map) => local_map.ui(ui),
        }
    }
}

impl<'b> Maps<'_, 'b, '_> {
    pub fn with_plugin(self, plugin: impl Plugin + 'b) -> Self {
        match self {
            Maps::Map(map) => Maps::Map(map.with_plugin(plugin)),
            Maps::LocalMap(local_map) => Maps::LocalMap(local_map.with_plugin(plugin)),
        }
    }

    /// Set whether map should perform zoom gesture.
    ///
    /// Zoom is typically triggered by the mouse wheel while holding <kbd>ctrl</kbd> key on native
    /// and web, and by pinch gesture on Android.
    pub fn zoom_gesture(self, enabled: bool) -> Self {
        match self {
            Maps::Map(map) => Maps::Map(map.zoom_gesture(enabled)),
            Maps::LocalMap(local_map) => Maps::LocalMap(local_map.zoom_gesture(enabled)),
        }
    }

    /// Set whether map should perform drag gesture.
    pub fn drag_gesture(self, enabled: bool) -> Self {
        match self {
            Maps::Map(map) => Maps::Map(map.drag_gesture(enabled)),
            Maps::LocalMap(local_map) => Maps::LocalMap(local_map.drag_gesture(enabled)),
        }
    }

    /// Change how far to zoom in/out.
    /// Default value is 2.0
    pub fn zoom_speed(self, speed: f64) -> Self {
        match self {
            Maps::Map(map) => Maps::Map(map.zoom_speed(speed)),
            Maps::LocalMap(local_map) => Maps::LocalMap(local_map.zoom_speed(speed)),
        }
    }

    /// Set whether to enable double click primary mouse button to zoom
    pub fn double_click_to_zoom(self, enabled: bool) -> Self {
        match self {
            Maps::Map(map) => Maps::Map(map.double_click_to_zoom(enabled)),
            Maps::LocalMap(local_map) => Maps::LocalMap(local_map.double_click_to_zoom(enabled)),
        }
    }

    /// Set whether to enable double click secondary mouse button to zoom out
    pub fn double_click_to_zoom_out(self, enabled: bool) -> Self {
        match self {
            Maps::Map(map) => Maps::Map(map.double_click_to_zoom_out(enabled)),
            Maps::LocalMap(local_map) => {
                Maps::LocalMap(local_map.double_click_to_zoom_out(enabled))
            }
        }
    }

    /// Sets the zoom behaviour
    ///
    /// When enabled zoom is done with mouse wheel while holding <kbd>ctrl</kbd> key on native
    /// and web. Panning is done with mouse wheel without <kbd>ctrl</kbd> key
    ///
    /// When disabled, zooming can be done without holding <kbd>ctrl</kbd> key
    /// but panning with mouse wheel is disabled
    ///
    /// Has no effect on Android
    pub fn zoom_with_ctrl(self, enabled: bool) -> Self {
        match self {
            Maps::Map(map) => Maps::Map(map.zoom_with_ctrl(enabled)),
            Maps::LocalMap(local_map) => Maps::LocalMap(local_map.zoom_with_ctrl(enabled)),
        }
    }
}
