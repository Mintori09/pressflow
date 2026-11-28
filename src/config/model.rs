use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::config::key_binding::KeyBinding;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
    pub keys: IndexMap<KeyBinding, ChordNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutMode {
    #[default]
    Classic,
    WhichKey,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HudAnchor {
    Top,
    Bottom,
    Left,
    Right,
    Center,
    TopLeft,
    TopRight,
    #[default]
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheatsheetIdleMode {
    Indicator,
    #[default]
    Invisible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub color_scheme: ColorScheme,
    pub corner_radius: f32,
    pub border_width: f32,
    pub window_opacity: f32,
    pub columns: u32,
    pub layout: LayoutMode,
    pub hud_anchor: HudAnchor,
    pub hud_margin: u32,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            corner_radius: 12.0,
            border_width: 1.0,
            window_opacity: 0.85,
            columns: 4,
            layout: LayoutMode::default(),
            hud_anchor: HudAnchor::default(),
            hud_margin: 16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub autostart: bool,
    pub cheatsheet_delay_ms: u64,
    pub cheatsheet_hidden: bool,
    pub cheatsheet_idle_mode: CheatsheetIdleMode,
    pub watchdog_timeout_secs: u64,
    pub key_repeat_suppression: bool,
    pub auto_close: bool,
    pub fetch_url_favicons: bool,
    pub show_app_icons: bool,
    pub auto_reload_config: bool,
    pub default_terminal: String,
    pub allow_external_url_triggers: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            autostart: true,
            cheatsheet_delay_ms: 1000,
            cheatsheet_hidden: false,
            cheatsheet_idle_mode: CheatsheetIdleMode::default(),
            watchdog_timeout_secs: 10,
            key_repeat_suppression: true,
            auto_close: true,
            fetch_url_favicons: false,
            show_app_icons: true,
            auto_reload_config: true,
            default_terminal: "auto".into(),
            allow_external_url_triggers: false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionOptions {
    pub repeatable: bool,
    pub ask: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ActionTuple {
    WithOptions(String, String, ActionOptions),
    Simple(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "ActionTuple")]
pub struct ActionNode {
    pub command: String,
    pub label: String,
    pub options: ActionOptions,
}

impl ActionNode {
    pub fn has_input(&self) -> bool {
        self.command.contains("{input}") || self.command.contains("{input:")
    }
}

impl From<ActionTuple> for ActionNode {
    fn from(tuple: ActionTuple) -> Self {
        match tuple {
            ActionTuple::WithOptions(command, label, options) => Self {
                command,
                label,
                options,
            },
            ActionTuple::Simple(command, label) => Self {
                command,
                label,
                options: ActionOptions::default(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupNode {
    pub label: String,
    #[serde(flatten)]
    pub children: IndexMap<KeyBinding, ChordNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChordNode {
    Action(ActionNode),
    Group(GroupNode),
}
