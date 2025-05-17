use ratatui::{Frame, layout::{Layout, Constraint, Direction}, widgets::*};
use ratatui::backend::Backend;
use ratatui::style::Style;
use ratatui::text::Line;
use crate::app::{App, Tab};

pub fn draw_ui<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.size());

    let tab_titles = ["1: Overview", "2: Cluster", "3: Logs", "4: Actions"];
    let tabs = Tabs::new(tab_titles.iter().cloned().map(Line::from).collect::<Vec<Line>>())
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .highlight_style(Style::default().fg(ratatui::style::Color::Yellow))
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
    let block = Block::default().title("Overview").borders(Borders::ALL);
    f.render_widget(block, area);
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
