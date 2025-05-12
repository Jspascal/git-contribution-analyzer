use glob::glob;
use itertools::Itertools;
use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    process::Command,
};

use crate::app::AuthorSummary;

#[derive(Debug, Clone)]
pub struct Contribution {
    pub author: String,
    pub email: String,
    pub commits: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub contribution_percent: f64,
    pub repository: String,
}

pub fn is_git_repository(path: &Path) -> bool {
    let git_dir = path.join(".git");
    git_dir.exists() && git_dir.is_dir()
}

pub fn find_repositories(
    parent_path: &Path,
    pattern: &str,
) -> Result<Vec<PathBuf>, Box<dyn Error + Send>> {
    let mut repositories = Vec::new();
    let pattern_path = parent_path.join(pattern);
    let pattern_str = pattern_path.to_string_lossy().to_string();

    for entry in glob(&pattern_str).map_err(|e| Box::new(e) as Box<dyn Error + Send>)? {
        match entry {
            Ok(path) => {
                if path.is_dir() && is_git_repository(&path) {
                    repositories.push(path);
                }
            }
            Err(e) => eprintln!("Error matching path: {}", e),
        }
    }

    Ok(repositories)
}

pub fn analyze_repository(repo_path: &Path) -> Result<(String, Vec<Contribution>), Box<dyn Error>> {
    let repo_name = repo_path
        .file_name()
        .ok_or("Invalid repository path")?
        .to_string_lossy()
        .to_string();

    let mut contributions = Vec::new();

    let total_output = Command::new("git")
        .args(["log", "--no-merges", "--numstat"])
        .current_dir(repo_path)
        .output()?
        .stdout;

    let total_lines = String::from_utf8_lossy(&total_output);
    let mut total_lines_changed = 0;

    for line in total_lines.lines() {
        if let Some((added, deleted, _)) = line.split_whitespace().collect_tuple() {
            if added != "-" && deleted != "-" {
                if let (Ok(a), Ok(d)) = (added.parse::<u32>(), deleted.parse::<u32>()) {
                    total_lines_changed += a + d;
                }
            }
        }
    }

    let authors_output = Command::new("git")
        .args(["log", "--no-merges", "--format=%ae|%an"])
        .current_dir(repo_path)
        .output()?
        .stdout;

    let authors = String::from_utf8_lossy(&authors_output);

    let mut author_map = HashMap::new();

    for line in authors.lines() {
        if let Some((email, name)) = line.split_once('|') {
            author_map
                .entry(email.to_string())
                .or_insert_with(|| name.to_string());
        }
    }

    for (email, name) in author_map {
        let commits = Command::new("git")
            .args(["log", "--no-merges", "--author", &email, "--format=%H"])
            .current_dir(repo_path)
            .output()?
            .stdout;

        let commit_count = String::from_utf8_lossy(&commits).lines().count() as u32;

        let stats_output = Command::new("git")
            .args([
                "log",
                "--no-merges",
                "--author",
                &email,
                "--numstat",
                "--pretty=format:",
            ])
            .current_dir(repo_path)
            .output()?
            .stdout;

        let stats_str = String::from_utf8_lossy(&stats_output);

        let mut lines_added = 0;
        let mut lines_deleted = 0;

        for line in stats_str.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some((added, deleted, _)) = line.split_whitespace().collect_tuple() {
                if added != "-" && deleted != "-" {
                    if let (Ok(a), Ok(d)) = (added.parse::<u32>(), deleted.parse::<u32>()) {
                        lines_added += a;
                        lines_deleted += d;
                    }
                }
            }
        }

        let lines_changed = lines_added + lines_deleted;
        let contribution_percent = if total_lines_changed > 0 {
            (lines_changed as f64 / total_lines_changed as f64) * 100.0
        } else {
            0.0
        };

        contributions.push(Contribution {
            author: name,
            email,
            commits: commit_count,
            lines_added,
            lines_deleted,
            contribution_percent,
            repository: repo_name.clone(),
        });
    }

    contributions.sort_by(|a, b| {
        b.contribution_percent
            .partial_cmp(&a.contribution_percent)
            .unwrap()
    });

    Ok((repo_name, contributions))
}

pub fn calculate_author_summaries(
    contributions_map: &HashMap<String, Vec<Contribution>>,
) -> Vec<AuthorSummary> {
    let mut author_data: HashMap<String, (String, String, u32, u32, u32, HashMap<String, f64>)> =
        HashMap::new();
    let mut total_lines_changed_all_repos = 0;

    for (repo_name, contributions) in contributions_map {
        for contrib in contributions {
            let email = &contrib.email;
            let author_name = &contrib.author;
            let lines_changed = contrib.lines_added + contrib.lines_deleted;

            total_lines_changed_all_repos += lines_changed;

            let entry = author_data
                .entry(email.clone())
                .or_insert_with(|| (author_name.clone(), email.clone(), 0, 0, 0, HashMap::new()));

            entry.2 += contrib.commits;
            entry.3 += contrib.lines_added;
            entry.4 += contrib.lines_deleted;
            entry
                .5
                .insert(repo_name.clone(), contrib.contribution_percent);
        }
    }

    let mut summaries = Vec::new();

    for (email, (author, _, commits, lines_added, lines_deleted, repo_percentages)) in author_data {
        let total_lines_changed = lines_added + lines_deleted;
        let overall_percent = if total_lines_changed_all_repos > 0 {
            (total_lines_changed as f64 / total_lines_changed_all_repos as f64) * 100.0
        } else {
            0.0
        };

        let mut preferred_repo = String::new();
        let mut highest_percent = 0.0;

        for (repo, percent) in &repo_percentages {
            if *percent > highest_percent {
                highest_percent = *percent;
                preferred_repo = repo.clone();
            }
        }

        summaries.push(AuthorSummary {
            author,
            email,
            total_commits: commits,
            total_lines_added: lines_added,
            total_lines_deleted: lines_deleted,
            overall_contribution_percent: overall_percent,
            preferred_repo,
            preferred_repo_percent: highest_percent,
        });
    }

    summaries.sort_by(|a, b| {
        b.overall_contribution_percent
            .partial_cmp(&a.overall_contribution_percent)
            .unwrap()
    });

    summaries
}
