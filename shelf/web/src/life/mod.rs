mod context;
mod draw;

book::error!(draw::Error, context::Error, crate::tree::Error);

use context::Context;

use std::sync::Arc;

pub fn handle(key: &str) -> Result<(), Error> {
    let context = Arc::new(Context::new());

    let set = crate::tree::root(&format!("canvas.{}", key))?;

    for tree in set {
        draw::handle(&tree, context.clone())?;
    }

    Ok(())
}
