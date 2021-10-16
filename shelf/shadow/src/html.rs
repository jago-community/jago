#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Serialize")]
    Serialize,
    #[error("Deserialize")]
    Deserialize,
}

use std::{
    borrow::{Borrow, Cow},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    ops::Deref,
};

use html5ever::{
    interface::{ExpandedName, QualName},
    tendril::{fmt::UTF8, NonAtomic, StrTendril, Tendril},
    tree_builder::{Attribute, ElementFlags, NodeOrText, QuirksMode, TreeSink},
    LocalName,
};

use crdts::{
    merkle_reg::{Hash, MerkleReg, Sha3Hash},
    CmRDT, Dot, GSet, LWWReg,
};

#[test]
fn test_sink() {
    use html5ever::{parse_document, tendril::stream::TendrilSink};

    let input = "<!DOCTYPE html>
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
        </body>
    </html>";

    let mut sink = Document::default();

    let parser = parse_document(sink, Default::default());

    let got = parser.from_utf8().one(input.as_bytes());
}

#[derive(Debug)]
struct Document {
    spot: Dot<u8>,
    handle: LWWReg<Option<Hash>, u64>,
    register: MerkleReg<Node>,
}

impl Default for Document {
    fn default() -> Self {
        let spot = Dot::new(0, 0);

        Self {
            spot,
            handle: LWWReg {
                val: None,
                marker: spot.counter,
            },
            register: MerkleReg::default(),
        }
    }
}

impl Document {
    fn set_handle(&mut self, hash: Hash) {
        self.spot.apply_inc();
        self.handle.update(Some(hash), self.spot.counter);
    }

    fn put_node(&mut self, value: Node, children: BTreeSet<Hash>) -> Hash {
        let node = self.register.write(value, children);
        let hash = node.hash();
        self.register.apply(node);
        hash
    }

    fn get_node(&self, hash: Hash) -> Option<&Node> {
        self.register.node(hash).map(|node| &node.value)
    }
}

use tiny_keccak::Sha3;

#[derive(Debug, Hash)]
pub enum Node {
    Document,
    Fragment,
    Doctype {
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    },
    Comment {
        content: StrTendril,
    },
    Text {
        content: StrTendril,
    },
    Element {
        name: QualName,
        id: Option<LocalName>,
        classes: BTreeSet<LocalName>,
        attrs: BTreeMap<QualName, StrTendril>,
    },
    ProcessingInstruction {
        target: StrTendril,
        data: StrTendril,
    },
}

impl Sha3Hash for Node {
    fn hash(&self, hasher: &mut Sha3) {
        use Node::*;

        match self {
            Document => {
                [0].hash(hasher);
            }
            Fragment => {
                [1].hash(hasher);
            }
            Doctype {
                name,
                public_id,
                system_id,
            } => {
                name.as_bytes().hash(hasher);
                public_id.as_bytes().hash(hasher);
                system_id.as_bytes().hash(hasher);
            }
            Comment { content } => {
                content.as_bytes().hash(hasher);
            }
            Text { content } => {
                content.as_bytes().hash(hasher);
            }
            Element {
                name,
                id,
                classes,
                attrs,
            } => {
                if let Some(prefix) = &name.prefix {
                    prefix.as_ref().hash(hasher);
                }

                name.ns.as_ref().hash(hasher);
                name.local.as_ref().hash(hasher);

                if let Some(id) = id {
                    id.as_ref().hash(hasher);
                }

                for class in classes {
                    class.as_ref().hash(hasher);
                }

                for (key, value) in attrs {
                    if let Some(prefix) = &key.prefix {
                        prefix.as_ref().hash(hasher);
                    }

                    key.ns.as_ref().hash(hasher);
                    key.local.as_ref().hash(hasher);

                    value.as_bytes().hash(hasher);
                }
            }
            ProcessingInstruction { target, data } => {
                target.as_bytes().hash(hasher);
                data.as_bytes().hash(hasher);
            }
        }
    }
}

impl TreeSink for Document {
    type Handle = Hash;
    type Output = Self;

    fn finish(self) -> Self::Output {
        unimplemented!()
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        unimplemented!()
    }

    fn get_document(&mut self) -> Self::Handle {
        match self.handle.val {
            Some(hash) => hash,
            None => {
                let hash = self.put_node(Node::Document, Default::default());
                self.set_handle(hash);
                hash
            }
        }
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {
        unimplemented!()
        //use html5ever::{expanded_name, local_name, namespace_url, ns};

        //let target = match target {
        //Ok(target) => target,
        //Err(_error) => return expanded_name!("", "div"),
        //};

        //match self.get_node(*target) {
        //Ok(Node::Element { name, .. }) => {
        //let b = name.0;
        //b.expanded()
        //}
        //_ => {
        //unimplemented!()
        //}
        //}
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        let id = attrs
            .iter()
            .find(|a| a.name.local.deref() == "id")
            .map(|a| LocalName::from(a.value.deref()));

        let classes: BTreeSet<LocalName> = attrs
            .iter()
            .find(|a| a.name.local.deref() == "class")
            .map_or(BTreeSet::new(), |a| {
                a.value
                    .deref()
                    .split_whitespace()
                    .map(LocalName::from)
                    .collect()
            });

        self.put_node(
            Node::Element {
                name,
                id,
                classes,
                attrs: attrs
                    .into_iter()
                    .map(|attribute| (attribute.name, attribute.value))
                    .collect(),
            },
            Default::default(),
        )
    }

    fn create_comment(&mut self, text: Tendril<UTF8, NonAtomic>) -> Self::Handle {
        unimplemented!()
    }

    fn create_pi(
        &mut self,
        target: Tendril<UTF8, NonAtomic>,
        data: Tendril<UTF8, NonAtomic>,
    ) -> Self::Handle {
        unimplemented!()
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        if let Some(parent) = self.register.node(*parent) {
            match child {
                NodeOrText::AppendNode(child) => {
                    let mut children = parent.children.clone();
                    children.insert(child);
                }
                NodeOrText::AppendText(text) => {
                    dbg!(text);
                    unimplemented!()
                }
            };
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        unimplemented!()
    }

    fn append_doctype_to_document(
        &mut self,
        _name: Tendril<UTF8, NonAtomic>,
        _public_id: Tendril<UTF8, NonAtomic>,
        _system_id: Tendril<UTF8, NonAtomic>,
    ) {
        log::trace!("append_doctype_to_document");
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        unimplemented!()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        unimplemented!()
    }

    fn set_quirks_mode(&mut self, _mode: QuirksMode) {
        log::trace!("set_quirks_mode");
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        unimplemented!()
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {
        unimplemented!()
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        unimplemented!()
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        unimplemented!()
    }

    //fn mark_script_already_started(&mut self, _node: &Self::Handle) { ... }
    //fn pop(&mut self, _node: &Self::Handle) { ... }
    //fn associate_with_form(
    //&mut self,
    //_target: &Self::Handle,
    //_form: &Self::Handle,
    //_nodes: (&Self::Handle, Option<&Self::Handle>)
    //) { ... }
    //fn is_mathml_annotation_xml_integration_point(
    //&self,
    //_handle: &Self::Handle
    //) -> bool { ... }
    //fn set_current_line(&mut self, _line_number: u64) { ... }
    //fn complete_script(&mut self, _node: &Self::Handle) -> NextParserState { ... }
}
