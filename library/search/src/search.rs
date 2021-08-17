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
        "https://shop.sportsbasement.com",
        "down-sweater-hoody279",
        vec!["https://shop.sportsbasement.com/products/down-sweater-hoody279"],
    )];

    for (input, pattern, want) in cases {
        let got = search(input, pattern).unwrap();
        assert_eq!(got, want);
    }
}

use std::{future::Future, sync::Arc};

use crate::combinator;

use tokio::{
    runtime::Runtime,
    sync::{mpsc, oneshot},
};
use url::Url;

fn search<'a>(input: &'a str, _pattern: &'a str) -> Result<Vec<String>, Error> {
    let runtime = Runtime::new()?;
    let runtime_handle = runtime.handle();

    let mut output: Vec<String> = vec![];

    let location = Url::parse(input)?;
    let robots_location = location.join("robots.txt")?;

    let robots = runtime_handle.block_on(get_page(robots_location))?;

    let sitemaps = combinator::with_fn(&robots, combinator::tagged_lines("sitemap: "))?;

    let mut blocks = vec![];

    for sitemap in sitemaps {
        let sitemap = Url::parse(sitemap)?;

        if sitemap.domain() != location.domain() {
            continue;
        }

        blocks.push(async {
            match reqwest::get(sitemap).await {
                Ok(response) => match response.text().await {
                    Ok(text) => match crate::sitemap::sitemaps(&text) {
                        Ok(sitemaps) => Ok(sitemaps),
                        Err(error) => Err(Error::from(error)),
                    },
                    Err(error) => Err(Error::from(error)),
                },
                Err(error) => Err(Error::from(error)),
            }
        });
    }

    unimplemented!()
}

use tokio::runtime::Handle;

async fn handle_search<'a>(input: &'a str, _pattern: &'a str) -> Result<Vec<String>, Error> {
    let mut output: Vec<String> = vec![];

    let location = Url::parse(input)?;
    let robots_location = location.join("robots.txt")?;

    let robots = get_page(robots_location).await?;

    let sitemaps = combinator::with_fn(&robots, combinator::tagged_lines("sitemap: "))?;

    let (sitemap_sender, mut sitemap_receiver) = mpsc::unbounded_channel();

    let _getting_sitemaps = get_sitemaps(sitemap_sender, location, sitemaps);

    while let Some(v) = sitemap_receiver.recv().await {
        println!("GOT = {:?}", v);
    }

    //sitemap_receiver
    //.for_each(|yep| futures::future::ready(()))
    //.await;

    unimplemented!()
}

async fn get_sitemaps(
    sender: mpsc::UnboundedSender<String>,
    location: Url,
    sitemaps: Vec<&str>,
) -> Result<(), Error> {
    for sitemap in sitemaps {
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
