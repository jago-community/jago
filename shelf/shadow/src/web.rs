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
    collections::{BTreeSet, HashMap, HashSet},
};

use html5ever::{
    interface::{ExpandedName, QualName},
    tendril::{fmt::UTF8, NonAtomic, StrTendril, Tendril},
    tree_builder::{Attribute, ElementFlags, NodeOrText, QuirksMode, TreeSink},
    LocalName,
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

use crdts::{
    merkle_reg::{Hash, MerkleReg},
    CmRDT, Dot, LWWReg,
};

type Reflection = Vec<u8>;

struct Document {
    spot: Dot<u8>,
    handle: LWWReg<Option<Hash>, u64>,
    register: MerkleReg<Reflection>,
}

impl TreeSink for Document {
    type Handle = Result<Hash, Error>;
    type Output = Self;

    fn finish(self) -> Self::Output {
        unimplemented!()
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        unimplemented!()
    }

    fn get_document(&mut self) -> Self::Handle {
        match self.handle.val {
            Some(hash) => Ok(hash),
            None => {
                let hash = self.put_node(Node::Document, Default::default())?;
                self.set_handle(hash);
                Ok(hash)
            }
        }
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {
        unimplemented!()
    }

    fn create_element(
        &mut self,
        name: QualName,
        _attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        unimplemented!()
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
        unimplemented!()
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

use serde::{
    de::{self, Error as _, MapAccess, SeqAccess, Visitor},
    ser::Error as _,
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(PartialEq, Eq, Hash)]
struct StringBox(StrTendril);

impl Serialize for StringBox {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.0.borrow();

        serializer.serialize_bytes(bytes)
    }
}

impl<'de> Deserialize<'de> for StringBox {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;

        let output = StrTendril::try_from_byte_slice(&bytes)
            .map_err(|_| D::Error::custom("invalid utf8 sequence"))?;

        Ok(StringBox(output))
    }
}

#[derive(PartialEq, Eq, Hash)]
struct NameBox(QualName);

impl Serialize for NameBox {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut qual_name = serializer.serialize_struct("QualName", 3)?;
        qual_name.serialize_field("prefix", &self.0.prefix)?;
        qual_name.serialize_field("ns", &self.0.ns)?;
        qual_name.serialize_field("local", &self.0.local)?;
        qual_name.end()
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field {
    Prefix,
    Ns,
    Local,
}

struct NameBoxVisitor;

impl<'de> Visitor<'de> for NameBoxVisitor {
    type Value = NameBox;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Duration")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let prefix = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let ns = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let local = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;

        Ok(NameBox(QualName::new(prefix, ns, local)))
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut prefix = None;
        let mut ns = None;
        let mut local = None;

        while let Some(key) = map.next_key()? {
            match key {
                Field::Prefix => {
                    if prefix.is_some() {
                        return Err(de::Error::duplicate_field("prefix"));
                    }
                    prefix = Some(map.next_value()?);
                }
                Field::Ns => {
                    if ns.is_some() {
                        return Err(de::Error::duplicate_field("ns"));
                    }
                    ns = Some(map.next_value()?);
                }
                Field::Local => {
                    if local.is_some() {
                        return Err(de::Error::duplicate_field("local"));
                    }
                    local = Some(map.next_value()?);
                }
            }
        }

        let prefix = prefix.ok_or_else(|| de::Error::missing_field("prefix"))?;
        let ns = ns.ok_or_else(|| de::Error::missing_field("ns"))?;
        let local = local.ok_or_else(|| de::Error::missing_field("local"))?;

        Ok(NameBox(QualName::new(prefix, ns, local)))
    }
}

impl<'de> Deserialize<'de> for NameBox {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("NameBox", &["prefix", "ns", "local"], NameBoxVisitor)
    }
}

#[derive(Deserialize, Serialize)]
pub enum Node {
    Document,
    Fragment,
    Doctype {
        name: StringBox,
        public_id: StringBox,
        system_id: StringBox,
    },
    Comment {
        content: StringBox,
    },
    Text {
        content: StringBox,
    },
    Element {
        name: NameBox,
        id: Option<LocalName>,
        classes: HashSet<LocalName>,
        attrs: HashMap<NameBox, StringBox>,
    },
    ProcessingInstruction {
        target: StringBox,
        data: StringBox,
    },
}

impl Node {
    fn hash(&self) -> Result<Hash, Error> {
        use tiny_keccak::{Hasher, Sha3};

        let mut sha3 = Sha3::v256();

        let bytes = bincode::serialize(self).map_err(|_| Error::Serialize)?;

        let mut hash = [0u8; 32];
        sha3.finalize(&mut hash);

        Ok(hash)
    }
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

    fn put_node(&mut self, value: Node, children: BTreeSet<Hash>) -> Result<Hash, Error> {
        let value = bincode::serialize(&value).map_err(|_| Error::Serialize)?;
        let node = self.register.write(value, children);
        let hash = node.hash();
        self.register.apply(node);
        Ok(hash)
    }
}
