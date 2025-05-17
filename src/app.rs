use ratatui::Terminal;
use ratatui::backend::Backend;
use crate::ui::draw_ui;

#[derive(Copy, Clone)]
pub enum Tab {
    Overview,
    Cluster,
    Logs,
    Actions,
}

pub struct App {
    pub current_tab: Tab,
    // сюда можно добавить: status_data, logs, cluster_nodes и т.п.
}

impl App {
    pub fn new() -> Self {
        App {
            current_tab: Tab::Overview,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| draw_ui::<B>(frame, self))?;

            if crossterm::event::poll(std::time::Duration::from_millis(200))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    use crossterm::event::{KeyCode, KeyEventKind};

                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('1') => self.current_tab = Tab::Overview,
                            KeyCode::Char('2') => self.current_tab = Tab::Cluster,
                            KeyCode::Char('3') => self.current_tab = Tab::Logs,
                            KeyCode::Char('4') => self.current_tab = Tab::Actions,
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
