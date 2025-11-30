use crate::{
    action::action::{Action, QueryAction},
    components::traits::renderable::Renderable,
};

pub trait HandleQuery: Renderable {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action>;
}
