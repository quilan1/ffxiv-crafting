use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, FixedOffset};

use crate::tables::download_commits;

#[derive(serde::Deserialize)]
struct CommitList(Vec<CommitParent>);

#[derive(serde::Deserialize)]
struct CommitParent {
    commit: Commit,
}

#[derive(serde::Deserialize)]
struct Commit {
    author: CommitAuthor,
}

#[derive(serde::Deserialize)]
struct CommitAuthor {
    date: String,
}

pub async fn last_updated_from_github(file_name: &str) -> Result<DateTime<FixedOffset>> {
    let commits_str = download_commits(file_name).await?;
    let commits: CommitList = match serde_json::from_str(&commits_str) {
        Ok(commits) => commits,
        Err(err) => {
            println!("Github returned: {commits_str}");
            bail!(err);
        }
    };
    let err = anyhow!("No commits found for '{file_name}'");
    let date_str = &commits.0.first().ok_or(err)?.commit.author.date;
    Ok(DateTime::parse_from_rfc3339(date_str)?)
}
