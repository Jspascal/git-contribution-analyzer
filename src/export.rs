use crate::app::App;
use std::{error::Error, fs, path::Path};

pub fn export_html_report(app: &App, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Git Contribution Analysis Report</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 20px;
            color: #333;
        }
        h1, h2 {
            color: #2c3e50;
        }
        table {
            border-collapse: collapse;
            width: 100%;
            margin-bottom: 20px;
        }
        th, td {
            text-align: left;
            padding: 12px;
            border-bottom: 1px solid #ddd;
        }
        th {
            background-color: #f2f2f2;
            font-weight: bold;
        }
        tr:hover {
            background-color: #f5f5f5;
        }
        .report-date {
            color: #7f8c8d;
            font-style: italic;
            margin-bottom: 30px;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        .repo-section {
            margin-bottom: 40px;
            border: 1px solid #eee;
            padding: 20px;
            border-radius: 5px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Git Contribution Analysis Report</h1>
        <p class="report-date">Generated on: "#,
    )
    .to_string();

    use chrono::Local;
    html.push_str(&Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    html.push_str(
        r#"</p>
        
        <div class="repo-section">
            <h2>Summary Across All Repositories</h2>
            <table>
                <thead>
                    <tr>
                        <th>Author</th>
                        <th>Email</th>
                        <th>Total Commits</th>
                        <th>Lines Added</th>
                        <th>Lines Deleted</th>
                        <th>Overall %</th>
                        <th>Preferred Repo</th>
                        <th>Preferred %</th>
                    </tr>
                </thead>
                <tbody>
"#,
    );

    for summary in &app.author_summaries {
        html.push_str(&format!(
            r#"
                    <tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{:.2}%</td>
                        <td>{}</td>
                        <td>{:.2}%</td>
                    </tr>
"#,
            summary.author,
            summary.email,
            summary.total_commits,
            summary.total_lines_added,
            summary.total_lines_deleted,
            summary.overall_contribution_percent,
            summary.preferred_repo,
            summary.preferred_repo_percent
        ));
    }

    html.push_str(
        r#"
                </tbody>
            </table>
        </div>
"#,
    );

    for repo_name in &app.repositories {
        html.push_str(&format!(
            r#"
        <div class="repo-section">
            <h2>Repository: {}</h2>
            <table>
                <thead>
                    <tr>
                        <th>Author</th>
                        <th>Email</th>
                        <th>Commits</th>
                        <th>Lines Added</th>
                        <th>Lines Deleted</th>
                        <th>Contribution %</th>
                    </tr>
                </thead>
                <tbody>
"#,
            repo_name
        ));

        if let Some(contributions) = app.contributions.get(repo_name) {
            for contrib in contributions {
                html.push_str(&format!(
                    r#"
                    <tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{:.2}%</td>
                    </tr>
"#,
                    contrib.author,
                    contrib.email,
                    contrib.commits,
                    contrib.lines_added,
                    contrib.lines_deleted,
                    contrib.contribution_percent
                ));
            }
        }

        html.push_str(
            r#"
                </tbody>
            </table>
        </div>
"#,
        );
    }

    html.push_str(
        r#"
    </div>
</body>
</html>
"#,
    );

    fs::write(output_path, html)?;

    Ok(())
}
