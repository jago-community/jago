author::error!(
    Incomplete,
    url::ParseError,
    reqwest::Error,
    std::io::Error,
    crate::combinator::Error,
    futures::channel::mpsc::TrySendError<Url>,
    futures::channel::mpsc::TrySendError<String>,
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

use std::sync::Arc;

use crate::combinator;

use futures::Future;
use tokio::runtime::Runtime;
use url::Url;

fn search<'a>(input: &'a str, pattern: &'a str) -> Result<Vec<String>, Error> {
    let runtime = Runtime::new()?;

    runtime.block_on(handle_search(input, pattern))
}

use crate::sitemap;

use futures::{
    channel::mpsc,
    future,
    stream::{Stream, StreamExt},
};
use std::iter::{FromIterator, Iterator};

async fn handle_search<'a>(input: &'a str, _pattern: &'a str) -> Result<Vec<String>, Error> {
    let mut output: Vec<String> = vec![];

    let location = Url::parse(input)?;
    let robots_location = location.join("robots.txt")?;

    let robots = get_page(robots_location).await?;

    let sitemaps = combinator::with_fn(&robots, combinator::tagged_lines("sitemap: ")?)?;

    let (index_sender, mut index_receiver) = mpsc::unbounded();

    let getting_sitemaps = get_sitemaps(index_sender, location, sitemaps);

    let (sitemap_sender, mut sitemap_receiver) = mpsc::unbounded::<Url>();

    let getting_pages = async {
        //let (_, errors) =
        index_receiver
            .inspect(|h| {
                dbg!(h);
            })
            .map(|index| sitemap::parse(&index))
            .filter_map(|index| future::ready(index.ok()))
            .filter_map(|index| future::ready(sitemap::locations(&index).ok()))
            .flat_map(|locations| {
                futures::stream::FuturesUnordered::from_iter(
                    locations.into_iter().map(future::ready),
                )
            })
            .map(|location| {
                future::ready(
                    sitemap_sender
                        .unbounded_send(location.clone())
                        .map_err(Error::from),
                )
            })
            .for_each(|a| {
                dbg!(a);
                future::ready(())
            })
            .await;
    };

    let (got_result, _) = futures::future::join(getting_sitemaps, getting_pages).await;

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

        sender.unbounded_send(sitemap)?;
    }

    Ok(())
}

async fn get_page(location: Url) -> Result<String, Error> {
    let response = reqwest::get(location).await?;

    let body = response.text().await?;

    Ok(body)
}
