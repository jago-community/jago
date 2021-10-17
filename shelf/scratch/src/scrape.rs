#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Request {0}")]
    Request(#[from] reqwest::Error),
}

#[test]
fn test_scrape() {
    let input = r#"<!DOCTYPE html>
    <html>
        <head>
            <title>Hello, stranger</title>
        </head>
        <body>
            <h1>
                An ode to math.
            </h1>
            <p>
                Te gusta bailar.
            </p>

            <pre>
                Book 

                Hello stranger

                Here's mental gymnastics

                An ode to math

                It is what it is

                Life.
            </pre>
        </body>
    </html>"#;

    let _got = handle(input).unwrap();
}

use crate::{Context, Key, Scratch};

use scraper::{Html, Node, Selector};

impl Scratch {
    async fn scrape(&mut self, context: &Context, address: &str) -> Result<(), Error> {
        let document = reqwest::get(address).await?.text().await?;

        let document = Html::parse_document(&document);

        let mut surface = String::new();

        for text in document.root_element().text() {
            let text = text.trim();
            if text.len() > 0 {
                surface.push_str(text);
            }
        }

        let key = address.as_bytes();

        self.fill(context, &key.into(), surface);

        Ok(())
    }
}
