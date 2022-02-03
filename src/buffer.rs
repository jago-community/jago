use crdts::{CmRDT, List};

pub struct Buffer<'a, Item> {
    inner: List<Item, u8>,
    life: std::marker::PhantomData<&'a ()>,
}

use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;

impl<'a, A: AsRef<str>> From<A> for Buffer<'a, Cow<'a, str>> {
    fn from(s: A) -> Self {
        let mut inner = List::new();
        for item in s.as_ref().graphemes(true) {
            let item = Cow::Owned(item.into());
            inner.apply(inner.append(item, 0));
        }
        Buffer {
            inner,
            life: Default::default(),
        }
    }
}

use crate::terminal::{AsCommand, Directive};

impl<'a, Item> Buffer<'a, Item> {
    pub fn directives(&'a self) -> impl Iterator<Item = Directive<'a, Item>>
    where
        Item: Clone + AsCommand<'a>,
    {
        self.inner.iter().map(|it| Directive::from(it))
    }
}
