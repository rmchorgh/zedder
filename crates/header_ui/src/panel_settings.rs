use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use workspace::dock::DockPosition;

#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, Debug)]
pub struct PanelSettingsContent {
    /// Whether to show the panel button in the status bar.
    ///
    /// Default: true
    pub button: Option<bool>,
    /// Where to dock the panel.
    ///
    /// Default: left
    pub dock: Option<DockPosition>,
    /// Default width of the panel in pixels.
    ///
    /// Default: 240
    pub default_width: Option<f32>,
}
