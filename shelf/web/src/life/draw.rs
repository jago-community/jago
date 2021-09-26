#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoWindow")]
    NoWindow,
    #[error("NoContext")]
    NoContext,
    #[error("External")]
    External(wasm_bindgen::JsValue),
    #[error("Conversion error between external items")]
    Conversion,
    #[error("Tree")]
    Tree(#[from] crate::tree::Error),
}

use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use super::context::Context;

pub fn handle(node: &web_sys::Node, context: Arc<Context>) -> Result<(), Error> {
    let height = context.height();
    let width = context.width();

    let canvas = node
        .dyn_ref::<web_sys::HtmlCanvasElement>()
        .map_or(Err(Error::Conversion), Ok)?;

    let cell_size = 12;

    let (_, body) = crate::tree::context()?;

    canvas.set_height((cell_size + 1) * height + 1);
    canvas.set_width((cell_size + 1) * width + 1);

    //canvas.set_height((cell_size + 1) * height + 1);
    //canvas.set_width((cell_size + 1) * width + 1);

    let render_context =
        canvas
            .get_context("2d")
            .map_err(Error::External)
            .and_then(|maybe_context| {
                maybe_context.map_or(Err(Error::NoContext), |context| {
                    context
                        .dyn_into::<web_sys::CanvasRenderingContext2d>()
                        .map_err(|_| Error::Conversion)
                })
            })?;

    render_loop(context, Arc::new(render_context))?;

    Ok(())
}

use std::{cell::RefCell, rc::Rc, sync::Arc};

fn render_loop(
    context: Arc<Context>,
    render_context: Arc<web_sys::CanvasRenderingContext2d>,
) -> Result<(), Error> {
    let handle = Rc::new(RefCell::new(None));
    let haandle = handle.clone();

    let mut stop = false;

    *haandle.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if stop {
            let _ = handle.borrow_mut().take();

            return;
        }

        draw_grid(context.clone(), render_context.clone());
        //draw_cells(context.clone(), render_context.clone());

        if let Err(error) = render_loop(context.clone(), render_context.clone()) {
            log::error!("{:?}", error);
            stop = true;
        }
    }) as Box<dyn FnMut()>));

    next_frame(haandle.borrow().as_ref().unwrap())?;

    Ok(())
}

fn next_frame(handle: &Closure<dyn FnMut()>) -> Result<(), Error> {
    let window = web_sys::window().map_or(Err(Error::NoWindow), Ok)?;
    window
        .request_animation_frame(handle.as_ref().unchecked_ref())
        .map(|_| ())
        .map_err(Error::External)
}

fn draw_grid(context: Arc<Context>, render_context: Arc<web_sys::CanvasRenderingContext2d>) {
    let cell_size = 16;
    let grid_color = "#CCCCCC";

    let width = context.width();
    let height = context.height();

    render_context.begin_path();
    render_context.set_stroke_style(&JsValue::from_str(grid_color));

    for index in 0..width {
        render_context.move_to((index * (cell_size + 1) + 1) as f64, 0f64);
        render_context.line_to(
            (index * (cell_size + 1) + 1) as f64,
            ((cell_size + 1) * height + 1) as f64,
        );
    }

    for index in 0..height {
        render_context.move_to(0f64, (index * (cell_size + 1) + 1) as f64);
        render_context.line_to(
            ((cell_size + 1) * width + 1) as f64,
            (index * (cell_size + 1) + 1) as f64,
        );
    }

    render_context.stroke();
}

fn draw_cells(context: Arc<Context>, render_context: Arc<web_sys::CanvasRenderingContext2d>) {
    let cell_size = 16;
    let dead_color = JsValue::from("#FFFFFF");
    let alive_color = JsValue::from("#000000");

    render_context.begin_path();

    for row in 0..context.height() {
        for column in 0..context.width() {
            render_context.set_fill_style(if context.is_alive(row, column) {
                &alive_color
            } else {
                &dead_color
            });

            render_context.fill_rect(
                (column * (cell_size + 1) + 1) as f64,
                (row * (cell_size + 1) + 1) as f64,
                cell_size as f64,
                cell_size as f64,
            );
        }
    }

    render_context.stroke();
}
