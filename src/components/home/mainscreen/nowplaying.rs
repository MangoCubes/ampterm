mod playing;
mod stopped;

use color_eyre::Result;
use playing::Playing;
use ratatui::{layout::Rect, Frame};
use stopped::Stopped;

use crate::{
    action::{Action, FromPlayerWorker},
    components::traits::{component::Component, synccomp::SyncComp},
};

enum Comp {
    Playing(Playing),
    Stopped(Stopped),
}

pub struct NowPlaying {
    comp: Comp,
}

pub trait NowPlayingComponent: Component {}

impl NowPlaying {
    pub fn new() -> Self {
        Self {
            comp: Comp::Stopped(Stopped::new()),
        }
    }
}

impl Component for NowPlaying {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}

impl SyncComp for NowPlaying {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::FromPlayerWorker(FromPlayerWorker::InQueue {
            vol,
            speed,
            pos,
            play,
        }) = action
        {
            match play.items.get(play.index) {
                Some(p) => {
                    self.comp = Comp::Playing(Playing::new(
                        p.title.clone(),
                        p.artist.clone().unwrap_or("Unknown".to_string()),
                        p.album.clone().unwrap_or("Unknown".to_string()),
                        p.duration.unwrap_or(1),
                        vol,
                        speed,
                        pos,
                    ));
                }
                None => self.comp = Comp::Stopped(Stopped::new()),
            };
            Ok(None)
        } else {
            self.comp.update(action)
        }
    }
}
