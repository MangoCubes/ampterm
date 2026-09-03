use serde::Deserialize;

fn default_true() -> bool {
    true
}

fn default_copy_command() -> String {
    "wl-copy".to_string()
}

#[derive(Clone, Debug, Deserialize)]
pub struct BehaviourConfig {
    /// Automatically changes the currently focused items
    #[serde(default = "default_true")]
    pub auto_focus: bool,
    /// Show some internal tasks in the tasks view that may not be very interesting
    #[serde(default)]
    pub show_internal_tasks: bool,
    /// Command to run when copying text into clipboard
    #[serde(default = "default_copy_command")]
    pub copy_command: String,
}

impl Default for BehaviourConfig {
    fn default() -> Self {
        Self {
            auto_focus: true,
            show_internal_tasks: false,
            copy_command: "wl-copy".to_string(),
        }
    }
}
