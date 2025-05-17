use crate::app::{App, Tab};
use crate::components::cluster::Cluster;
use crate::components::logs::Logs;
use crate::components::overview::{Overview, OverviewData};
use ratatui::backend::Backend;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::*,
    widgets::*,
};

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
            Tab::Logs => self.draw_logs::<B>(
                frame,
                chunks[1],
                app.log_selected,
                app.log_scroll,
                app.log_focus_right,
            ),
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
            ])
            .split(inner_area);

        self.draw_overview_header::<B>(frame, &data, chunks[0], chunks[1]);
        self.draw_overview_table::<B>(frame, &data, chunks[2]);
    }

    fn draw_overview_header<B: Backend>(
        &self,
        frame: &mut Frame,
        data: &OverviewData,
        header_area: ratatui::layout::Rect,
        subheader_area: ratatui::layout::Rect,
    ) {
        let label_style = Style::default().fg(Color::Gray);
        // стиль для значений
        let value_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let header1_line = Line::from(vec![
            Span::styled("Scope: ", label_style),
            Span::styled(&data.cluster_data.scope, value_style),
            Span::raw("    "),
            Span::styled("State: ", label_style),
            Span::styled(&data.cluster_data.patroni_data.state, value_style),
        ]);
        frame.render_widget(Paragraph::new(header1_line), header_area);

        let header2_line = Line::from(vec![
            Span::styled("Host: ", label_style),
            Span::styled(format!("{} ({})", data.hostname, data.ip), value_style),
            Span::raw("    "),
            Span::styled("Role: ", label_style),
            Span::styled(&data.cluster_data.patroni_data.role, value_style),
            Span::raw("    "),
            Span::styled("Leader: ", label_style),
            Span::styled(&data.cluster_data.leader_node_name, value_style),
        ]);
        frame.render_widget(Paragraph::new(header2_line), subheader_area);
    }

    fn draw_overview_table<B: Backend>(
        &self,
        frame: &mut Frame,
        data: &OverviewData,
        table_area: ratatui::layout::Rect,
    ) {
        let rows: Vec<Row> = data
            .statuses
            .iter()
            .map(|(svc, status)| {
                let error_count = data
                    .errors
                    .iter()
                    .find(|(error_svc, _)| error_svc == svc)
                    .map(|(_, count)| count)
                    .unwrap_or(&0);

                let status_color = if status == "UP" {
                    Color::Green
                } else {
                    Color::Red
                };

                Row::new(vec![
                    Cell::from(svc.clone()),
                    Cell::from(status.clone()).style(Style::default().fg(status_color)),
                    Cell::from(error_count.to_string()),
                ])
            })
            .collect();

        let combined_table = Table::new(
            rows,
            &[
                Constraint::Length(15),
                Constraint::Length(8),
                Constraint::Length(8),
            ],
        )
        .header(
            Row::new(vec!["Component", "Status", "Errors"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Services Status"),
        );

        frame.render_widget(combined_table, table_area);
    }

    fn draw_cluster<B: Backend>(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let data = self.cluster_srv.get_cluster_info();

        let outer_block = Block::default()
            .title("Cluster Status")
            .borders(Borders::ALL);
        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        let rows: Vec<Row> = data
            .members
            .iter()
            .map(|node| {
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
            })
            .collect();

        let table = Table::new(rows, &[Constraint::Length(15), Constraint::Length(8)])
            .block(Block::default().borders(Borders::ALL).title("Nodes"))
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(16),
                Constraint::Length(6),
            ])
            .header(
                Row::new(["Name", "Role", "State", "Host", "Lag"]).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
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
        let block = Block::default().title("Logs").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(1)])
            .split(inner);

        let items: Vec<ListItem> = SERVICES
            .iter()
            .enumerate()
            .map(|(i, svc)| {
                let style = if i == selected && !focus_right {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(svc.to_string()).style(style)
            })
            .collect();

        let svc_list =
            List::new(items).block(Block::default().title("Services").borders(Borders::ALL));
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
            .block(
                Block::default()
                    .title(format!("{} log", selected_service))
                    .borders(Borders::ALL)
                    .border_style(border_style),
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