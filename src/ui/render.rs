use crate::app::Tab;
use crate::config::Config;
use crate::patroni::patroni::ClusterInfo;
use crate::services::actions::{Action, ActionsService};
use crate::services::overview::{OverviewData, OverviewService};
use crate::system;
use crate::ui::layout;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs, Wrap};

/// Create the tabs widget
pub fn create_tabs(current_tab: &Tab) -> Tabs<'static> {
    let tab_titles = ["1: Overview", "2: Cluster", "3: Logs", "4: Actions"];
    Tabs::new(
        tab_titles
            .iter()
            .cloned()
            .map(Line::from)
            .collect::<Vec<Line>>(),
    )
    .block(Block::default().borders(Borders::ALL).title("Navigation"))
    .highlight_style(Style::default().fg(Color::Yellow))
    .select(*current_tab as usize)
}

/// Draw the overview tab
pub fn draw_overview(
    frame: &mut Frame,
    area: Rect,
    data: &OverviewData,
    overview_service: &OverviewService,
    config: &Config,
) {
    let (outer_area, chunks) = layout::create_overview_layout(area);

    // Render the outer block
    let outer_block = Block::default()
        .title("Overview")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    frame.render_widget(outer_block, outer_area);

    // Create and render the header
    let header_chunks = layout::create_overview_header_layout(chunks[0]);
    draw_overview_header(frame, data, header_chunks[0], header_chunks[1], overview_service, config);

    // Create and render the table
    draw_overview_table(frame, data, chunks[1]);
}

