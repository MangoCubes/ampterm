use crate::components::traits::renderable::Renderable;

pub trait OnTick: Renderable {
    fn on_tick(&mut self);
}
