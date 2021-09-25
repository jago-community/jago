use crate::tree;

book::error!(Incomplete, tree::Error);

use seed::{prelude::*, *};

pub fn handle(key: &str) -> Result<(), Error> {
    let key = format!("div.{}", key);

    for root in tree::roots(&key)? {
        App::start(root, init, update, view);
    }

    Ok(())
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model::default()
}

type Model = i32;

enum Msg {
    Increment,
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => *model += 1,
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        "This is a counter: ",
        C!["counter"],
        button![model, ev(Ev::Click, |_| Msg::Increment)],
    ]
}
