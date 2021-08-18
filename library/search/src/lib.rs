mod combinator;
mod search;
mod sitemap;

author::error!(Incomplete, std::io::Error, reqwest::Error, url::ParseError,);

use std::iter::Peekable;

/*
pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "search" => {
            let _ = input.next();

            if let Some(source) = input.next() {
                handle_search(&source, input.next().as_deref()).map(|_| ())
            } else {
                Err(Error::Incomplete)
            }
        }
        _ => Err(Error::Incomplete),
    }
}

#[test]
#[ignore]
fn test_search() {
    let cases = vec![(
        "https://garageclothing.com",
        "100048721",
        vec!["https://www.garageclothing.com/p/elastic-waist-half-zip-sweatshirt-/100048721.html"],
    )];

    for (source, identifier, want) in cases {
        let got = handle_search(source, Some(identifier)).unwrap();
        assert_eq!(got, want);
    }
}

fn handle_search(source: &str, input: Option<&str>) -> Result<Vec<String>, Error> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async { search(source, input).await })
}

use url::Url;

async fn search(source: &str, input: Option<&str>) -> Result<Vec<String>, Error> {
    let location = Url::parse(source)?;

    let robots = reqwest::get(location.join("robots.txt")?)
        .await?
        .text()
        .await?;

    let sitemaps = combinator::with_fn(&robots, combinator::tagged_lines("sitemap: "))?;

    let requests = sitemaps
        .iter()
        .filter_map(|sitemap| Url::parse(sitemap).ok())
        .filter(|sitemap| sitemap.domain() == location.domain())
        .map(|sitemap| reqwest::get(sitemap));

    let mut sitemaps = vec![];

    for response in requests {
        sitemaps.push(response.await?.text().await?);
    }

    let requests = sitemaps
        .iter()
        .filter_map(|sitemap| sitemap::sitemaps(&sitemap).ok())
        .flatten()
        .map(|sitemap| reqwest::get(sitemap));

    let mut sitemaps = vec![];

    for response in requests {
        sitemaps.push(response.await?.text().await?);
    }

    let pages = sitemaps
        .iter()
        .filter_map(|sitemap| sitemap::pages(&sitemap).ok())
        .flatten()
        .collect::<Vec<_>>();

    if let Some(term) = input {
        let matched_pages = pages
            .iter()
            .filter_map(|location| {
                combinator::with_fn(location.path(), combinator::matched(term))
                    .ok()
                    .map(move |_| location.as_str().into())
            })
            .collect::<Vec<_>>();

        if matched_pages.len() > 0 {
            return Ok(matched_pages);
        }
    }

    let requests = pages.iter().cloned().map(|page| reqwest::get(dbg!(page)));

    let mut pages = vec![];

    for response in requests {
        pages.push(response.await?.text().await?);

        if pages.len() == 1 {
            break;
        }
    }

    //
    // let sitemaps = parse_robots(buffer);
    //
    // sitemaps.filter(|sitemap| sitemap.starts_with(source));
    //
    // let pages = sitemaps
    //   .map(|sitemap| parse_sitemap(sitemap))
    //   .filter_map(|parsed| parsed.ok())
    //   .flatten();
    //
    // if let Some(input) = input {
    //   if let Some(matched_page) = pages.par_iter().find_any(|page| {
    //     page.contains(input)
    //   }) {
    //     return Ok(matched_page);
    //   }
    //
    //   // search each page as html
    // }
    //

    Ok(vec!["nope".into()])
}
*/
