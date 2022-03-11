#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

use dioxus::prelude::*;

use ::{context::Context, instrument::prelude::*};

pub fn watch(context: Context) -> Result<(), Error> {
    warn!("launching: {}", context);

    dioxus::web::launch_with_props(app, context.into(), |c| c);

    Ok(())
}

fn app(scope: Scope<Context>) -> Element {
    let context = scope.props;

    scope.render(rsx! {
        div { "{context}" }
    })
}
