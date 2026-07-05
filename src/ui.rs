use crate::app::{App, Panel};
use crate::shape::{Shape, Status};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Padding, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_header(f, root[0], app);
    draw_footer(f, root[2], app);

    if app.zoomed {
        let idx = Panel::ALL.iter().position(|p| p == &app.focus).unwrap_or(0);
        draw_panel(
            f,
            root[1],
            app.focus,
            app.shapes[idx].as_ref(),
            true,
            app.spinner(),
        );
        return;
    }

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body[1]);

    let areas = [left[0], left[1], right[0], right[1]];

    for (i, panel) in Panel::ALL.iter().enumerate() {
        let focused = app.focus == *panel;
        let shape = app.shapes[i].as_ref();
        draw_panel(f, areas[i], *panel, shape, focused, app.spinner());
    }
}

fn accent(panel: Panel) -> Color {
    match panel {
        Panel::Clusters => Color::Cyan,
        Panel::Jobs => Color::Magenta,
        Panel::Pipelines => Color::Green,
        Panel::Warehouses => Color::Blue,
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut left = vec![
        Span::styled(" ◢◤ ", Style::default().fg(Color::Red)),
        Span::styled(
            "Databricks",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" TUI v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
    ];
    if let Some(Shape::Badge(b)) = &app.user_badge {
        left.push(Span::styled("  ·  ", Style::default().fg(Color::DarkGray)));
        left.push(Span::styled(
            format!("{} {}", b.label, b.value),
            Style::default().fg(Color::Cyan),
        ));
    }
    if app.zoomed {
        left.push(Span::styled("  ·  ", Style::default().fg(Color::DarkGray)));
        left.push(Span::styled(
            format!("⛶ {}", app.focus.title()),
            Style::default()
                .fg(accent(app.focus))
                .add_modifier(Modifier::BOLD),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(left)), inner);

    let right = if app.loading {
        Line::from(Span::styled(
            format!("{} refreshing ", app.spinner()),
            Style::default().fg(Color::Yellow),
        ))
    } else {
        Line::from(Span::styled(
            format!("updated {}s ago ", app.last_refresh_age().as_secs()),
            Style::default().fg(Color::DarkGray),
        ))
    };
    f.render_widget(Paragraph::new(right).alignment(Alignment::Right), inner);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let key = |k: &'static str| {
        Span::styled(
            k,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    };
    let dim = |t: &'static str| Span::styled(t, Style::default().fg(Color::DarkGray));
    let spans = vec![
        dim(" "),
        key("tab"),
        dim("/"),
        key("h"),
        dim("/"),
        key("l"),
        dim(" switch   "),
        key("z"),
        dim(if app.zoomed { " unzoom   " } else { " zoom   " }),
        key("r"),
        dim(" refresh   "),
        key("q"),
        dim(" quit"),
    ];
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_panel(
    f: &mut Frame,
    area: Rect,
    panel: Panel,
    shape: Option<&Shape>,
    focused: bool,
    spinner: &str,
) {
    let accent = accent(panel);
    let (border_style, title_style) = if focused {
        (
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(Color::DarkGray),
            Style::default().fg(Color::Gray),
        )
    };
    let count = match shape {
        Some(Shape::List(items)) => format!(" · {}", items.len()),
        Some(Shape::Table(data)) => format!(" · {}", data.rows.len()),
        _ => String::new(),
    };
    let title = Line::from(vec![
        Span::styled(format!(" {} ", panel.icon()), Style::default().fg(accent)),
        Span::styled(format!("{}{} ", panel.title(), count), title_style),
    ]);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    match shape {
        None => {
            let p = Paragraph::new(format!("{} Loading…", spinner))
                .style(Style::default().fg(Color::Yellow))
                .block(block);
            f.render_widget(p, area);
        }
        Some(Shape::List(items)) if items.is_empty() => {
            let p = Paragraph::new("— none —")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(p, area);
        }
        Some(Shape::List(items)) => {
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|item| {
                    let color = status_color(&item.status);
                    let mut spans = vec![
                        Span::styled("● ", Style::default().fg(color)),
                        Span::styled(item.name.as_str(), Style::default().fg(Color::White)),
                        Span::styled(
                            format!("  {}", item.status.label()),
                            Style::default().fg(color).add_modifier(Modifier::DIM),
                        ),
                    ];
                    if let Some(detail) = &item.detail {
                        spans.push(Span::styled(
                            format!("  {}", detail),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    ListItem::new(Line::from(spans))
                })
                .collect();
            let list = List::new(list_items).block(block);
            f.render_widget(list, area);
        }
        Some(Shape::Table(data)) => {
            let header_cells: Vec<Cell> = data
                .headers
                .iter()
                .map(|h| {
                    Cell::from(h.as_str()).style(Style::default().add_modifier(Modifier::BOLD))
                })
                .collect();
            let header = Row::new(header_cells).style(Style::default().fg(accent));
            let rows: Vec<Row> = data
                .rows
                .iter()
                .map(|r| Row::new(r.iter().map(|c| Cell::from(c.as_str())).collect::<Vec<_>>()))
                .collect();
            let widths: Vec<Constraint> = data
                .headers
                .iter()
                .map(|_| Constraint::Percentage(100 / data.headers.len() as u16))
                .collect();
            let table = Table::new(rows, widths).header(header).block(block);
            f.render_widget(table, area);
        }
        Some(Shape::Badge(b)) => {
            let text = format!("{}: {}", b.label, b.value);
            let p = Paragraph::new(text).block(block);
            f.render_widget(p, area);
        }
        Some(Shape::Text(t)) => {
            let p = Paragraph::new(t.as_str()).block(block);
            f.render_widget(p, area);
        }
    }
}

fn status_color(status: &Status) -> Color {
    match status {
        Status::Running => Color::Green,
        Status::Stopped => Color::DarkGray,
        Status::Pending => Color::Yellow,
        Status::Failed => Color::Red,
        Status::Unknown(_) => Color::White,
    }
}
