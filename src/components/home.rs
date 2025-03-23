pub mod login;

use color_eyre::Result;
use login::Login;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config, tui::Event};

pub struct Home {
    action_tx: UnboundedSender<Action>,
    component: Box<dyn Component>,
}

impl Home {
    pub fn new(action_tx: UnboundedSender<Action>, config: Config) -> Self {
        let auth = config.auth;
        let comp = match auth {
            Some(creds) => todo!(),
            None => Login::new(action_tx.clone()),
        };
        Self {
            action_tx,
            component: Box::new(comp),
        }
    }
}

impl Component for Home {
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        let action = match event {
            Event::Key(key_event) => self.handle_key_event(key_event)?,
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        if let Some(action) = self.component.handle_events(event.clone())? {
            self.action_tx.send(action)?;
        }
        Ok(action)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Err(err) = self.component.draw(frame, area) {
            self.action_tx.send(Action::Error(err.to_string()))?;
        }
        Ok(())
    }
}
