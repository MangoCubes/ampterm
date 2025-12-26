use crate::{
    action::action::Action, components::traits::renderable::Renderable,
    playerworker::player::FromPlayerWorker,
};

pub trait HandlePlayer: Renderable {
    fn handle_player(&mut self, pw: FromPlayerWorker) -> Option<Action>;
}
