author::error!(
    Incomplete,
    url::ParseError,
    reqwest::Error,
    std::io::Error,
    crate::combinator::Error,
    TrySendUrl(futures::channel::mpsc::TrySendError<Url>),
    TrySendString(futures::channel::mpsc::TrySendError<String>),
    crate::sitemap::Error,
    regex::Error,
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

use std::iter::{FromIterator, Iterator};

use crate::sitemap;

use futures::{
    channel::mpsc,
    future::{self, Either},
    stream::{self, Stream, StreamExt, TryStreamExt},
    FutureExt,
};
use regex::RegexBuilder;

async fn handle_search<'a>(input: &'a str, pattern: &'a str) -> Result<Vec<String>, Error> {
    //let pattern_matcher = combinator::matched(pattern)?;
    //let pattern_matcher = combinator::matched(pattern)?;

    let pattern = RegexBuilder::new(pattern).case_insensitive(true).build()?;

    let location = Url::parse(input)?;
    let robots_location = location.join("robots.txt")?;

    let robots = get_page(robots_location).await?;

    // TODO: make lazy
    let sitemap_pattern = RegexBuilder::new("sitemap:")
        .case_insensitive(true)
        .build()?;

    let sitemaps = combinator::with_fn(&robots, combinator::tagged_lines(sitemap_pattern))?;

    let results = stream::iter(sitemaps)
        .map(Url::parse)
        .map_err(Error::from)
        // START sitemap index
        .map_ok(get_page)
        .then(|result| async move {
            match result {
                Ok(page) => page.await,
                Err(error) => Err(error),
            }
        })
        .map(|result| match result {
            Ok(index) => sitemap::parse(&index).map_err(Error::from),
            Err(error) => Err(error),
        })
        .map(|result| match result {
            Ok(node) => sitemap::locations(&node).map_err(Error::from),
            Err(error) => Err(error),
        })
        .flat_map(|result| match result {
            Ok(locations) => futures::stream::FuturesUnordered::from_iter(
                locations.into_iter().map(Ok).map(future::ready),
            ),
            Err(error) => futures::stream::FuturesUnordered::from_iter(
                vec![Result::<Url, Error>::Err(error)]
                    .into_iter()
                    .map(future::ready),
            ),
        })
        // END sitemap index
        // START addresses
        .map_ok(get_page)
        .then(|result| async move {
            match result {
                Ok(page) => page.await,
                Err(error) => Err(error),
            }
        })
        .map(|result| match result {
            Ok(index) => sitemap::parse(&index).map_err(Error::from),
            Err(error) => Err(error),
        })
        .map(|result| match result {
            Ok(node) => sitemap::locations(&node).map_err(Error::from),
            Err(error) => Err(error),
        })
        .flat_map(|result| match result {
            Ok(locations) => futures::stream::FuturesUnordered::from_iter(
                locations.into_iter().map(Ok).map(future::ready),
            ),
            Err(error) => futures::stream::FuturesUnordered::from_iter(
                vec![Result::<Url, Error>::Err(error)]
                    .into_iter()
                    .map(future::ready),
            ),
        })
        // END addresses
        // START addresses match pattern
        .map_ok(|address| {
            let path = address.path();
            let matched = combinator::with_fn(path, combinator::matched(pattern.clone())).is_ok();

            if matched {
                Either::Left(address.clone())
            } else {
                Either::Right(address.clone())
            }
        })
        .fold(
            (vec![], vec![], vec![]),
            |(mut matched_addresses, mut unmatched_addresses, mut errors), result| async move {
                match result {
                    Ok(either) => match either {
                        Either::Left(left) => matched_addresses.push(left),
                        Either::Right(right) => unmatched_addresses.push(right),
                    },
                    Err(error) => errors.push(error),
                };

                (matched_addresses, unmatched_addresses, errors)
            },
        )
        .await;

    dbg!(results.0);
    dbg!(results.1.len());
    dbg!(results.2);

    /*
    let (index_sender, mut index_receiver) = mpsc::unbounded();

    let getting_index = get_sitemaps(index_sender, location, sitemaps);

    let (sitemap_sender, mut sitemap_receiver) = mpsc::unbounded::<Url>();

    let getting_sitemaps = async {
        index_receiver
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
                        .map_err(Error::TrySendUrl),
                )
            })
            .collect::<Vec<_>>()
            .await
    };

    //let (page_sender, mut page_receiver) = mpsc::unbounded::<String>();

    let getting_pages = async {
        sitemap_receiver
            .map(|location| async move { reqwest::get(location).await })
            .filter_map(|result| async move { result.await.ok() })
            .map(|response| async move { response.text().await })
            .filter_map(|result| async move { result.await.ok() })
            .map(|sitemap| sitemap::parse(&sitemap))
            .filter_map(|index| future::ready(index.ok()))
            .filter_map(|index| future::ready(sitemap::locations(&index).ok()))
            .flat_map(|locations| {
                futures::stream::FuturesUnordered::from_iter(
                    locations.into_iter().map(future::ready),
                )
            })
            .inspect(|h| {
                dbg!(h);
            })
            //.map(|location| {
            //future::ready(
            //sitemap_sender
            //.unbounded_send(location.clone())
            //.map_err(Error::TrySendUrl),
            //)
            //})
            .collect::<Vec<_>>()
            .await
    };

    let (got_result, pages) = futures::future::join(getting_index, getting_pages).await;

    got_result?;
    */

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

        sender
            .unbounded_send(sitemap)
            .map_err(Error::TrySendString)?;
    }

    Ok(())
}

async fn get_page(location: Url) -> Result<String, Error> {
    let response = reqwest::get(location).await?;

    let body = response.text().await?;

    Ok(body)
}
