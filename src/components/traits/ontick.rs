use crate::components::traits::fullcomp::FullComp;

pub trait OnTick: FullComp {
    fn on_tick(&mut self);
}
