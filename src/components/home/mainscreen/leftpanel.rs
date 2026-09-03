mod playlistlist;

use crate::{
    action::action::Action,
    compid::CompID,
    components::{
        home::mainscreen::leftpanel::playlistlist::PlaylistList,
        traits::{
            copyable::Copyable,
            focusable::Focusable,
            handlekeyseq::{ComponentKeyHelp, KeySeqResult, PassKeySeq},
            handlequery::HandleQuery,
            renderable::Renderable,
        },
    },
    config::Config,
    queryworker::query::QueryStatus,
};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::Block,
    Frame,
};

enum Comp {
    PlaylistList(PlaylistList),
}

pub struct LeftPanel {
    comp: Comp,
    enabled: bool,
}

impl LeftPanel {
    pub fn new(config: Config, enabled: bool) -> (Self, Action) {
        let (comp, action) = PlaylistList::new(config, enabled);
        (
            Self {
                comp: Comp::PlaylistList(comp),
                enabled,
            },
            action,
        )
    }
    fn gen_block(&self) -> Block<'static> {
        let style = if self.enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title_text = match &self.comp {
            Comp::PlaylistList(_) => "Playlist View".to_string(),
        };

        let title = Span::styled(
            title_text,
            if self.enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }
}

impl Renderable for LeftPanel {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let block = self.gen_block();
        let inner = block.inner(area);
        frame.render_widget(block, area);
        match &mut self.comp {
            Comp::PlaylistList(pl) => pl.draw(frame, inner),
        }
    }
}

impl HandleQuery for LeftPanel {
    fn handle_query(&mut self, dest: CompID, ticket: usize, res: QueryStatus) -> Option<Action> {
        match &mut self.comp {
            Comp::PlaylistList(comp) => comp.handle_query(dest, ticket, res),
        }
    }
}

impl PassKeySeq for LeftPanel {
    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        match &self.comp {
            Comp::PlaylistList(comp) => comp.get_help(),
        }
    }
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        match &mut self.comp {
            Comp::PlaylistList(comp) => comp.handle_key_seq(keyseq),
        }
    }
}

impl Focusable for LeftPanel {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
        };
    }
}

impl Copyable for LeftPanel {
    fn get_copyable_item(&self) -> Option<String> {
        match &self.comp {
            Comp::PlaylistList(pl) => pl.get_copyable_item(),
        }
    }
}
