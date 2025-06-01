use crate::components::Component;

pub trait Stateful<T>: Component {
    fn update_state(&mut self, state: T);
}
