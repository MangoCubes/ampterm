use crate::{
    action::action::Action,
    compid::CompID,
    components::{
        lib::centered::Centered,
        traits::{handlequery::HandleQuery, renderable::Renderable},
    },
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getcoverart::CoverID, QueryStatus, ResponseType, ToQueryWorker},
    },
};
use image::DynamicImage;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};

enum State {
    Loading(usize, Centered),
    Loaded(StatefulProtocol),
    NotFound(Centered),
    Error(Centered),
}

pub struct ImageComp {
    state: State,
    picker: Picker,
}

impl ImageComp {
    pub fn make_query(id: CoverID) -> ToQueryWorker {
        ToQueryWorker::new(HighLevelQuery::GetCover(id))
    }
    pub fn new(coverid: Option<String>, mut picker: Picker) -> (Self, Option<Action>) {
        picker.set_background_color([0, 0, 0, 0]);
        if let Some(id) = coverid {
            let query = Self::make_query(CoverID(id));
            (
                Self {
                    picker,
                    state: State::Loading(
                        query.ticket,
                        Centered::new(vec!["Loading cover...".to_string()]),
                    ),
                },
                Some(Action::ToQuery(query)),
            )
        } else {
            (
                Self {
                    picker,
                    state: State::NotFound(Centered::new(vec!["No cover art".to_string()])),
                },
                None,
            )
        }
    }

    pub fn wait_for(&mut self, ticket: usize) {
        self.state = State::Loading(ticket, Centered::new(vec!["Loading cover...".to_string()]));
    }

    pub fn unset_image(&mut self) {
        self.state = State::NotFound(Centered::new(vec!["No cover art".to_string()]));
    }

    pub fn set_image(&mut self, ticket: usize, d: Result<DynamicImage, String>) {
        if let State::Loading(t, _) = &self.state {
            if *t != ticket {
                return;
            }
        } else {
            return;
        }
        let Ok(decoded) = d else {
            self.state = State::Error(Centered::new(vec![
                "Failed to".to_string(),
                "load image!".to_string(),
            ]));
            return;
        };
        let image = self.picker.new_resize_protocol(decoded);
        self.state = State::Loaded(image);
    }
}

impl HandleQuery for ImageComp {
    fn handle_query(&mut self, _dest: CompID, ticket: usize, res: QueryStatus) -> Option<Action> {
        if let QueryStatus::Finished(ResponseType::GetCover(d)) = res {
            self.set_image(ticket, d);
        };
        None
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
