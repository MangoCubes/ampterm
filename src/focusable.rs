use crate::components::Component;

pub trait Focusable: Component {
    fn set_enabled(&mut self, enable: bool);
}
