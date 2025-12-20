use ratatui_image::Image;

use crate::action::action::Action;

pub struct ImageComp {
    url: String,
    image: Option<Image<'static>>,
}

impl ImageComp {
    pub fn new(url: String) -> (Self, Option<Action>) {
        (Self { url, image: None }, None)
    }
}
