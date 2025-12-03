use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Row, Table},
    Frame,
};

use crate::{
    components::traits::{
        handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult},
        renderable::Renderable,
    },
    config::{keybindings::KeyBindings, localkeybinds::HelpAction, Config},
};

pub struct Help {
    border: Block<'static>,
    table: Vec<Table<'static>>,
    current: usize,
    binds: KeyBindings<HelpAction>,
}

impl HandleKeySeq<HelpAction> for Help {
    fn handle_local_action(&mut self, action: HelpAction) -> KeySeqResult {
        match action {
            HelpAction::Left => {
                if self.current == 0 {
                    self.current = self.table.len() - 1;
                } else {
                    self.current -= 1;
                }
            }
            HelpAction::Right => {
                if self.current == self.table.len() - 1 {
                    self.current = 0;
                } else {
                    self.current += 1;
                }
            }
            _ => {}
        };
        KeySeqResult::NoActionNeeded
    }

    fn get_keybinds(&self) -> &KeyBindings<HelpAction> {
        &self.binds
    }

    fn get_name(&self) -> &str {
        "Help"
    }
}

impl Help {
    pub fn display(&mut self, binds: Vec<ComponentKeyHelp>) {
        fn gen_section(comp: ComponentKeyHelp) -> Table<'static> {
            let mut rows: Vec<Row<'static>> = comp
                .bindings
                .into_iter()
                .map(|entry| Row::new(vec![entry.keyseq, entry.desc]))
                .collect();
            rows.insert(
                0,
                Row::new(vec![format!("Help for ← {} →", comp.name), "".to_string()]),
            );
            Table::new(rows, [Constraint::Max(40), Constraint::Min(1)])
        }

        self.table = binds.into_iter().map(gen_section).collect();
    }
    pub fn new(config: Config) -> Self {
        Self {
            binds: config.local.help,
            border: Self::gen_block(),
            table: vec![],
            current: 0,
        }
    }

    fn gen_block() -> Block<'static> {
        let style = Style::new().white();
        let title = Span::styled("Help", Style::default().add_modifier(Modifier::BOLD));
        Block::bordered().title(title).border_style(style)
    }
}

impl Renderable for Help {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([Constraint::Percentage(80)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(80)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.border, area);
        if let Some(t) = self.table.get(self.current) {
            frame.render_widget(t, self.border.inner(area));
        } else if let Some(t) = self.table.get(0) {
            frame.render_widget(t, self.border.inner(area));
        } else {
            panic!("Failed to get help!");
        }
    }
}
