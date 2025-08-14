mod playing;
mod stopped;

use color_eyre::Result;
use playing::Playing;
use ratatui::{layout::Rect, Frame};
use stopped::Stopped;

use crate::{
    action::{Action, FromPlayerWorker},
    components::traits::component::Component,
};

pub struct NowPlaying {
    comp: Box<dyn NowPlayingComponent>,
}

pub trait NowPlayingComponent: Component {}

impl NowPlaying {
    pub fn new() -> Self {
        Self {
            comp: Box::new(Stopped::new()),
        }
    }
}

impl Component for NowPlaying {
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
                    self.comp = Box::new(Playing::new(
                        p.title.clone(),
                        p.artist.clone().unwrap_or("Unknown".to_string()),
                        p.album.clone().unwrap_or("Unknown".to_string()),
                        p.duration.unwrap_or(1),
                        vol,
                        speed,
                        pos,
                    ));
                }
                None => self.comp = Box::new(Stopped::new()),
            };
            Ok(None)
        } else {
            self.comp.update(action)
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        self.comp.draw(frame, area)
    }
}
