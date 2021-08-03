author::error!(Parse(String));

use std::borrow::Cow;

use tendril::{fmt::UTF8, NonAtomic, StrTendril, Tendril};

pub struct Document {
    key: Option<StrTendril>,
    errors: Vec<Error>,
}

use xml5ever::{
    expanded_name,
    namespace_url,
    ns,
    local_name,
    interface::{ExpandedName, QualName, tree_builder::{QuirksMode, ElementFlags}, Attribute},
    tree_builder::{NodeOrText, TreeSink},
};

impl TreeSink for Document {
    type Handle = Option<StrTendril>;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&mut self, message: Cow<'static, str>) {
        self.errors.push(Error::Parse(message.into()));
    }

    fn get_document(&mut self) -> Self::Handle {
        None
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {
        match target {
            Some(target) => expanded_name!("", target),
            None => expanded_name!("", ""),
        }
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags
    ) -> Self::Handle {
        self.
    }

    fn create_comment(&mut self, text: Tendril<UTF8, NonAtomic>) -> Self::Handle {
        None
    }

    fn create_pi(
        &mut self,
        target: Tendril<UTF8, NonAtomic>,
        data: Tendril<UTF8, NonAtomic>,
    ) -> Self::Handle {
        None
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        println!("append");

        match child {
            NodeOrText::AppendNode(node) => {
                println!("node {:?}", node);
            }
            NodeOrText::AppendText(text) => {
                println!("text {:?}", text);
            }
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        println!("append_based_on_parent_node");
        match child {
            NodeOrText::AppendNode(node) => {
                println!("node {:?}", node);
            }
            NodeOrText::AppendText(text) => {
                println!("text {:?}", text);
            }
        }
    }

    fn append_doctype_to_document(
        &mut self,
        name: Tendril<UTF8, NonAtomic>,
        public_id: Tendril<UTF8, NonAtomic>,
        system_id: Tendril<UTF8, NonAtomic>,
    ) {
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        None
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, _mode: QuirksMode) {}

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        println!("append_before_sibling");

        match new_node {
            NodeOrText::AppendNode(node) => {
                println!("node {:?}", node);
            }
            NodeOrText::AppendText(text) => {
                println!("text {:?}", text);
            }
        }
    }

    fn add_attrs_if_missing(&mut self, _target: &Self::Handle, _attrs: Vec<Attribute>) {}

    fn remove_from_parent(&mut self, _target: &Self::Handle) {}

    fn reparent_children(&mut self, _node: &Self::Handle, _new_parent: &Self::Handle) {}
}
