use crate::components::traits::renderable::Renderable;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::Block,
    Frame,
};

pub struct Speed {
    speed: f32,
    coloured: Block<'static>,
}

impl Speed {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            coloured: Block::default().bg(Color::White),
        }
    }
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
    pub fn get_speed(&self) -> f32 {
        self.speed
    }
}

impl Renderable for Speed {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let border = Block::bordered()
            .border_style(Style::new().white())
            .title_top("SPD")
            .title_bottom(format!("{:.1}", self.speed));
        let coloured = if self.speed >= 2.0 {
            100 as u16
        } else {
            ((self.speed * 100.0).round() as u16) / 2
        };
        let div = Layout::vertical([
            Constraint::Percentage(100 - coloured),
            Constraint::Percentage(coloured),
        ]);
        let areas = div.split(border.inner(area));
        frame.render_widget(&self.coloured, areas[1]);

        frame.render_widget(border, area);
    }
}
