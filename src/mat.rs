use ndarray::array;

#[test]
#[ignore]
fn test_mat() {
    let mat = Array2::zeros((10, 5));

    let mut serializer = Serializer::new(Rc::new(RefCell::new(mat)));

    use serde::Serialize;

    "Hello, stranger.\n\n:D".serialize(&mut serializer).unwrap();

    assert_eq!(
        *serializer.buffer.borrow(),
        array![
            [b'H', b'e', b'l', b'l', b'o'],
            [b',', b' ', b'S', b't', b'r'],
            [b'a', b'n', b'g', b'e', b'r'],
            [b'.', b'\n', 0, 0, 0],
            [b'\n', 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]
    );
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Serialize {0}")]
    Serialize(String),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub struct Cursor {
    yx: Rc<RefCell<(usize, usize)>>,
}

impl From<(usize, usize)> for Cursor {
    fn from(yx: (usize, usize)) -> Self {
        Self {
            yx: Rc::new(RefCell::new(yx)),
        }
    }
}

impl Cursor {
    fn yx(&self) -> Ref<'_, (usize, usize)> {
        self.yx.borrow()
    }

    fn yx_mut(&self) -> RefMut<'_, (usize, usize)> {
        self.yx.borrow_mut()
    }

    fn y(&self) -> Ref<'_, usize> {
        Ref::map(self.yx.borrow(), |yx| &yx.0)
    }

    fn x(&self) -> Ref<'_, usize> {
        Ref::map(self.yx.borrow(), |yx| &yx.1)
    }

    fn y_mut(&self) -> RefMut<'_, usize> {
        RefMut::map(self.yx.borrow_mut(), |yx| &mut yx.0)
    }

    fn x_mut(&self) -> RefMut<'_, usize> {
        RefMut::map(self.yx.borrow_mut(), |yx| &mut yx.1)
    }
}

use ndarray::Array2;

pub type Matrix = Rc<RefCell<Array2<u8>>>;

pub struct Serializer {
    buffer: Matrix,
    cursor: Cursor,
}

impl Serializer {
    pub fn new(buffer: Matrix) -> Self {
        Self {
            buffer,
            cursor: Cursor::from((0, 0)),
        }
    }
}

use ::{
    crossterm::{cursor::MoveToNextLine, style::Print, Command, QueueableCommand},
    itertools::Itertools,
    std::fmt::Display,
    unicode_segmentation::UnicodeSegmentation,
};

impl Serializer {
    fn width(&self) -> usize {
        self.buffer.borrow().ncols()
    }

    pub fn consume(&mut self, directive: impl Command) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn display(&mut self, item: impl Display) -> Result<(), Error> {
        unimplemented!()
    }
}

use ::{serde::ser, std::fmt};

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::Serialize(msg.to_string())
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SerializeSeq<'a>;
    type SerializeMap = SerializeMap<'a>;
    type SerializeTuple = SerializeTuple<'a>;
    type SerializeStruct = SerializeStruct<'a>;
    type SerializeStructVariant = SerializeStructVariant<'a>;
    type SerializeTupleStruct = SerializeTupleStruct<'a>;
    type SerializeTupleVariant = SerializeTupleVariant<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.display(v)?;

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let width = self.width();

        for grapheme in v.graphemes(true) {
            // TODO: change camel case to snake case on the fly. checkedA->checked_a

            let next_x = self.cursor.x().checked_add(grapheme.len()).unwrap_or(0);

            if grapheme == "\n" || next_x > width {
                *self.cursor.x_mut() = 0;
            }

            let yx = self.cursor.yx();

            let mut buffer = self.buffer.borrow_mut();

            for (i, c) in grapheme.char_indices() {
                // ...
                //buffer[yx.1 + i..yx.1 + i + c.len_utf8()] = c;
            }

            /*
            let chars = grapheme.chars().collect_vec();

            let mut slice = buffer.slice_mut(ndarray::s![yx.1..next_x]);

            let this = array![chars];

            ndarray::Zip::from(&mut slice)
                .and(&this)
                .for_each(|a, &b| *a = b);
            */

            /*else {
                *self.cursor.x_mut() = next_x;
            }*/
        }

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        dbg!(name);
        dbg!(variant_index);
        dbg!(variant);

        unimplemented!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq {
            serializer: self,
            len,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeTuple {
            serializer: self,
            len,
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeTupleStruct {
            serializer: self,
            name,
            len,
        })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            serializer: self,
            name,
            variant_index,
            variant,
            len,
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            serializer: self,
            len,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct {
            serializer: self,
            name,
            len,
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            serializer: self,
            name,
            variant_index,
            variant,
            len,
        })
    }
}

pub struct SerializeSeq<'a> {
    serializer: &'a mut Serializer,
    len: Option<usize>,
}

use ::{crossterm::style::SetForegroundColor, serde::Serialize, std::io::Write};

impl<'a> ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

pub struct SerializeMap<'a> {
    serializer: &'a mut Serializer,
    len: Option<usize>,
}

impl<'a> ser::SerializeMap for SerializeMap<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct SerializeTuple<'a> {
    serializer: &'a mut Serializer,
    len: usize,
}

impl<'a> ser::SerializeTuple for SerializeTuple<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        //self.serializer.buffer.flush().map_err(Error::from)
        Ok(())
    }
}

pub struct SerializeStruct<'a> {
    serializer: &'a mut Serializer,
    name: &'static str,
    len: usize,
}

impl<'a> ser::SerializeStruct for SerializeStruct<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;
        value.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        //self.serializer.buffer.flush().map_err(Error::from)
        Ok(())
    }
}

pub struct SerializeStructVariant<'a> {
    serializer: &'a mut Serializer,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a> ser::SerializeStructVariant for SerializeStructVariant<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;
        value.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        //self.serializer.buffer.flush().map_err(Error::from)
        Ok(())
    }
}

pub struct SerializeTupleStruct<'a> {
    serializer: &'a mut Serializer,
    name: &'static str,
    len: usize,
}

impl<'a> ser::SerializeTupleStruct for SerializeTupleStruct<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        //self.serializer.buffer.flush().map_err(Error::from)
        Ok(())
    }
}

pub struct SerializeTupleVariant<'a> {
    serializer: &'a mut Serializer,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a> ser::SerializeTupleVariant for SerializeTupleVariant<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(self.serializer.buffer.clone()))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        //self.serializer.buffer.flush().map_err(Error::from)
        Ok(())
    }
}
