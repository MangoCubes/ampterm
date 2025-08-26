use crate::{
    action::{
        useraction::{Common, Normal, UserAction},
        Action,
    },
    components::{
        home::mainscreen::playlistlist::PlaylistListComps,
        traits::{component::Component, focusable::Focusable},
    },
    osclient::response::getplaylists::SimplePlaylist,
    playerworker::player::QueueLocation,
    queryworker::query::{QueryType, ToQueryWorker},
};
use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, List, ListState},
    Frame,
};

pub struct Loaded {
    comp: List<'static>,
    list: Vec<SimplePlaylist>,
    state: ListState,
    enabled: bool,
}

impl Loaded {
    fn select_playlist(&self) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
            let name = self.list[pos].name.clone();
            Some(Action::ToQueryWorker(ToQueryWorker::new(
                QueryType::GetPlaylist {
                    name: Some(name),
                    id: key,
                },
            )))
        } else {
            None
        }
    }

    fn gen_block(enabled: bool, title: &str) -> Block<'static> {
        let style = if enabled {
            Style::new().white()
        } else {
            Style::new().dark_gray()
        };
        let title = Span::styled(
            title.to_string(),
            if enabled {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        );
        Block::bordered().title(title).border_style(style)
    }

    fn gen_list(list: &Vec<SimplePlaylist>, enabled: bool) -> List<'static> {
        let items: Vec<String> = list.iter().map(|p| p.name.clone()).collect();
        List::new(items)
            .block(Self::gen_block(enabled, "Playlist"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">")
    }
    pub fn new(enabled: bool, list: Vec<SimplePlaylist>, state: ListState) -> Self {
        Self {
            enabled,
            comp: Self::gen_list(&list, enabled),
            list,
            state,
        }
    }
    pub fn add_to_queue(&mut self, ql: QueueLocation) -> Option<Action> {
        if let Some(pos) = self.state.selected() {
            let key = self.list[pos].id.clone();
        }
        None
    }
}

impl Component for Loaded {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::User(UserAction::Common(local)) => {
                match local {
                    Common::Up => {
                        self.state.select_previous();
                        Ok(None)
                    }
                    Common::Down => {
                        self.state.select_next();
                        Ok(None)
                    }
                    Common::Confirm => Ok(self.select_playlist()),
                    Common::Top => {
                        self.state.select_first();
                        Ok(None)
                    }
                    Common::Bottom => {
                        self.state.select_last();
                        Ok(None)
                    }
                    // TODO: Add horizontal text scrolling
                    _ => Ok(None),
                }
            }
            Action::User(UserAction::Normal(normal)) => {
                if let Normal::Add(pos) = normal {
                    Ok(self.add_to_queue(pos))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.state);
        Ok(())
    }
}

impl Focusable for Loaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            self.comp = Self::gen_list(&self.list, self.enabled);
        };
    }
}

impl PlaylistListComps for Loaded {}
