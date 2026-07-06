use crate::app::{App, Panel, ThemeMode};
use crate::shape::{Shape, Status};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Padding, Paragraph, Row,
        Table, Wrap,
    },
    Frame,
};

struct Palette {
    text: Color,
    dim: Color,
    border: Color,
    warn: Color,
    ok: Color,
    err: Color,
    key: Color,
    brand: Color,
    clusters: Color,
    jobs: Color,
    pipelines: Color,
    warehouses: Color,
}

fn palette(mode: ThemeMode) -> Palette {
    match mode {
        // Dark theme sticks to ANSI colors so it follows the terminal's own scheme.
        ThemeMode::Dark => Palette {
            text: Color::White,
            dim: Color::DarkGray,
            border: Color::DarkGray,
            warn: Color::Yellow,
            ok: Color::Green,
            err: Color::Red,
            key: Color::Cyan,
            brand: Color::Red,
            clusters: Color::Cyan,
            jobs: Color::Magenta,
            pipelines: Color::Green,
            warehouses: Color::Blue,
        },
        // Light theme uses explicit darker shades that stay readable on a white background.
        ThemeMode::Light => Palette {
            text: Color::Black,
            dim: Color::Rgb(107, 114, 128),
            border: Color::Rgb(156, 163, 175),
            warn: Color::Rgb(180, 83, 9),
            ok: Color::Rgb(21, 128, 61),
            err: Color::Rgb(185, 28, 28),
            key: Color::Rgb(8, 145, 178),
            brand: Color::Rgb(220, 38, 38),
            clusters: Color::Rgb(8, 145, 178),
            jobs: Color::Rgb(162, 28, 175),
            pipelines: Color::Rgb(21, 128, 61),
            warehouses: Color::Rgb(29, 78, 216),
        },
    }
}

