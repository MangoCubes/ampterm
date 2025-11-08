use crate::components::traits::fullcomp::FullComp;

pub trait Focusable: FullComp {
    fn set_enabled(&mut self, enable: bool);
}
