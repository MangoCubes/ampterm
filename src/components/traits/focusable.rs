use crate::components::traits::renderable::Renderable;

pub trait Focusable: Renderable {
    fn set_enabled(&mut self, enable: bool);
}
