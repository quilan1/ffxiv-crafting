use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use reqwest::header::{HeaderMap, HeaderValue};

pub trait FileDownloader {
    fn download(url: &str) -> BoxFuture<'_, Result<String>>;
}

pub struct GithubDownloader;

impl FileDownloader for GithubDownloader {
    fn download(url: &str) -> BoxFuture<'_, Result<String>> {
        async fn inner(url: &str) -> Result<String> {
            let mut headers = HeaderMap::new();
            headers.insert("User-Agent", HeaderValue::from_static("Tiny Rust Program"));
            let client = reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()?;
            let request = client.get(url).build()?;
            Ok(client.execute(request).await?.text().await?)
        }
        inner(url).boxed()
    }
}

pub struct ReqwestDownloader;

impl FileDownloader for ReqwestDownloader {
    fn download(url: &str) -> BoxFuture<'_, Result<String>> {
        async fn inner(url: &str) -> Result<String> {
            Ok(reqwest::get(url).await?.text().await?)
        }
        inner(url).boxed()
    }
}

pub struct FaultyDownloader<const CHANCE: u32>;

impl<const CHANCE: u32> FileDownloader for FaultyDownloader<CHANCE> {
    fn download(url: &str) -> BoxFuture<'_, Result<String>> {
        async fn inner(url: &str, chance: u32) -> Result<String> {
            use rand::Rng;
            if rand::thread_rng().gen::<u32>() % 100 < chance {
                Ok("".into())
            } else {
                Ok(reqwest::get(url).await?.text().await?)
            }
        }
        inner(url, CHANCE).boxed()
    }
}