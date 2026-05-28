use crate::cli::DatabricksCli;
use crate::fetchers;
use crate::shape::Shape;
use anyhow::Result;
use std::time::{Duration, Instant};

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
}

pub struct App {
    pub focus: Panel,
    pub shapes: Vec<Option<Shape>>,
    pub user_badge: Option<Shape>,
    pub error: Option<String>,
    pub refresh_interval: Duration,
    last_refresh: Instant,
    pub loading: bool,
}

impl App {
    pub fn new(refresh_secs: u64) -> Self {
        Self {
            focus: Panel::Clusters,
            shapes: vec![None, None, None, None],
            user_badge: None,
            error: None,
            refresh_interval: Duration::from_secs(refresh_secs),
            last_refresh: Instant::now()
                .checked_sub(Duration::from_secs(refresh_secs + 1))
                .unwrap_or(Instant::now()),
            loading: false,
        }
    }

    pub fn focus_next(&mut self) {
        let idx = Panel::ALL.iter().position(|p| p == &self.focus).unwrap_or(0);
        self.focus = Panel::ALL[(idx + 1) % Panel::ALL.len()];
    }

    pub fn focus_prev(&mut self) {
        let idx = Panel::ALL.iter().position(|p| p == &self.focus).unwrap_or(0);
        self.focus = Panel::ALL[(idx + Panel::ALL.len() - 1) % Panel::ALL.len()];
    }

    pub fn needs_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.refresh_interval
    }

    pub async fn refresh(&mut self, cli: &DatabricksCli) -> Result<()> {
        self.loading = true;
        self.error = None;
        self.last_refresh = Instant::now();

        let (clusters, jobs, pipelines, warehouses, user) = tokio::join!(
            fetchers::clusters::fetch(cli),
            fetchers::jobs::fetch(cli),
            fetchers::pipelines::fetch(cli),
            fetchers::warehouses::fetch(cli),
            fetchers::current_user::fetch(cli),
        );

        self.shapes[0] = clusters.ok();
        self.shapes[1] = jobs.ok();
        self.shapes[2] = pipelines.ok();
        self.shapes[3] = warehouses.ok();
        self.user_badge = user.ok();
        self.loading = false;
        Ok(())
    }
}
