use crate::app::{App, Tab};
use crate::components::overview::{OverviewData, Overview};
use ratatui::backend::Backend;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::*,
    widgets::*,
};
use crate::components::cluster::Cluster;
use crate::components::logs::Logs;

pub const SERVICES: [&str; 4] = ["patroni", "haproxy", "pgbouncer", "keepalived"];

pub struct UI {
    pub overview_srv: Overview,
    pub cluster_srv: Cluster,
    pub logs_srv: Logs,
}

impl UI {
    pub fn new(overview_srv: Overview, cluster_srv: Cluster, logs_srv: Logs) -> Self {
        UI {
            overview_srv,
            cluster_srv,
            logs_srv,
        }
    }

    pub fn draw_ui<B: Backend>(&self, frame: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.size());

        let tab_titles = ["1: Overview", "2: Cluster", "3: Logs", "4: Actions"];
        let tabs = Tabs::new(
            tab_titles
                .iter()
                .cloned()
                .map(Line::from)
                .collect::<Vec<Line>>(),
        )
            .block(Block::default().borders(Borders::ALL).title("Navigation"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(app.current_tab as usize);
        frame.render_widget(tabs, chunks[0]);

        match app.current_tab {
            Tab::Overview => self.draw_overview::<B>(frame, chunks[1]),
            Tab::Cluster => self.draw_cluster::<B>(frame, chunks[1]),
            Tab::Logs => self.draw_logs::<B>(frame, chunks[1], app.log_selected, app.log_scroll, app.log_focus_right),
            Tab::Actions => self.draw_actions::<B>(frame, chunks[1]),
        }
    }

    fn draw_overview<B: Backend>(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let data: OverviewData = self.overview_srv.get_overview();

        let outer_block = Block::default()
            .title("Overview")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(5),
                Constraint::Length(6),
            ])
            .split(inner_area);

        let header1 = Paragraph::new(format!(
            "Scope: {}    State: {}",
            data.cluster_data.scope, data.cluster_data.patroni_data.state,
        ))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(header1, chunks[0]);

        let header2 = Paragraph::new(format!(
            "Host: {} ({})    Role: {}    Leader: {}",
            data.hostname,
            data.ip,
            data.cluster_data.patroni_data.role,
            data.cluster_data.leader_node_name
        ))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(header2, chunks[1]);

        let rows: Vec<Row> = data
            .statuses
            .iter()
            .map(|(svc, status)| {
                let color = if status == "UP" {
                    Color::Green
                } else {
                    Color::Red
                };
                Row::new(vec![
                    Cell::from(svc.clone()),
                    Cell::from(status.clone()).style(Style::default().fg(color)),
                ])
            })
            .collect();

        let service_table = Table::new(rows, &[Constraint::Length(15), Constraint::Length(8)])
            .block(Block::default().borders(Borders::ALL).title("Services"));
        frame.render_widget(service_table, chunks[2]);

        let error_rows: Vec<Row> = data
            .errors
            .iter()
            .map(|(svc, count)| Row::new(vec![Cell::from(svc.clone()), Cell::from(count.to_string())]))
            .collect();

        let error_table = Table::new(error_rows, &[Constraint::Length(15), Constraint::Length(6)])
            .block(Block::default().borders(Borders::ALL).title("Log Errors"));
        frame.render_widget(error_table, chunks[3]);
    }

    fn draw_cluster<B: Backend>(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let data = self.cluster_srv.get_cluster_info();

        let outer_block = Block::default()
            .title("Cluster Status")
            .borders(Borders::ALL);
        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        let rows: Vec<Row> = data.members.iter().map(|node| {
            let color = match node.role.as_str() {
                "leader" => Color::Green,
                "replica" => Color::Cyan,
                _ => Color::Yellow,
            };

            let status_color = if node.state == "running" {
                Color::Green
            } else {
                Color::Cyan
            };

            Row::new(vec![
                Cell::from(node.name.clone()),
                Cell::from(node.role.clone()).style(Style::default().fg(color)),
                Cell::from(node.state.clone()).style(Style::default().fg(status_color)),
                Cell::from(node.host.clone()),
                Cell::from(node.lag.map_or("-".to_string(), |l| l.to_string())),
            ])
        }).collect();

        let table = Table::new(rows, &[Constraint::Length(15), Constraint::Length(8)])
            .block(Block::default().borders(Borders::ALL).title("Nodes"))
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(16),
                Constraint::Length(6),
            ])
            .header(Row::new(["Name", "Role", "State", "Host", "Lag"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            );

        frame.render_widget(table, inner_area);
    }

    fn draw_logs<B: Backend>(
        &self,
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        selected: usize,
        scroll: u16,
        focus_right: bool,
    ) {
        let block = Block::default()
            .title("Logs")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20),
                Constraint::Min(1)
            ])
            .split(inner);

        let items: Vec<ListItem> = SERVICES.iter().enumerate().map(|(i, svc)| {
            let style = if i == selected && !focus_right {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(svc.to_string()).style(style)
        }).collect();

        let svc_list = List::new(items)
            .block(Block::default().title("Services").borders(Borders::ALL));
        frame.render_widget(svc_list, chunks[0]);

        let selected_service = SERVICES[selected];
        let lines = self.logs_srv.read_logs(selected_service, 100);
        let text: Vec<Line> = lines.iter().map(|l| Line::from(l.clone())).collect();

        let border_style = if focus_right {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let logs = Paragraph::new(text)
            .block(Block::default().
                title(format!("{} log", selected_service)).
                borders(Borders::ALL).
                border_style(border_style),
            )
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false });

        frame.render_widget(logs, chunks[1]);
    }

    fn draw_actions<B: Backend>(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let block = Block::default().title("Actions").borders(Borders::ALL);
        frame.render_widget(block, area);
    }
}