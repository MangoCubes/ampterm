use crate::{
    action::action::Action, compid::CompID, components::traits::renderable::Renderable,
    queryworker::query::QueryStatus,
};

pub trait HandleQuery: Renderable {
    fn handle_query(&mut self, dest: CompID, ticket: usize, res: QueryStatus) -> Option<Action>;
}
