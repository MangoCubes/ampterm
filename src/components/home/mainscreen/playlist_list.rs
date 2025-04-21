use crate::{action::Action, components::Component};
use color_eyre::Result;
use ratatui::{layout::Rect, widgets::Block, Frame};
use tokio::sync::mpsc::UnboundedSender;

pub struct PlayListList {
    action_tx: UnboundedSender<Action>,
}

impl PlayListList {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        // action_tx.send(
        Self { action_tx }
    }
}

impl Component for PlayListList {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(Block::bordered(), area);
        Ok(())
    }
}
