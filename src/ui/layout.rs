use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create the main layout with tabs and content area
pub fn create_main_layout(size: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size)
        .to_vec()
}

/// Create the overview layout
pub fn create_overview_layout(area: Rect) -> (Rect, Vec<Rect>) {
    let outer_block = ratatui::widgets::Block::default()
        .title("Overview")
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::White));

    let inner_area = outer_block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(5),
        ])
        .split(inner_area)
        .to_vec();

    (area, chunks)
}

/// Create the overview header layout
pub fn create_overview_header_layout(header_area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3)])
        .split(header_area)
        .to_vec()
}

/// Create the cluster layout
pub fn create_cluster_layout(area: Rect) -> Rect {
    let outer_block = ratatui::widgets::Block::default()
        .title("Cluster Status")
        .borders(ratatui::widgets::Borders::ALL);

    outer_block.inner(area)
}

/// Create the logs layout
pub fn create_logs_layout(area: Rect) -> (Rect, Vec<Rect>) {
    let block = ratatui::widgets::Block::default()
        .title("Logs")
        .borders(ratatui::widgets::Borders::ALL);

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(1)])
        .split(inner)
        .to_vec();

    (area, chunks)
}
