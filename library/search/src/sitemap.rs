author::error!(NotFound, quick_xml::de::DeError);

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SitemapIndex {
    #[serde(rename = "sitemap")]
    sitemaps: Vec<Location>,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    loc: Option<String>,
}

use url::Url;

pub fn sitemaps<'a>(input: &'a str) -> Result<Vec<Url>, Error> {
    let index: SitemapIndex = quick_xml::de::from_str(input)?;

    Ok(index
        .sitemaps
        .iter()
        .filter_map(|sitemap| match &sitemap.loc {
            Some(location) => Url::parse(location).ok(),
            _ => None,
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct UrlSet {
    #[serde(default)]
    url: Vec<Location>,
    #[serde(default)]
    loc: Vec<Location>,
}

pub fn pages(input: &str) -> Result<Vec<Url>, Error> {
    let set: UrlSet = quick_xml::de::from_str(input)?;
    let locations = set.url.iter().chain(&set.loc);
    Ok(locations
        .filter_map(|sitemap| match &sitemap.loc {
            Some(location) => Url::parse(location).ok(),
            _ => None,
        })
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct Node {
    #[serde(rename = "sitemap")]
    sitemaps: Option<Vec<Node>>,
    #[serde(default)]
    urlset: Option<Vec<Node>>,
    #[serde(default)]
    url: Option<Vec<Node>>,
    #[serde(default)]
    loc: Option<Vec<Url>>,
}

pub fn parse(input: &str) -> Result<Node, Error> {
    let node: Node = quick_xml::de::from_str(input)?;

    Ok(node)
}

use std::iter::Iterator;

pub fn locations(node: &Node) -> Result<Vec<Url>, Error> {
    let mut output: Vec<Url> = vec![];

    if let Some(sitemaps) = &node.sitemaps {
        let locations: Vec<_> = sitemaps
            .iter()
            .map(locations)
            .filter_map(Result::ok)
            .flat_map(|a| a)
            .collect();
        output.extend_from_slice(&locations);
    }

    if let Some(urlset) = &node.urlset {
        let locations: Vec<_> = urlset
            .iter()
            .map(locations)
            .filter_map(Result::ok)
            .flat_map(|a| a)
            .collect();
        output.extend_from_slice(&locations);
    }

    if let Some(urls) = &node.url {
        let locations: Vec<_> = urls
            .iter()
            .map(locations)
            .filter_map(Result::ok)
            .flat_map(|a| a)
            .collect();
        output.extend_from_slice(&locations);
    }

    if let Some(locations) = &node.loc {
        output.extend_from_slice(&locations);
    }

    Ok(output)
}
