use crate::components::traits::renderable::Renderable;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::Block,
    Frame,
};

pub struct Volume {
    volume: f32,
    coloured: Block<'static>,
}

impl Volume {
    pub fn new(volume: f32) -> Self {
        Self {
            volume,
            coloured: Block::default().bg(Color::White),
        }
    }
    pub fn set_volume(&mut self, speed: f32) {
        self.volume = speed;
    }
}

impl Renderable for Volume {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let border = Block::bordered()
            .border_style(Style::new().white())
            .title_top("VOL")
            .title_bottom(format!("{:03}", (self.volume * 100.0).round() as u16));
        let coloured = if self.volume >= 1.0 {
            100 as u16
        } else {
            (self.volume * 100.0).round() as u16
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
