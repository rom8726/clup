use crate::config::Config;
use crate::patroni::patroni::Patroni;
use crate::services::actions::{Action, ActionsService};
use crate::services::cluster::ClusterService;
use crate::services::logs::LogsService;
use crate::services::overview::OverviewService;
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
    pub config: Config,

    // Actions tab state
    pub action_selected: usize,
    pub action_confirmation: bool,
    pub action_confirmation_yes: bool,
    pub action_target_node: String,
    pub action_error: Option<String>,
}

impl PartialEq for Tab {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl App {
    pub fn new(config: Config) -> Self {
        let patroni_client = Patroni::new(config.patroni_addr.clone());
        let overview_service = OverviewService::new(patroni_client.clone(), config.clone());
        let cluster_service = ClusterService::new(patroni_client.clone());
        let logs_service = LogsService::new();

        App {
            current_tab: Tab::Overview,
            ui: UI::new(overview_service, cluster_service, logs_service, config.clone()),
            log_selected: 0,
            log_scroll: 0,
            log_focus_right: false,
            config,

            // Initialize Actions tab state
            action_selected: 0,
            action_confirmation: false,
            action_confirmation_yes: false,
            action_target_node: String::new(),
            action_error: None,
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
                                let services_len = self.config.services_list().len();
                                if self.log_selected < services_len - 1 {
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
                            // Actions tab navigation
                            KeyCode::Down | KeyCode::Char('j') if self.current_tab == Tab::Actions => {
                                if !self.action_confirmation {
                                    let actions_len = Action::all().len();
                                    if self.action_selected < actions_len - 1 {
                                        self.action_selected += 1;
                                    }
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') if self.current_tab == Tab::Actions => {
                                if !self.action_confirmation {
                                    if self.action_selected > 0 {
                                        self.action_selected -= 1;
                                    }
                                }
                            }
                            KeyCode::Char('n') if self.current_tab == Tab::Actions => {
                                // Select next node for node-specific actions
                                if !self.action_confirmation {
                                    let actions = Action::all();
                                    if self.action_selected < actions.len() {
                                        let action = &actions[self.action_selected];
                                        let cluster_info = self.ui.actions_service.get_cluster_info();

                                        match action {
                                            Action::Switchover => {
                                                // Find next replica node
                                                let replicas: Vec<_> = cluster_info.members
                                                    .iter()
                                                    .filter(|n| n.role != "leader")
                                                    .collect();

                                                if !replicas.is_empty() {
                                                    if self.action_target_node.is_empty() {
                                                        self.action_target_node = replicas[0].name.clone();
                                                    } else {
                                                        // Find current node index and select next
                                                        let current_idx = replicas
                                                            .iter()
                                                            .position(|n| n.name == self.action_target_node);

                                                        if let Some(idx) = current_idx {
                                                            let next_idx = (idx + 1) % replicas.len();
                                                            self.action_target_node = replicas[next_idx].name.clone();
                                                        } else {
                                                            self.action_target_node = replicas[0].name.clone();
                                                        }
                                                    }
                                                }
                                            }
                                            Action::Restart | Action::Reinitialize => {
                                                // Select next node
                                                if !cluster_info.members.is_empty() {
                                                    if self.action_target_node.is_empty() {
                                                        self.action_target_node = cluster_info.members[0].name.clone();
                                                    } else {
                                                        // Find current node index and select next
                                                        let current_idx = cluster_info.members
                                                            .iter()
                                                            .position(|n| n.name == self.action_target_node);

                                                        if let Some(idx) = current_idx {
                                                            let next_idx = (idx + 1) % cluster_info.members.len();
                                                            self.action_target_node = cluster_info.members[next_idx].name.clone();
                                                        } else {
                                                            self.action_target_node = cluster_info.members[0].name.clone();
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            KeyCode::Enter if self.current_tab == Tab::Actions => {
                                let actions = Action::all();
                                if self.action_selected < actions.len() {
                                    let action = &actions[self.action_selected];

                                    if self.action_confirmation {
                                        // Handle confirmation
                                        if self.action_confirmation_yes {
                                            // Execute the action
                                            let result = match action {
                                                Action::Switchover => {
                                                    let cluster_info = self.ui.actions_service.get_cluster_info();
                                                    self.ui.actions_service.switchover(
                                                        &cluster_info.leader_node_name,
                                                        &self.action_target_node
                                                    )
                                                }
                                                Action::Restart => {
                                                    self.ui.actions_service.restart_node(&self.action_target_node)
                                                }
                                                Action::Reinitialize => {
                                                    self.ui.actions_service.reinitialize_node(&self.action_target_node)
                                                }
                                                Action::PauseCluster => {
                                                    self.ui.actions_service.pause_cluster()
                                                }
                                                Action::ResumeCluster => {
                                                    self.ui.actions_service.resume_cluster()
                                                }
                                            };

                                            // Handle result
                                            if let Err(e) = result {
                                                self.action_error = Some(e.to_string());
                                            }

                                            // Reset confirmation state
                                            self.action_confirmation = false;
                                        } else {
                                            // User selected "No"
                                            self.action_confirmation = false;
                                        }
                                    } else if action.is_destructive() {
                                        // Show confirmation dialog for destructive actions
                                        self.action_confirmation = true;
                                        self.action_confirmation_yes = false;

                                        // Initialize target node if needed
                                        if (matches!(action, Action::Switchover | Action::Restart | Action::Reinitialize)) 
                                            && self.action_target_node.is_empty() {
                                            let cluster_info = self.ui.actions_service.get_cluster_info();
                                            if !cluster_info.members.is_empty() {
                                                if matches!(action, Action::Switchover) {
                                                    // For switchover, select first replica
                                                    let replicas: Vec<_> = cluster_info.members
                                                        .iter()
                                                        .filter(|n| n.role != "leader")
                                                        .collect();

                                                    if !replicas.is_empty() {
                                                        self.action_target_node = replicas[0].name.clone();
                                                    }
                                                } else {
                                                    // For other actions, select first node
                                                    self.action_target_node = cluster_info.members[0].name.clone();
                                                }
                                            }
                                        }
                                    } else {
                                        // Execute non-destructive actions immediately
                                        let result = match action {
                                            Action::ResumeCluster => {
                                                self.ui.actions_service.resume_cluster()
                                            }
                                            _ => Ok(()) // Should not happen
                                        };

                                        // Handle result
                                        if let Err(e) = result {
                                            self.action_error = Some(e.to_string());
                                        }
                                    }
                                }
                            }
                            KeyCode::Left | KeyCode::Right if self.current_tab == Tab::Actions && self.action_confirmation => {
                                // Toggle between Yes and No in confirmation dialog
                                self.action_confirmation_yes = !self.action_confirmation_yes;
                            }
                            KeyCode::Esc if self.current_tab == Tab::Actions => {
                                // Clear error or cancel confirmation
                                if self.action_error.is_some() {
                                    self.action_error = None;
                                } else if self.action_confirmation {
                                    self.action_confirmation = false;
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
