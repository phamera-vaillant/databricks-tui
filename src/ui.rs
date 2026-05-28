use crate::app::{App, Panel};
use crate::shape::{Shape, Status};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    draw_header(f, root[0], app);

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
        draw_panel(f, areas[i], panel.title(), shape, focused);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let badge_text = match &app.user_badge {
        Some(Shape::Badge(b)) => format!(" {} {} ", b.label, b.value),
        _ => " Databricks TUI ".to_string(),
    };
    let status = if app.loading {
        " [refreshing…]"
    } else {
        ""
    };
    let title = format!("{}{}", badge_text, status);
    let p = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(p, area);
}

fn draw_panel(
    f: &mut Frame,
    area: Rect,
    title: &str,
    shape: Option<&Shape>,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    match shape {
        None => {
            let p = Paragraph::new("Loading…").block(block);
            f.render_widget(p, area);
        }
        Some(Shape::List(items)) => {
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|item| {
                    let color = status_color(&item.status);
                    let mut spans = vec![
                        Span::styled(
                            format!("[{}] ", item.status.label()),
                            Style::default().fg(color),
                        ),
                        Span::raw(item.name.clone()),
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
                .map(|h| Cell::from(h.as_str()).style(Style::default().add_modifier(Modifier::BOLD)))
                .collect();
            let header = Row::new(header_cells).style(Style::default().fg(Color::Cyan));
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
