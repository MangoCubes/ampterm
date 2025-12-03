use crate::{action::action::Mode, components::traits::renderable::Renderable};

pub trait HandleMode: Renderable {
    fn handle_mode(&mut self, mode: Mode);
}
