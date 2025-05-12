use crate::git::Contribution;
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum AppState {
    Loading,
    Main,
}

pub struct App {
    pub state: AppState,
    pub repositories: Vec<String>,
    pub contributions: HashMap<String, Vec<Contribution>>,
    pub author_summaries: Vec<AuthorSummary>,
    pub current_tab: usize,
    pub selected_in_tab: Vec<Option<usize>>,
    pub loading_message: String,
    pub loading_progress: u8,
    pub show_help: bool,
    pub quit: bool,
}

#[derive(Debug, Clone)]
pub struct AuthorSummary {
    pub author: String,
    pub email: String,
    pub total_commits: u32,
    pub total_lines_added: u32,
    pub total_lines_deleted: u32,
    pub overall_contribution_percent: f64,
    pub preferred_repo: String,
    pub preferred_repo_percent: f64,
}

impl App {
    pub fn new() -> App {
        App {
            state: AppState::Loading,
            repositories: Vec::new(),
            contributions: HashMap::new(),
            author_summaries: Vec::new(),
            current_tab: 0,
            selected_in_tab: Vec::new(),
            loading_message: String::from("Initializing..."),
            loading_progress: 0,
            show_help: false,
            quit: false,
        }
    }

    pub fn next(&mut self) {
        if self.current_tab >= self.repositories.len() {
            if let Some(i) = self.selected_in_tab[self.current_tab] {
                if i >= self.author_summaries.len() - 1 {
                    self.selected_in_tab[self.current_tab] = Some(0);
                } else {
                    self.selected_in_tab[self.current_tab] = Some(i + 1);
                }
            } else if !self.author_summaries.is_empty() {
                self.selected_in_tab[self.current_tab] = Some(0);
            }
        } else {
            if let Some(i) = self.selected_in_tab[self.current_tab] {
                let repo_name = &self.repositories[self.current_tab];
                if let Some(repo_contribs) = self.contributions.get(repo_name) {
                    if i >= repo_contribs.len() - 1 {
                        self.selected_in_tab[self.current_tab] = Some(0);
                    } else {
                        self.selected_in_tab[self.current_tab] = Some(i + 1);
                    }
                }
            } else {
                let repo_name = &self.repositories[self.current_tab];
                if let Some(repo_contribs) = self.contributions.get(repo_name) {
                    if !repo_contribs.is_empty() {
                        self.selected_in_tab[self.current_tab] = Some(0);
                    }
                }
            }
        }
    }

    pub fn previous(&mut self) {
        if self.current_tab >= self.repositories.len() {
            if let Some(i) = self.selected_in_tab[self.current_tab] {
                if i == 0 {
                    self.selected_in_tab[self.current_tab] = Some(self.author_summaries.len() - 1);
                } else {
                    self.selected_in_tab[self.current_tab] = Some(i - 1);
                }
            } else if !self.author_summaries.is_empty() {
                self.selected_in_tab[self.current_tab] = Some(self.author_summaries.len() - 1);
            }
        } else {
            if let Some(i) = self.selected_in_tab[self.current_tab] {
                let repo_name = &self.repositories[self.current_tab];
                if let Some(repo_contribs) = self.contributions.get(repo_name) {
                    if i == 0 {
                        self.selected_in_tab[self.current_tab] = Some(repo_contribs.len() - 1);
                    } else {
                        self.selected_in_tab[self.current_tab] = Some(i - 1);
                    }
                }
            } else {
                let repo_name = &self.repositories[self.current_tab];
                if let Some(repo_contribs) = self.contributions.get(repo_name) {
                    if !repo_contribs.is_empty() {
                        self.selected_in_tab[self.current_tab] = Some(repo_contribs.len() - 1);
                    }
                }
            }
        }
    }

    pub fn next_tab(&mut self) {
        let tab_count = self.repositories.len() + 1;
        self.current_tab = (self.current_tab + 1) % tab_count;
    }

    pub fn previous_tab(&mut self) {
        let tab_count = self.repositories.len() + 1;
        self.current_tab = (self.current_tab + tab_count - 1) % tab_count;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}
