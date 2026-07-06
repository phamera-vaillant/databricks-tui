use crate::cli::DatabricksCli;
use crate::fetchers;
use crate::shape::Shape;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn toggled(self) -> Self {
        match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Clusters,
    Jobs,
    Pipelines,
    Warehouses,
}

impl Panel {
    pub const ALL: &'static [Panel] = &[
        Panel::Clusters,
        Panel::Jobs,
        Panel::Pipelines,
        Panel::Warehouses,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Panel::Clusters => "Clusters",
            Panel::Jobs => "Jobs",
            Panel::Pipelines => "Pipelines",
            Panel::Warehouses => "Warehouses",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Panel::Clusters => "◆",
            Panel::Jobs => "◈",
            Panel::Pipelines => "⇶",
            Panel::Warehouses => "▣",
        }
    }
}

enum Update {
    Panel(usize, Option<Shape>),
    Badge(Option<Shape>),
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct App {
    pub focus: Panel,
    pub theme: ThemeMode,
    pub zoomed: bool,
    pub shapes: Vec<Option<Shape>>,
    pub user_badge: Option<Shape>,
    pub error: Option<String>,
    pub refresh_interval: Duration,
    last_refresh: Instant,
    pub loading: bool,
    pending: Option<mpsc::UnboundedReceiver<Update>>,
    in_flight: usize,
    spinner_frame: usize,
}

impl App {
    pub fn new(refresh_secs: u64, theme: ThemeMode) -> Self {
        Self {
            focus: Panel::Clusters,
            theme,
            zoomed: false,
            shapes: vec![None, None, None, None],
            user_badge: None,
            error: None,
            refresh_interval: Duration::from_secs(refresh_secs),
            last_refresh: Instant::now()
                .checked_sub(Duration::from_secs(refresh_secs + 1))
                .unwrap_or(Instant::now()),
            loading: false,
            pending: None,
            in_flight: 0,
            spinner_frame: 0,
        }
    }

    pub fn last_refresh_age(&self) -> Duration {
        self.last_refresh.elapsed()
    }

    pub fn spinner(&self) -> &'static str {
        SPINNER_FRAMES[self.spinner_frame % SPINNER_FRAMES.len()]
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_frame = self.spinner_frame.wrapping_add(1);
    }

    pub fn toggle_zoom(&mut self) {
        self.zoomed = !self.zoomed;
    }

    pub fn focus_next(&mut self) {
        let idx = Panel::ALL
            .iter()
            .position(|p| p == &self.focus)
            .unwrap_or(0);
        self.focus = Panel::ALL[(idx + 1) % Panel::ALL.len()];
    }

    pub fn focus_prev(&mut self) {
        let idx = Panel::ALL
            .iter()
            .position(|p| p == &self.focus)
            .unwrap_or(0);
        self.focus = Panel::ALL[(idx + Panel::ALL.len() - 1) % Panel::ALL.len()];
    }

    pub fn needs_refresh(&self) -> bool {
        !self.loading && self.last_refresh.elapsed() >= self.refresh_interval
    }

    pub fn start_refresh(&mut self, cli: &Arc<DatabricksCli>) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.error = None;
        self.last_refresh = Instant::now();

        let (tx, rx) = mpsc::unbounded_channel();
        self.pending = Some(rx);
        self.in_flight = 5;

        // One task per source so each panel updates as soon as its fetch lands,
        // instead of waiting for the slowest of the five.
        macro_rules! spawn_fetch {
            ($update:expr, $fetch:path) => {{
                let cli = Arc::clone(cli);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let result = $fetch(&cli).await.ok();
                    let _ = tx.send($update(result));
                });
            }};
        }

        spawn_fetch!(|s| Update::Panel(0, s), fetchers::clusters::fetch);
        spawn_fetch!(|s| Update::Panel(1, s), fetchers::jobs::fetch);
        spawn_fetch!(|s| Update::Panel(2, s), fetchers::pipelines::fetch);
        spawn_fetch!(|s| Update::Panel(3, s), fetchers::warehouses::fetch);
        spawn_fetch!(Update::Badge, fetchers::current_user::fetch);
    }

    /// Applies any fetch results that have arrived; returns true if the UI should redraw.
    pub fn poll_refresh(&mut self) -> bool {
        let Some(rx) = &mut self.pending else {
            return false;
        };
        let mut changed = false;
        loop {
            match rx.try_recv() {
                // Keep the previous data when a fetch fails so panels don't blank out.
                Ok(Update::Panel(i, shape)) => {
                    if shape.is_some() {
                        self.shapes[i] = shape;
                    }
                    self.in_flight -= 1;
                    changed = true;
                }
                Ok(Update::Badge(badge)) => {
                    if badge.is_some() {
                        self.user_badge = badge;
                    }
                    self.in_flight -= 1;
                    changed = true;
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    self.in_flight = 0;
                    break;
                }
            }
        }
        if self.in_flight == 0 {
            self.loading = false;
            self.pending = None;
            changed = true;
        }
        changed
    }
}
