use serde::Deserialize;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct BehaviourConfig {
    /// Automatically changes the currently focused items
    #[serde(default = "default_true")]
    pub auto_focus: bool,
    /// Show some internal tasks in the tasks view that may not be very interesting
    #[serde(default)]
    pub show_internal_tasks: bool,
    /// If true, then if the item that is currently being played is deleted, it is automatically
    /// skipped. If false, then instead of deleting the item, the item is marked as Temporary (the
    /// item removes itself from the play queue once it is finished).
    #[serde(default = "default_true")]
    pub skip_on_delete: bool,
}
