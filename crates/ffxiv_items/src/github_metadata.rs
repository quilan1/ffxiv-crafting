use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, FixedOffset};

use mock_traits::FileDownloader;

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

pub async fn last_updated_from_github<F: FileDownloader>(
    file_name: &str,
) -> Result<DateTime<FixedOffset>> {
    let commits_str = download_commits::<F>(file_name).await?;
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

pub async fn download_commits<F: FileDownloader>(file_name: &str) -> Result<String> {
    F::download(&format!(
        "https://api.github.com/repos/xivapi/ffxiv-datamining/commits?path=csv/{file_name}&page=1&page_per=1"
    )).await
}
