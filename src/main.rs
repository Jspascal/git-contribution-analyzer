use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::HashMap,
    error::Error,
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use tui::{backend::CrosstermBackend, Terminal};

use git_contribution_analyzer::{
    app::{App, AppState},
    error::io_err_to_box_err,
    export::export_html_report,
    git::{analyze_repository, calculate_author_summaries, find_repositories},
    ui::{render_loading_screen, render_main_view},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long)]
    path: PathBuf,

    /// Repository pattern to match (e.g., "bwt-*")
    #[arg(short, long, default_value = "*")]
    pattern: String,
}

fn main() -> Result<(), Box<dyn Error + Send>> {
    let args = CliArgs::parse();
    let parent_path = args.path.clone();
    let pattern = args.pattern.clone();

    enable_raw_mode().map_err(io_err_to_box_err)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(io_err_to_box_err)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(io_err_to_box_err)?;

    let app = Arc::new(Mutex::new(App::new()));
    let app_ui = Arc::clone(&app);

    let loading_thread = thread::spawn(move || -> Result<(), Box<dyn Error + Send>> {
        {
            let mut guard = app.lock().map_err(|_| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire lock".to_string(),
                )) as Box<dyn Error + Send>
            })?;
            guard.loading_message = String::from("Finding Git repositories");
        }

        let repositories = find_repositories(&parent_path, &pattern)?;

        if repositories.is_empty() {
            let mut guard = app.lock().map_err(|_| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire lock".to_string(),
                )) as Box<dyn Error + Send>
            })?;
            guard.loading_message = String::from("No Git repositories found!");
            thread::sleep(std::time::Duration::from_secs(2));
            guard.state = AppState::Main;
            return Ok(());
        }

        let repo_count = repositories.len();
        let mut repository_names = Vec::new();
        let mut contributions_map = HashMap::new();

        for (index, repo_path) in repositories.iter().enumerate() {
            let repo_name = repo_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            {
                let mut guard = app.lock().map_err(|_| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to acquire mutex lock".to_string(),
                    )) as Box<dyn Error + Send>
                })?;
                guard.loading_message = format!(
                    "Analyzing repository {}/{}: {}",
                    index + 1,
                    repo_count,
                    repo_name
                );
                guard.loading_progress = ((index as f32 / repo_count as f32) * 100.0) as u8;
            }

            match analyze_repository(repo_path) {
                Ok((name, contributions)) => {
                    repository_names.push(name.clone());
                    contributions_map.insert(name, contributions);
                }
                Err(e) => {
                    eprintln!("Error analyzing repository {}: {}", repo_name, e);
                }
            }
        }

        repository_names.sort();

        let author_summaries = calculate_author_summaries(&contributions_map);

        {
            let mut guard = app.lock().map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to acquire mutex lock: {}", e),
                )) as Box<dyn Error + Send>
            })?;
            guard.repositories = repository_names;
            guard.contributions = contributions_map;
            guard.author_summaries = author_summaries;
            guard.selected_in_tab = vec![None; guard.repositories.len() + 1];
            guard.state = AppState::Main;
        }

        Ok(())
    });

    let mut last_tick = std::time::Instant::now();
    let tick_rate = std::time::Duration::from_millis(100);
    let mut loading_thread = Some(loading_thread);
    let mut loading_thread_complete = false;

    loop {
        terminal
            .draw(|f| {
                if let Ok(guard) = app_ui.lock() {
                    match guard.state {
                        AppState::Loading => render_loading_screen(f, &guard),
                        AppState::Main => render_main_view(f, &guard),
                    }
                }
            })
            .map_err(io_err_to_box_err)?;

        if !loading_thread_complete {
            if let Ok(guard) = app_ui.lock() {
                if guard.state == AppState::Main {
                    if let Some(thread) = loading_thread.take() {
                        if let Err(e) = thread.join() {
                            eprintln!("Loading thread error: {:?}", e);
                        }
                    }
                    loading_thread_complete = true;
                }
            }
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        if event::poll(timeout).map_err(io_err_to_box_err)? {
            if let Event::Key(key) = event::read().map_err(io_err_to_box_err)? {
                if let Ok(mut guard) = app_ui.lock() {
                    if guard.state == AppState::Main {
                        match key.code {
                            KeyCode::Char('q') => {
                                guard.quit = true;
                            }
                            KeyCode::Char('?') => guard.toggle_help(),
                            KeyCode::Char('h') => {
                                let output_path = PathBuf::from("git_contribution_report.html");
                                match export_html_report(&guard, &output_path) {
                                    Ok(_) => {
                                        guard.loading_message =
                                            format!("Report exported to {}", output_path.display());
                                    }
                                    Err(e) => {
                                        guard.loading_message =
                                            format!("Error exporting report: {}", e);
                                    }
                                }
                            }
                            KeyCode::Down => guard.next(),
                            KeyCode::Up => guard.previous(),
                            KeyCode::Tab => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    guard.previous_tab();
                                } else {
                                    guard.next_tab();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Check quit condition
        if let Ok(guard) = app_ui.lock() {
            if guard.quit {
                break;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let Ok(mut guard) = app_ui.lock() {
                if guard.state == AppState::Loading {
                    guard.loading_progress = (guard.loading_progress + 1) % 100;
                }
            }
            last_tick = std::time::Instant::now();
        }
    }

    disable_raw_mode().map_err(io_err_to_box_err)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(io_err_to_box_err)?;
    terminal.show_cursor().map_err(io_err_to_box_err)?;

    Ok(())
}
