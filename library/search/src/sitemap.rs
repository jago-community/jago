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
    #[serde(rename = "url")]
    locations: Vec<Location>,
}

pub fn pages(input: &str) -> Result<Vec<Url>, Error> {
    let set: UrlSet = quick_xml::de::from_str(input)?;
    Ok(set
        .locations
        .iter()
        .filter_map(|sitemap| match &sitemap.loc {
            Some(location) => Url::parse(location).ok(),
            _ => None,
        })
        .collect())
}
