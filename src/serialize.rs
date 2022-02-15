#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Serialize {0}")]
    Serialize(String),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

pub struct Serializer<'a, W> {
    buffer: &'a mut W,
}

impl<'a, W> Serializer<'a, W> {
    pub fn new(buffer: &'a mut W) -> Self {
        Self { buffer }
    }
}

use ::{
    crossterm::{style::Print, Command, QueueableCommand},
    std::fmt::Display,
};

impl<'a, W> Serializer<'a, W>
where
    W: Write,
{
    pub fn consume(&mut self, directive: impl Command) -> Result<&mut W, Error> {
        self.buffer.queue(directive).map_err(Error::from)
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.buffer.flush().map_err(Error::from)
    }

    fn display(&mut self, item: impl Display) -> Result<&mut W, Error> {
        self.consume(Print(item))
    }
}

use ::{serde::ser, std::fmt};

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::Serialize(msg.to_string())
    }
}

impl<'a, 'b, B> ser::Serializer for &'b mut Serializer<'a, B>
where
    B: Write + 'a,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SerializeSeq<'a, 'b, B>;
    type SerializeMap = SerializeMap<'a, 'b, B>;
    type SerializeTuple = SerializeTuple<'a, 'b, B>;
    type SerializeStruct = SerializeStruct<'a, 'b, B>;
    type SerializeStructVariant = SerializeStructVariant<'a, 'b, B>;
    type SerializeTupleStruct = SerializeTupleStruct<'a, 'b, B>;
    type SerializeTupleVariant = SerializeTupleVariant<'a, 'b, B>;

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
        self.display(v)?;

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

pub struct SerializeSeq<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    len: Option<usize>,
}

use ::{serde::Serialize, std::io::Write};

impl<'a, 'b, B> ser::SerializeSeq for SerializeSeq<'a, 'b, B>
where
    B: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.serializer.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeMap<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    len: Option<usize>,
}

impl<'a, 'b, W> ser::SerializeMap for SerializeMap<'a, 'b, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTuple<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    len: usize,
}

impl<'a, 'b, W> ser::SerializeTuple for SerializeTuple<'a, 'b, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeStruct<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    name: &'static str,
    len: usize,
}

impl<'a, 'b, W> ser::SerializeStruct for SerializeStruct<'a, 'b, W>
where
    W: Write,
{
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
        key.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;
        value.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeStructVariant<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a, 'b, W> ser::SerializeStructVariant for SerializeStructVariant<'a, 'b, W>
where
    W: Write,
{
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
        key.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;
        value.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTupleStruct<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    name: &'static str,
    len: usize,
}

impl<'a, 'b, W> ser::SerializeTupleStruct for SerializeTupleStruct<'a, 'b, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTupleVariant<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a, 'b, W> ser::SerializeTupleVariant for SerializeTupleVariant<'a, 'b, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.serializer.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.buffer.flush().map_err(Error::from)
    }
}
