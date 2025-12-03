use ratatui::{
    prelude::Rect,
    style::{Style, Stylize},
    widgets::{List, ListState},
    Frame,
};

use crate::{
    components::traits::{
        handlekeyseq::{HandleKeySeq, KeySeqResult},
        renderable::Renderable,
    },
    config::{keybindings::KeyBindings, localkeybinds::LyricsAction, Config},
};

pub struct Unsynced {
    comp: List<'static>,
    state: ListState,
    config: Config,
}

impl Unsynced {
    pub fn new(config: Config, found: String) -> Self {
        let list: Vec<String> = found.lines().map(|line| line.to_string()).collect();
        let mut default = ListState::default();
        default.select_first();
        Self {
            comp: Self::gen_list(list),
            state: default,
            config,
        }
    }
    fn gen_list(list: Vec<String>) -> List<'static> {
        List::new(list)
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
            .scroll_padding(1)
    }
}

impl Renderable for Unsynced {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
    }
}

impl HandleKeySeq<LyricsAction> for Unsynced {
    fn handle_local_action(&mut self, action: LyricsAction) -> KeySeqResult {
        match action {
            LyricsAction::Up => self.state.select_previous(),
            LyricsAction::Down => self.state.select_next(),
            LyricsAction::Top => self.state.select_first(),
            LyricsAction::Bottom => self.state.select_last(),
        }
        KeySeqResult::NoActionNeeded
    }

    fn get_keybinds(&self) -> &KeyBindings<LyricsAction> {
        &self.config.local.lyrics
    }
}
