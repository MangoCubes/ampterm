use std::io::Cursor;

use crate::{
    action::action::{Action, QueryAction},
    components::{lib::centered::Centered, traits::renderable::Renderable},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getcoverart::CoverID, ToQueryWorker},
    },
};
use bytes::Bytes;
use image::ImageReader;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};

enum State {
    Loading(usize, Centered),
    Loaded(StatefulProtocol),
    NotFound(Centered),
    Error(Centered),
}

pub struct ImageComp {
    state: State,
}

impl ImageComp {
    pub fn new(coverid: Option<String>) -> (Self, Option<Action>) {
        if let Some(id) = coverid {
            let query = ToQueryWorker::new(HighLevelQuery::GetCover(CoverID(id)));
            (
                Self {
                    state: State::Loading(
                        query.ticket,
                        Centered::new(vec!["Loading cover...".to_string()]),
                    ),
                },
                Some(Action::Query(QueryAction::ToQueryWorker(query))),
            )
        } else {
            (
                Self {
                    state: State::NotFound(Centered::new(vec!["No cover art".to_string()])),
                },
                None,
            )
        }
    }

    pub fn set_image(&mut self, d: Result<Bytes, String>) {
        let bytes = match d {
            Ok(b) => b,
            Err(s) => {
                self.state = State::Error(Centered::new(vec![s]));
                return;
            }
        };
        let Ok(reader) = ImageReader::new(Cursor::new(bytes)).with_guessed_format() else {
            self.state = State::Error(Centered::new(vec![
                "Failed to determine image format!".to_string()
            ]));
            return;
        };
        let Ok(decoded) = reader.decode() else {
            self.state = State::Error(Centered::new(vec![
                "Failed to".to_string(),
                "decode image!".to_string(),
            ]));
            return;
        };
        let Ok(mut picker) = Picker::from_query_stdio() else {
            self.state = State::Error(Centered::new(vec![
                "Failed to get".to_string(),
                "terminal image".to_string(),
                "capabilities!".to_string(),
            ]));
            return;
        };
        picker.set_background_color([0, 0, 0, 0]);
        let image = picker.new_resize_protocol(decoded);
        self.state = State::Loaded(image);
    }
}

impl Renderable for ImageComp {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        match &mut self.state {
            State::NotFound(b) | State::Loading(_, b) | State::Error(b) => {
                b.draw(frame, area);
            }
            State::Loaded(stateful_protocol) => {
                let image = StatefulImage::default();
                frame.render_stateful_widget(image, area, stateful_protocol);
            }
        }
    }
}
