use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Row, Table},
    Frame,
};

use crate::{
    action::{
        action::{Action, TargetedAction},
        localaction::HelpAction,
    },
    components::traits::{
        handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult},
        renderable::Renderable,
    },
    config::{keybindings::KeyBindings, Config},
};

pub struct Help {
    border: Block<'static>,
    table: Vec<(String, Table<'static>)>,
    current: Index,
    binds: KeyBindings<HelpAction>,
    global_table: Table<'static>,
}

enum Index {
    Global,
    Local(usize),
}

impl HandleKeySeq<HelpAction> for Help {
    fn handle_local_action(&mut self, action: HelpAction) -> KeySeqResult {
        match action {
            HelpAction::Left => {
                if self.table.len() != 0 {
                    self.current = match self.current {
                        Index::Global => Index::Local(self.table.len() - 1),
                        Index::Local(i) => {
                            if i == 0 {
                                Index::Global
                            } else {
                                Index::Local(i - 1)
                            }
                        }
                    }
                }
            }
            HelpAction::Right => {
                if self.table.len() != 0 {
                    self.current = match self.current {
                        Index::Global => Index::Local(0),
                        Index::Local(i) => {
                            if i == self.table.len() - 1 {
                                Index::Global
                            } else {
                                Index::Local(i + 1)
                            }
                        }
                    }
                }
            }
            HelpAction::Close => {
                return KeySeqResult::ActionNeeded(Action::Targeted(TargetedAction::ClosePopup));
            }
            _ => {}
        };
        self.border = match self.current {
            Index::Global => Self::gen_block("Global", self.table.len() + 1, self.table.len() + 1),
            Index::Local(i) => Self::gen_block(&self.table[i].0, i + 1, self.table.len() + 1),
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
    fn gen_section(comp: ComponentKeyHelp) -> (String, Table<'static>) {
        let rows: Vec<Row<'static>> = comp
            .bindings
            .into_iter()
            .map(|entry| Row::new(vec![entry.keyseq, entry.desc]))
            .collect();
        (
            comp.name,
            Table::new(rows, [Constraint::Max(40), Constraint::Min(1)]),
        )
    }

    pub fn display(&mut self, binds: Vec<ComponentKeyHelp>) {
        self.table = binds.into_iter().map(Self::gen_section).collect();
        self.current = if self.table.len() == 0 {
            Index::Global
        } else {
            Index::Local(0)
        };
        self.border = match self.current {
            Index::Global => Self::gen_block("Global", self.table.len() + 1, self.table.len() + 1),
            Index::Local(i) => Self::gen_block(&self.table[i].0, i + 1, self.table.len() + 1),
        };
    }

    pub fn new(config: Config) -> Self {
        Self {
            binds: config.local.help,
            border: Self::gen_block("", 1, 1),
            table: vec![],
            global_table: Self::gen_section(ComponentKeyHelp {
                bindings: config.global.to_help(),
                name: "Global".to_string(),
            })
            .1,
            current: Index::Local(0),
        }
    }

    fn gen_block(title: &str, current: usize, outof: usize) -> Block<'static> {
        let style = Style::new().white();
        let title_str = if outof != 1 {
            format!("({}/{}) Help for ← {} →", current, outof, title)
        } else {
            format!("Help for {}", title)
        };
        let title = Span::styled(title_str, Style::default().add_modifier(Modifier::BOLD));
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
        frame.render_widget(
            match &self.current {
                Index::Global => &self.global_table,
                Index::Local(idx) => &self.table[*idx].1,
            },
            self.border.inner(area),
        );
    }
}
