use std::collections::{HashMap, HashSet};

pub struct Context<Item> {
    items: HashSet<Item>,
    item_map: HashMap<usize, Item>,
}
