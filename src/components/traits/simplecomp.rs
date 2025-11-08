use crate::{action::Action, components::traits::renderable::Renderable};

pub trait SimpleComp: Renderable {
    fn update(&mut self, action: Action) {
        let _ = action; // to appease clippy
    }
}
