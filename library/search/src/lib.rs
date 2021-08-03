mod sink;

author::error!(Incomplete, std::io::Error, reqwest::Error, url::ParseError);

use std::iter::Peekable;

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
fn test_search() {
    let cases = vec![(
        "https://garageclothing.com",
        "100048721",
        "https://www.garageclothing.com/p/elastic-waist-half-zip-sweatshirt-/100048721.html",
    )];

    for (source, identifier, want) in cases {
        let got = handle_search(source, Some(identifier)).unwrap();
        assert_eq!(got, want);
    }
}

fn handle_search(source: &str, input: Option<&str>) -> Result<String, Error> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async { search(source, input).await })
}

use url::Url;

async fn search(source: &str, input: Option<&str>) -> Result<String, Error> {
    let location = Url::parse(source)?;

    let robots = reqwest::get(location.join("robots.txt")?)
        .await?
        .text()
        .await?;

    let sitemaps = robots
        .lines()
        .filter_map(|line| line.strip_prefix("sitemap:"))
        .map(|matched| matched.trim_start());

    let sitemaps = sitemaps
        .filter_map(|sitemap| Url::parse(sitemap).ok())
        .filter(|sitemap| sitemap.domain() == location.domain());

    let sitemaps = sitemaps
        .filter_map(|sitemap| futures::executor::block_on(reqwest::get(sitemap)).ok())
        .filter_map(|response| futures::executor::block_on(response.text()).ok());

    let sitemaps = sitemaps.map(|sitemap| quick_xml::de::from_str(&sitemap));

    println!("{:?}", sitemaps.collect::<Vec<Sitemap>>());

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

    Ok("nope".into())
}

use serde::Deserialize;

#[derive(Deserialize)]
struct Sitemap {
    loc: String,
}

impl Sitemap {
    fn location(&self) -> Result<Url, Error> {
        self.loc.parse().map_err(Error::from)
    }
}
