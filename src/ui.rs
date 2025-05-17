use crate::app::{App, Tab};
use crate::components::overview::{OverviewData, get_overview};
use ratatui::backend::Backend;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::*,
    widgets::*,
};

pub fn draw_ui<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

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
    f.render_widget(tabs, chunks[0]);

    match app.current_tab {
        Tab::Overview => draw_overview::<B>(f, chunks[1]),
        Tab::Cluster => draw_cluster::<B>(f, chunks[1]),
        Tab::Logs => draw_logs::<B>(f, chunks[1]),
        Tab::Actions => draw_actions::<B>(f, chunks[1]),
    }
}

fn draw_overview<B: Backend>(f: &mut Frame, area: ratatui::layout::Rect) {
    let data: OverviewData = get_overview();

    let outer_block = Block::default()
        .title("Overview")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

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
        data.patroni_data.scope, data.patroni_data.state,
    ))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(header1, chunks[0]);

    let header2 = Paragraph::new(format!(
        "Host: {} ({})    Role: {}    Leader: {}",
        data.hostname, data.ip, data.patroni_data.role, data.patroni_data.leader
    ))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(header2, chunks[1]);

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
    f.render_widget(service_table, chunks[2]);

    let error_rows: Vec<Row> = data
        .errors
        .iter()
        .map(|(svc, count)| Row::new(vec![Cell::from(svc.clone()), Cell::from(count.to_string())]))
        .collect();

    let error_table = Table::new(error_rows, &[Constraint::Length(15), Constraint::Length(6)])
        .block(Block::default().borders(Borders::ALL).title("Log Errors"));
    f.render_widget(error_table, chunks[3]);
}

fn draw_cluster<B: Backend>(f: &mut Frame, area: ratatui::layout::Rect) {
    let block = Block::default().title("Cluster").borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_logs<B: Backend>(f: &mut Frame, area: ratatui::layout::Rect) {
    let block = Block::default().title("Logs").borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_actions<B: Backend>(f: &mut Frame, area: ratatui::layout::Rect) {
    let block = Block::default().title("Actions").borders(Borders::ALL);
    f.render_widget(block, area);
}
