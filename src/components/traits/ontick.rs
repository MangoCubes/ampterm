use crate::components::traits::component::Component;

pub trait OnTick: Component {
    fn on_tick(&mut self);
}
