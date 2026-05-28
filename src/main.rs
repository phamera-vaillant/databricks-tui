use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use databricks_tui::{app::App, cli::DatabricksCli, ui};
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "databricks-tui", about = "Terminal dashboard for Databricks")]
struct Cli {
    #[arg(long, help = "Databricks CLI profile")]
    profile: Option<String>,

    #[arg(long, default_value = "30", help = "Auto-refresh interval in seconds")]
    refresh: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let cli = DatabricksCli::new(args.profile);
    let mut app = App::new(args.refresh);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app, &cli).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run(
    terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    cli: &DatabricksCli,
) -> Result<()> {
    loop {
        if app.needs_refresh() {
            app.refresh(cli).await.ok();
        }

        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                    (KeyCode::Tab, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                        app.focus_next()
                    }
                    (KeyCode::BackTab, _) | (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                        app.focus_prev()
                    }
                    (KeyCode::Char('r'), _) => {
                        app.refresh(cli).await.ok();
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
