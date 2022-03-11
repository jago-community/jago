#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Logs {0}")]
    Logs(#[from] logs::Error),
}

use context::Context;

pub fn watch(context: Context) -> Result<(), Error> {
    log::info!("launching 🧨 {}", context);

    //dioxus::web::launch_with_props(app, context.into(), |config| config.rootname("context"));

    Ok(())
}

use dioxus::prelude::*;

fn app(scope: Scope<Context>) -> Element {
    let context = scope.props;

    scope.render(rsx! {
        div { "Hello: {context}" }
    })
}
