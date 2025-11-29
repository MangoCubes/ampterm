use crate::{action::globalaction::InternalAction, components::traits::renderable::Renderable};

pub trait HandleInternal: Renderable {
    fn handle_internal(&mut self, action: InternalAction);
}
