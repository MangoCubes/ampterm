use crate::components::traits::component::Component;

pub trait Focusable: Component {
    fn set_enabled(&mut self, enable: bool);
}
