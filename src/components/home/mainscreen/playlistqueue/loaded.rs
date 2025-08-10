use crate::{
    action::{
        getplaylist::{FullPlaylist, Media},
        getplaylists::PlaylistID,
        Action, Dir, Local, Normal,
    },
    components::Component,
    focusable::Focusable,
    playerworker::player::{QueueLocation, ToPlayerWorker},
    queryworker::query::ToQueryWorker,
    statelib::visual::Visual,
};
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Row},
    Frame,
};

use super::PlaylistQueueComps;

pub struct Loaded<'a> {
    name: String,
    playlistid: PlaylistID,
    visual: Visual<'a, Media>,
    enabled: bool,
}

impl<'a> Loaded<'a> {
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
    fn select_music(&self, playpos: QueueLocation) -> Option<Action> {
        let item = self.visual.get_current();
        Some(Action::ToPlayerWorker(ToPlayerWorker::AddToQueue {
            pos: playpos,
            music: vec![item.clone()],
        }))
    }
    pub fn new(name: String, list: FullPlaylist, enabled: bool) -> Self {
        fn convert<'a>(media: &Media) -> Row<'a> {
            Row::new(vec![media.title.clone()])
        }
        Self {
            name,
            playlistid: list.id,
            visual: Visual::new(list.entry, convert, [Constraint::Min(0)].to_vec()),
            enabled,
        }
    }
}

impl<'a> Component for Loaded<'a> {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Local(local) => {
                match local {
                    Local::Move(dir) => {
                        match dir {
                            Dir::Up => self.visual.select_previous(),
                            Dir::Down => self.visual.select_next(),
                            _ => {}
                        };
                        Ok(None)
                    }
                    Local::Confirm => todo!(),
                    Local::Cancel => todo!(),
                    Local::Top => {
                        self.visual.select_first();
                        Ok(None)
                    }
                    Local::Bottom => {
                        self.visual.select_last();
                        Ok(None)
                    }
                    Local::Refresh => Ok(Some(Action::ToQueryWorker(ToQueryWorker::GetPlaylist {
                        name: Some(self.name.to_string()),
                        id: self.playlistid.clone(),
                    }))),
                    Local::ResetState => todo!(),
                    Local::Help => todo!(),
                }
                // match action {
                //     Action::Add(loc) => Ok(self.select_music(loc)),
                //     Action::ExitVisualModeDiscard => {
                //         self.visual.disable_visual(false);
                //         Ok(None)
                //     }
                //     Action::ExitVisualModeSave => {
                //         self.visual.disable_visual(true);
                //         Ok(None)
                //     }
                //     Action::ResetState => {
                //         self.visual.reset();
                //         Ok(None)
                //     }
                //     // TODO: Add horizontal text scrolling
                //     _ => Ok(None),
                // }
            }
            Action::Normal(normal) => match normal {
                Normal::SelectMode => {
                    self.visual.enable_visual(false);
                    Ok(None)
                }
                Normal::DeselectMode => {
                    self.visual.enable_visual(true);
                    Ok(None)
                }
                Normal::Add(queue_location) => Ok(self.select_music(queue_location)),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let border = Self::gen_block(self.enabled, &self.name);
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.visual.draw(frame, inner)
    }
}

impl<'a> Focusable for Loaded<'a> {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            if !self.enabled {
                self.visual.disable_visual(false);
            }
        };
    }
}

impl<'a> PlaylistQueueComps for Loaded<'a> {}
