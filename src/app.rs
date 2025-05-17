use crate::components::cluster::Cluster;
use crate::components::logs::Logs;
use crate::components::overview::Overview;
use crate::patroni::patroni::Patroni;
use crate::ui;
use crate::ui::UI;
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::cmp::PartialEq;

#[derive(Copy, Clone)]
pub enum Tab {
    Overview,
    Cluster,
    Logs,
    Actions,
}

pub struct App {
    pub current_tab: Tab,
    pub ui: UI,
    pub log_selected: usize,
    pub log_scroll: u16,
    pub log_focus_right: bool,
}

impl PartialEq for Tab {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl App {
    pub fn new() -> Self {
        let patroni_srv = Patroni::new("127.0.0.1:8008".to_string());
        let overview_srv = Overview::new(patroni_srv.clone());
        let cluster_srv = Cluster::new(patroni_srv.clone());
        let logs_srv = Logs::new();

        App {
            current_tab: Tab::Overview,
            ui: UI::new(overview_srv, cluster_srv, logs_srv),
            log_selected: 0,
            log_scroll: 0,
            log_focus_right: false,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| self.ui.draw_ui::<B>(frame, self))?;

            if crossterm::event::poll(std::time::Duration::from_millis(1000))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    use crossterm::event::{KeyCode, KeyEventKind};

                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('1') => self.current_tab = Tab::Overview,
                            KeyCode::Char('2') => self.current_tab = Tab::Cluster,
                            KeyCode::Char('3') => self.current_tab = Tab::Logs,
                            KeyCode::Char('4') => self.current_tab = Tab::Actions,
                            KeyCode::Right if self.current_tab == Tab::Logs => {
                                self.log_focus_right = true;
                            }
                            KeyCode::Left if self.current_tab == Tab::Logs => {
                                self.log_focus_right = false;
                            }
                            KeyCode::Down | KeyCode::Char('j')
                                if self.current_tab == Tab::Logs && !self.log_focus_right =>
                            {
                                if self.log_selected < ui::SERVICES.len() - 1 {
                                    self.log_selected += 1;
                                    self.log_scroll = 0;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k')
                                if self.current_tab == Tab::Logs && !self.log_focus_right =>
                            {
                                if self.log_selected > 0 {
                                    self.log_selected -= 1;
                                    self.log_scroll = 0;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j')
                                if self.current_tab == Tab::Logs && self.log_focus_right =>
                            {
                                self.log_scroll += 1;
                            }
                            KeyCode::Up | KeyCode::Char('k')
                                if self.current_tab == Tab::Logs && self.log_focus_right =>
                            {
                                if self.log_scroll > 0 {
                                    self.log_scroll -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
