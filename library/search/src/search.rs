author::error!(
    Incomplete,
    url::ParseError,
    reqwest::Error,
    std::io::Error,
    crate::combinator::Error,
    tokio::sync::oneshot::error::RecvError,
    crate::sitemap::Error,
);

#[test]
fn test_search() {
    let cases = vec![(
        "https://www.patagonia.com",
        "20050",
        vec!["https://www.patagonia.com/product/all-seasons-hemp-canvas-work-apron/20050.html"],
    )];

    for (input, pattern, want) in cases {
        let got = search(input, pattern).unwrap();
        assert_eq!(got, want);
    }
}

use std::{future::Future, sync::Arc};

use crate::combinator;

use tokio::{runtime::Runtime, sync::mpsc};
use url::Url;

fn search<'a>(input: &'a str, pattern: &'a str) -> Result<Vec<String>, Error> {
    let runtime = Runtime::new()?;

    runtime.block_on(handle_search(input, pattern))
}

async fn handle_search<'a>(input: &'a str, _pattern: &'a str) -> Result<Vec<String>, Error> {
    let mut output: Vec<String> = vec![];

    let location = Url::parse(input)?;
    let robots_location = location.join("robots.txt")?;

    let robots = get_page(robots_location).await?;

    let sitemaps = combinator::with_fn(&robots, combinator::tagged_lines("sitemap: ")?)?;

    let (sitemap_sender, mut sitemap_receiver) = mpsc::unbounded_channel();

    let getting_sitemaps = get_sitemaps(sitemap_sender, location, sitemaps);

    let other = async {
        while let Some(v) = sitemap_receiver.recv().await {
            println!("GOT = {:?}", v);
        }
    };

    let (got_result, _) = futures::future::join(getting_sitemaps, other).await;

    got_result?;

    unimplemented!()
}

async fn get_sitemaps(
    sender: mpsc::UnboundedSender<String>,
    location: Url,
    sitemaps: Vec<&str>,
) -> Result<(), Error> {
    for sitemap in sitemaps {
        dbg!(&sitemap);

        let sitemap = Url::parse(sitemap)?;

        if sitemap.domain() != location.domain() {
            continue;
        }

        let sitemap: String = get_page(sitemap).await?;

        if let Err(_) = sender.send(sitemap) {
            log::error!("sitemap_receiver dropped");
        }
    }

    Ok(())
}

async fn get_page(location: Url) -> Result<String, Error> {
    let response = reqwest::get(location).await?;

    let body = response.text().await?;

    Ok(body)
}
