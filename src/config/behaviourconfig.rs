use serde::Deserialize;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct BehaviourConfig {
    /// Automatically changes the currently focused items
    #[serde(default = "default_true")]
    pub auto_focus: bool,
    /// Show some internal tasks in the tasks view that may not be very interesting
    #[serde(default)]
    pub show_internal_tasks: bool,
}

impl Default for BehaviourConfig {
    fn default() -> Self {
        Self {
            auto_focus: true,
            show_internal_tasks: false,
        }
    }
}
