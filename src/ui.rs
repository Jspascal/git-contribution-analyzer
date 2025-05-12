use crate::{
    app::{App, AuthorSummary},
    git::Contribution,
};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn render_loading_screen(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &App) {
    let size = f.size();

    let block = Block::default()
        .title("Git Contribution Analyzer")
        .borders(Borders::ALL);
    f.render_widget(block, size);

    let loading_text = format!(
        "{} {}",
        app.loading_message,
        ".".repeat(((app.loading_progress % 4) + 1) as usize)
    );

    let loading_paragraph = Paragraph::new(loading_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default())
        .alignment(tui::layout::Alignment::Center);

    let loading_area = centered_rect(60, 20, size);
    f.render_widget(loading_paragraph, loading_area);
}

pub fn render_main_view(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &App) {
    let size = f.size();

    let main_block = Block::default()
        .title("Git Contribution Analyzer")
        .borders(Borders::ALL);
    f.render_widget(main_block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Help
        ])
        .split(size);

    let mut tab_titles = app
        .repositories
        .iter()
        .map(|repo| Spans::from(repo.clone()))
        .collect::<Vec<Spans>>();

    tab_titles.push(Spans::from("Summary"));

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title("Repositories"))
        .select(app.current_tab)
        .style(Style::default())
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, chunks[0]);

    if app.current_tab < app.repositories.len() {
        let repo_name = &app.repositories[app.current_tab];
        if let Some(contributions) = app.contributions.get(repo_name) {
            render_repository_tab(
                f,
                chunks[1],
                repo_name,
                contributions,
                app.selected_in_tab[app.current_tab],
            );
        }
    } else {
        render_summary_tab(
            f,
            chunks[1],
            &app.author_summaries,
            app.selected_in_tab[app.current_tab],
        );
    }

    if app.show_help {
        render_help(f, chunks[2]);
    } else {
        render_help_shortcut(f, chunks[2]);
    }
}

pub fn render_repository_tab(
    f: &mut Frame<CrosstermBackend<io::Stdout>>,
    area: Rect,
    repo_name: &str,
    contributions: &[Contribution],
    selected: Option<usize>,
) {
    let header_cells = [
        "Author",
        "Email",
        "Commits",
        "Lines Added",
        "Lines Deleted",
        "Contribution %",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells).style(Style::default()).height(1);

    let rows = contributions.iter().enumerate().map(|(i, c)| {
        let style = if Some(i) == selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let cells = [
            Cell::from(c.author.clone()),
            Cell::from(c.email.clone()),
            Cell::from(c.commits.to_string()),
            Cell::from(c.lines_added.to_string()),
            Cell::from(c.lines_deleted.to_string()),
            Cell::from(format!("{:.2}%", c.contribution_percent)),
        ];

        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title(format!("Repository: {}", repo_name))
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(10),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(14),
        ])
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_widget(table, area);
}

pub fn render_summary_tab(
    f: &mut Frame<CrosstermBackend<io::Stdout>>,
    area: Rect,
    summaries: &[AuthorSummary],
    selected: Option<usize>,
) {
    let header_cells = [
        "Author",
        "Email",
        "Total Commits",
        "Lines Added",
        "Lines Deleted",
        "Overall %",
        "Preferred Repo",
        "Preferred %",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells).style(Style::default()).height(1);

    let rows = summaries.iter().enumerate().map(|(i, s)| {
        let style = if Some(i) == selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let cells = [
            Cell::from(s.author.clone()),
            Cell::from(s.email.clone()),
            Cell::from(s.total_commits.to_string()),
            Cell::from(s.total_lines_added.to_string()),
            Cell::from(s.total_lines_deleted.to_string()),
            Cell::from(format!("{:.2}%", s.overall_contribution_percent)),
            Cell::from(s.preferred_repo.clone()),
            Cell::from(format!("{:.2}%", s.preferred_repo_percent)),
        ];

        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title("Summary Across All Repositories")
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
        ])
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_widget(table, area);
}

pub fn render_help_shortcut(f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
    let help_text = "Press '?' to show help";
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(help_paragraph, area);
}

pub fn render_help(f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect) {
    let help_text = vec![
        Spans::from("↑/↓: Navigate entries | Tab/Shift+Tab: Switch repositories"),
        Spans::from("?: Toggle help | q: Quit | h: Export HTML report"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default())
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Help"));

    f.render_widget(help_paragraph, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