/// Draw the overview header
fn draw_overview_header(
    frame: &mut Frame,
    data: &OverviewData,
    header_area: Rect,
    subheader_area: Rect,
    overview_service: &OverviewService,
    config: &Config,
) {
    let role_raw = data
        .cluster_data
        .members_map
        .get(&data.cluster_data.node_name)
        .map(|n| n.role.as_str())
        .unwrap_or("-");
    let role_human = match role_raw {
        "leader" => "Primary",
        _ => "Replica",
    };

    let title = Line::from(vec![
        Span::styled(
            format!("Cluster: {}  ", data.cluster_data.scope),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        ),
        Span::raw(format!(
            "Node: {}   Role: {}   Leader: {}",
            data.cluster_data.node_name, role_human, data.cluster_data.leader_node_name
        )),
    ]);

    let title_par = Paragraph::new(title)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title_par, header_area);

    let lag_str = data
        .cluster_data
        .members_map
        .get(&data.cluster_data.node_name)
        .and_then(|n| n.lag)
        .map(|micros| format!("{:.1}s", micros as f64 / 1_000_000.0))
        .unwrap_or_else(|| "-".to_string());

    // HAProxy backend stats
    let (ha_curr, ha_max) = overview_service.fetch_haproxy_backend_stats();

    // Replication health
    let repl_ok = data.cluster_data.replication_ok(config.max_replication_lag_us());

    // VIP
    let vip = system::detect_keepalived_vip();

    let sub_lines = vec![
        Line::from(vec![
            Span::styled("Lag: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{lag_str}   ")),
            Span::styled(
                if repl_ok { "Replication OK" } else { "Replication FAILED" },
                Style::default().fg(if repl_ok { Color::Green } else { Color::Red }),
            ),
        ]),
        Line::from(vec![
            Span::styled(" HAProxy: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("Backend OK ({}/{})   ", ha_curr, ha_max)),
            Span::styled("VIP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(vip),
        ]),
    ];

    let sub_par = Paragraph::new(sub_lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    frame.render_widget(sub_par, subheader_area);
}

/// Draw the overview table
fn draw_overview_table(
    frame: &mut Frame,
    data: &OverviewData,
    table_area: Rect,
) {
    let rows: Vec<Row> = data
        .components
        .iter()
        .map(|c| {
            let status_text = if c.up { "UP" } else { "DOWN" };
            let status_color = if c.up { Color::Green } else { Color::Red };

            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(status_text).style(Style::default().fg(status_color)),
                Cell::from(c.errors.to_string()),
                Cell::from(c.uptime.clone()),
                Cell::from(c.version.clone()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(25),
            Constraint::Length(18),
        ],
    )
        .header(
            Row::new(["Component", "Status", "Errors", "Uptime", "Version"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Services Status"),
        );

    frame.render_widget(table, table_area);
}

/// Draw the cluster tab
pub fn draw_cluster(
    frame: &mut Frame,
    area: Rect,
    data: &ClusterInfo,
) {
    let inner_area = layout::create_cluster_layout(area);

    // Render the outer block
    let outer_block = Block::default()
        .title("Cluster Status")
        .borders(Borders::ALL);
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

/// Draw the logs tab
pub fn draw_logs(
    frame: &mut Frame,
    area: Rect,
    services: &[String],
    selected: usize,
    scroll: u16,
    focus_right: bool,
    lines: &[String],
    selected_service: &str,
) {
    let (outer_area, chunks) = layout::create_logs_layout(area);

    // Render the outer block
    let block = Block::default().title("Logs").borders(Borders::ALL);
    frame.render_widget(block, outer_area);

    // Render the services list
    let items: Vec<ListItem> = services
        .iter()
        .enumerate()
        .map(|(i, svc)| {
            let style = if i == selected && !focus_right {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(svc.clone()).style(style)
        })
        .collect();

    let svc_list =
        List::new(items).block(Block::default().title("Services").borders(Borders::ALL));
    frame.render_widget(svc_list, chunks[0]);

    // Render the log content
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

/// Draw the actions tab
pub fn draw_actions(
    frame: &mut Frame,
    area: Rect,
    actions_service: &ActionsService,
    selected: usize,
    confirmation: bool,
    confirmation_yes: bool,
    target_node: &str,
    error: &Option<String>,
    cluster_info: &ClusterInfo,
) {
    let (outer_area, chunks) = layout::create_actions_layout(area);

    // Render the outer block
    let block = Block::default().title("Actions").borders(Borders::ALL);
    frame.render_widget(block, outer_area);

    // Get all available actions
    let actions = Action::all();

    // Render the actions list
    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if i == selected {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(action.as_str()).style(style)
        })
        .collect();

    let actions_list = List::new(items)
        .block(Block::default().title("Available Actions").borders(Borders::ALL));
    frame.render_widget(actions_list, chunks[0]);

    // Render the action details
    if selected < actions.len() {
        let action = &actions[selected];

        // Create details text
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    action.as_str(),
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from(action.description()),
            Line::from(""),
        ];

        // Add keyboard shortcuts
        lines.push(Line::from(vec![
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Execute action"),
        ]));

        // Add node selection for node-specific actions
        match action {
            Action::Switchover => {
                lines.push(Line::from(""));
                lines.push(Line::from("Select target node:"));

                // Add node list
                for node in &cluster_info.members {
                    if node.role != "leader" {
                        let selected_marker = if target_node == node.name { " (selected)" } else { "" };
                        lines.push(Line::from(format!("- {}{}", node.name, selected_marker)));
                    }
                }

                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(": Select next node"),
                ]));
            },
            Action::Restart | Action::Reinitialize => {
                lines.push(Line::from(""));
                lines.push(Line::from("Select node:"));

                // Add node list
                for node in &cluster_info.members {
                    let selected_marker = if target_node == node.name { " (selected)" } else { "" };
                    lines.push(Line::from(format!("- {}{}", node.name, selected_marker)));
                }

                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(": Select next node"),
                ]));
            },
            _ => {}
        }

        // Show confirmation dialog if needed
        if confirmation {
            let dialog_text = match action {
                Action::Switchover => format!("Are you sure you want to switchover to {}?", target_node),
                Action::Restart => format!("Are you sure you want to restart {}?", target_node),
                Action::Reinitialize => format!("Are you sure you want to reinitialize {}?", target_node),
                Action::PauseCluster => "Are you sure you want to pause the cluster?".to_string(),
                Action::ResumeCluster => "Are you sure you want to resume the cluster?".to_string(),
            };

            let yes_style = if confirmation_yes {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };

            let no_style = if !confirmation_yes {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };

            // Create confirmation dialog
            let dialog = Paragraph::new(vec![
                Line::from(dialog_text),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Yes", yes_style),
                    Span::raw("   "),
                    Span::styled("No", no_style),
                ]),
            ])
            .block(Block::default().title("Confirm Action").borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);

            // Calculate dialog position
            let dialog_width = 50;
            let dialog_height = 5;
            let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
            let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

            let dialog_area = Rect::new(
                area.x + dialog_x,
                area.y + dialog_y,
                dialog_width,
                dialog_height,
            );

            frame.render_widget(dialog, dialog_area);
        } else if let Some(err) = error {
            // Show error message if there is one
            let error_text = format!("Error: {}", err);

            // Create error dialog
            let dialog = Paragraph::new(vec![
                Line::from(error_text),
                Line::from(""),
                Line::from("Press Esc to dismiss"),
            ])
            .block(Block::default().title("Error").borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);

            // Calculate dialog position
            let dialog_width = 50;
            let dialog_height = 5;
            let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
            let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

            let dialog_area = Rect::new(
                area.x + dialog_x,
                area.y + dialog_y,
                dialog_width,
                dialog_height,
            );

            frame.render_widget(dialog, dialog_area);
        } else {
            // Render the details in the right panel
            let details = Paragraph::new(lines)
                .block(Block::default().title("Details").borders(Borders::ALL))
                .wrap(Wrap { trim: true });

            frame.render_widget(details, chunks[1]);
        }
    }
}