fn accent(panel: Panel, p: &Palette) -> Color {
    match panel {
        Panel::Clusters => p.clusters,
        Panel::Jobs => p.jobs,
        Panel::Pipelines => p.pipelines,
        Panel::Warehouses => p.warehouses,
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let p = palette(app.theme);
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_header(f, root[0], app, &p);
    draw_footer(f, root[2], app, &p);

    if app.detail.is_some() {
        draw_detail(f, root[1], app, &p);
        return;
    }

    if app.zoomed {
        let idx = Panel::ALL
            .iter()
            .position(|pn| pn == &app.focus)
            .unwrap_or(0);
        draw_panel(
            f,
            root[1],
            app.focus,
            app.shapes[idx].as_ref(),
            true,
            Some(app.selection(idx)),
            app.spinner(),
            &p,
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
        let selected = focused.then(|| app.selection(i));
        draw_panel(
            f,
            areas[i],
            *panel,
            shape,
            focused,
            selected,
            app.spinner(),
            &p,
        );
    }
}

fn draw_detail(f: &mut Frame, area: Rect, app: &App, p: &Palette) {
    let d = app.detail.as_ref().unwrap();
    let acc = accent(d.panel, p);
    let title = Line::from(vec![
        Span::styled(format!(" {} ", d.panel.icon()), Style::default().fg(acc)),
        Span::styled(
            format!("{} · {} ", d.panel.title(), d.name),
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        ),
    ]);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(acc).add_modifier(Modifier::BOLD))
        .padding(Padding::horizontal(1));

    match &d.content {
        None => {
            let par = Paragraph::new(format!("{} Loading…", app.spinner()))
                .style(Style::default().fg(p.warn))
                .block(block);
            f.render_widget(par, area);
        }
        Some(text) => {
            let par = Paragraph::new(text.as_str())
                .style(Style::default().fg(p.text))
                .wrap(Wrap { trim: false })
                .scroll((d.scroll, 0))
                .block(block);
            f.render_widget(par, area);
        }
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, p: &Palette) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(p.border));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut left = vec![
        Span::styled(" ◢◤ ", Style::default().fg(p.brand)),
        Span::styled(
            "Databricks",
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" TUI v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(p.dim),
        ),
    ];
    if let Some(Shape::Badge(b)) = &app.user_badge {
        left.push(Span::styled("  ·  ", Style::default().fg(p.dim)));
        left.push(Span::styled(
            format!("{} {}", b.label, b.value),
            Style::default().fg(p.key),
        ));
    }
    if app.zoomed {
        left.push(Span::styled("  ·  ", Style::default().fg(p.dim)));
        left.push(Span::styled(
            format!("⛶ {}", app.focus.title()),
            Style::default()
                .fg(accent(app.focus, p))
                .add_modifier(Modifier::BOLD),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(left)), inner);

    let right = if app.loading {
        Line::from(Span::styled(
            format!("{} refreshing ", app.spinner()),
            Style::default().fg(p.warn),
        ))
    } else {
        Line::from(Span::styled(
            format!("updated {}s ago ", app.last_refresh_age().as_secs()),
            Style::default().fg(p.dim),
        ))
    };
    f.render_widget(Paragraph::new(right).alignment(Alignment::Right), inner);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App, p: &Palette) {
    let key =
        |k: &'static str| Span::styled(k, Style::default().fg(p.key).add_modifier(Modifier::BOLD));
    let dim = |t: &'static str| Span::styled(t, Style::default().fg(p.dim));
    let spans = if app.detail.is_some() {
        vec![
            dim(" "),
            key("esc"),
            dim(" back   "),
            key("j"),
            dim("/"),
            key("k"),
            dim(" scroll   "),
            key("q"),
            dim(" quit"),
        ]
    } else {
        vec![
            dim(" "),
            key("tab"),
            dim("/"),
            key("h"),
            dim("/"),
            key("l"),
            dim(" switch   "),
            key("j"),
            dim("/"),
            key("k"),
            dim(" select   "),
            key("enter"),
            dim(" details   "),
            key("z"),
            dim(if app.zoomed { " unzoom   " } else { " zoom   " }),
            key("t"),
            dim(" theme   "),
            key("r"),
            dim(" refresh   "),
            key("q"),
            dim(" quit"),
        ]
    };
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

#[allow(clippy::too_many_arguments)]
fn draw_panel(
    f: &mut Frame,
    area: Rect,
    panel: Panel,
    shape: Option<&Shape>,
    focused: bool,
    selected: Option<usize>,
    spinner: &str,
    p: &Palette,
) {
    let accent = accent(panel, p);
    let (border_style, title_style) = if focused {
        (
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
            Style::default().fg(p.text).add_modifier(Modifier::BOLD),
        )
    } else {
        (Style::default().fg(p.border), Style::default().fg(p.dim))
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
            let par = Paragraph::new(format!("{} Loading…", spinner))
                .style(Style::default().fg(p.warn))
                .block(block);
            f.render_widget(par, area);
        }
        Some(Shape::List(items)) if items.is_empty() => {
            let par = Paragraph::new("— none —")
                .style(Style::default().fg(p.dim))
                .block(block);
            f.render_widget(par, area);
        }
        Some(Shape::List(items)) => {
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|item| {
                    let color = status_color(&item.status, p);
                    let mut spans = vec![
                        Span::styled("● ", Style::default().fg(color)),
                        Span::styled(item.name.as_str(), Style::default().fg(p.text)),
                        Span::styled(
                            format!("  {}", item.status.label()),
                            Style::default().fg(color).add_modifier(Modifier::DIM),
                        ),
                    ];
                    if let Some(detail) = &item.detail {
                        spans.push(Span::styled(
                            format!("  {}", detail),
                            Style::default().fg(p.dim),
                        ));
                    }
                    ListItem::new(Line::from(spans))
                })
                .collect();
            let list = List::new(list_items)
                .block(block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            match selected {
                Some(sel) => {
                    let mut state = ListState::default().with_selected(Some(sel));
                    f.render_stateful_widget(list, area, &mut state);
                }
                None => f.render_widget(list, area),
            }
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
            let par = Paragraph::new(text)
                .style(Style::default().fg(p.text))
                .block(block);
            f.render_widget(par, area);
        }
        Some(Shape::Text(t)) => {
            let par = Paragraph::new(t.as_str())
                .style(Style::default().fg(p.text))
                .block(block);
            f.render_widget(par, area);
        }
    }
}

fn status_color(status: &Status, p: &Palette) -> Color {
    match status {
        Status::Running => p.ok,
        Status::Stopped => p.dim,
        Status::Pending => p.warn,
        Status::Failed => p.err,
        Status::Unknown(_) => p.text,
    }
}
