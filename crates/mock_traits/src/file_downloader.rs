use std::{marker::PhantomData, time::Duration};

use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use reqwest::header::{HeaderMap, HeaderValue};
use tokio::time::sleep;

////////////////////////////////////////////////////////////

pub trait FileDownloader {
    fn download(url: &str) -> BoxFuture<'_, Result<String>>;
}

////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////

pub struct ReqwestDownloader;

impl FileDownloader for ReqwestDownloader {
    fn download(url: &str) -> BoxFuture<'_, Result<String>> {
        async fn inner(url: &str) -> Result<String> {
            Ok(reqwest::get(url).await?.text().await?)
        }
        inner(url).boxed()
    }
}

////////////////////////////////////////////////////////////

pub struct FaultyDownloader<const CHANCE: u32, F: FileDownloader = ReqwestDownloader> {
    marker: PhantomData<F>,
}

impl<const CHANCE: u32, F: FileDownloader + 'static> FileDownloader
    for FaultyDownloader<CHANCE, F>
{
    fn download(url: &str) -> BoxFuture<'_, Result<String>> {
        async fn inner<F: FileDownloader>(url: &str, chance: u32) -> Result<String> {
            use rand::Rng;
            if rand::thread_rng().gen::<u32>() % 100 < chance {
                Ok("".into())
            } else {
                F::download(url).await
            }
        }
        inner::<F>(url, CHANCE).boxed()
    }
}

////////////////////////////////////////////////////////////

pub struct DelayedDownloader<const DELAY: u32, F: FileDownloader = ReqwestDownloader> {
    marker: PhantomData<F>,
}

impl<const DELAY: u32, F: FileDownloader + 'static> FileDownloader for DelayedDownloader<DELAY, F> {
    fn download(url: &str) -> BoxFuture<'_, Result<String>> {
        async fn inner<F: FileDownloader>(url: &str, delay: u32) -> Result<String> {
            sleep(Duration::from_millis(delay as u64)).await;
            F::download(url).await
        }
        inner::<F>(url, DELAY).boxed()
    }
}
