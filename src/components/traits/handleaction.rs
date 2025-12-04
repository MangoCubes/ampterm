use crate::{
    action::action::{Action, TargetedAction},
    components::traits::renderable::Renderable,
};

/// If a component has this trait, it indicates that the component may take in action as an
/// input to modify its state. Unlike keys, actions are not what user directly inputs, but is a
/// byproduct from the user's action unless explicitly configured.
pub trait HandleAction: Renderable {
    fn handle_action(&mut self, action: TargetedAction) -> Option<Action>;
}
