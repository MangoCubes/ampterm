use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{
    action::{
        useraction::{Common, Normal, UserAction, Visual},
        Action, FromPlayerWorker, PlayState,
    },
    app::Mode,
    components::{lib::visualstate::VisualState, traits::component::Component},
    osclient::response::getplaylist::Media,
};

pub struct Something {
    comp: Table<'static>,
    list: PlayState,
    tablestate: TableState,
    visual: VisualState,
}

/// There are 4 unique states each item in the list can have:
/// 1. Position relative to the item currently being played
///    This is indicated by a ▶ at the front, with played items using grey as primary colour
/// 2. Temporary selection
///    This is indicated by colour inversion
/// 3. Selection
///    This is indicated with green (darker green used if the item has already been played)
/// 4. Current cursor position
///    This is indicated with > and inversion
///
/// As a result, a dedicated list component has to be made
impl Something {
    pub fn new(playstate: PlayState) -> Self {
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));
        let len = playstate.items.len();
        Self {
            comp: Table::default(),
            list: playstate,
            tablestate,
            visual: VisualState::new(len),
        }
    }
    fn gen_items(ms: &[Media], style: Style) -> Vec<Row<'static>> {
        ms.iter()
            .map(|m| {
                Row::new(vec![" ".to_string(), m.title.clone(), m.get_fav_marker()]).style(style)
            })
            .collect()
    }
    fn gen_current_item(ms: &Media) -> Row<'static> {
        let current = Style::new().bold();
        Row::new(vec!["▶".to_string(), ms.title.clone(), ms.get_fav_marker()]).style(current)
    }
    fn gen_table(&self) -> Table<'static> {
        let len = self.list.items.len();
        let before = Style::new().fg(Color::DarkGray);
        let after = Style::new();
        let items = match self.list.index {
            // Current item is beyond the current playlist
            _ if len == self.list.index => Self::gen_items(&self.list.items, before),
            // Current item is the last item in the playlist
            idx if (len - 1) == self.list.index => {
                let mut list = Self::gen_items(&self.list.items[..idx], before);
                list.push(Self::gen_current_item(&self.list.items[idx]));
                list
            }
            // Current item is the first element in the playlist
            0 => {
                let mut list = Self::gen_items(&self.list.items[1..], after);
                list.insert(0, Self::gen_current_item(&self.list.items[0]));
                list
            }
            // Every other cases
            idx => {
                let mut list = Self::gen_items(&self.list.items[..idx], before);
                list.append(&mut Self::gen_items(&self.list.items[(idx + 1)..], after));
                list.insert(idx, Self::gen_current_item(&self.list.items[idx]));
                list
            }
        };
        let comp = Table::new(
            items,
            [Constraint::Max(1), Constraint::Min(0), Constraint::Max(1)].to_vec(),
        );
        comp.highlight_symbol(">")
            .row_highlight_style(Style::new().reversed())
    }
}

impl Component for Something {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.tablestate);
        Ok(())
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FromPlayerWorker(FromPlayerWorker::InQueue {
                play,
                vol,
                speed,
                pos,
            }) => {
                self.list = play;
                // Create cursor if it did not exist
                if self.tablestate.selected() == None {
                    self.tablestate.select(Some(0));
                }
                self.comp = self.gen_table();
                Ok(None)
            }
            Action::User(ua) => {
                let Some(cur_pos) = self.tablestate.selected() else {
                    return Ok(None);
                };

                let action = match ua {
                    UserAction::Common(local) => match local {
                        Common::Up => {
                            self.tablestate.select_previous();
                            Ok(None)
                        }
                        Common::Down => {
                            self.tablestate.select_next();
                            Ok(None)
                        }
                        Common::Top => {
                            self.tablestate.select_first();
                            Ok(None)
                        }
                        Common::Bottom => {
                            self.tablestate.select_last();
                            Ok(None)
                        }
                        _ => Ok(None),
                    },
                    UserAction::Normal(normal) => match normal {
                        Normal::SelectMode => {
                            self.visual.enable_visual(cur_pos, false);
                            Ok(Some(Action::ChangeMode(Mode::Visual)))
                        }
                        Normal::DeselectMode => {
                            self.visual.enable_visual(cur_pos, true);
                            Ok(Some(Action::ChangeMode(Mode::Visual)))
                        }
                        _ => Ok(None),
                    },
                    UserAction::Visual(visual) => match visual {
                        Visual::ExitSave => {
                            self.visual.disable_visual(cur_pos);
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                        Visual::ExitDiscard => {
                            self.visual.disable_visual_discard();
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                        _ => Ok(None),
                    },
                    _ => Ok(None),
                };
                self.comp = self.gen_table();
                action
            }
            _ => Ok(None),
        }
    }
}
