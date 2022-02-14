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
    pub fn new(&self, buffer: &'a mut W) -> Self {
        Self { buffer }
    }
}

use ::{serde::ser, std::fmt};

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::Serialize(msg.to_string())
    }
}

pub struct SerializeSeq<'a, 'b, W> {
    serializer: &'b mut Serializer<'a, W>,
    len: Option<usize>,
}

use serde::Serialize;

impl<'a, 'b, B> ser::SerializeSeq for SerializeSeq<'a, 'b, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeMap<'a, B> {
    buffer: &'a mut B,
    len: Option<usize>,
}

impl<'a, B> ser::SerializeMap for SerializeMap<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTuple<'a, B> {
    buffer: &'a mut B,
    len: usize,
}

impl<'a, B> ser::SerializeTuple for SerializeTuple<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeStruct<'a, B> {
    buffer: &'a mut B,
    name: &'static str,
    len: usize,
}

impl<'a, B> ser::SerializeStruct for SerializeStruct<'a, B> {
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
        key.serialize(&mut Serializer::new(&mut self.buffer))?;
        value.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeStructVariant<'a, B> {
    buffer: &'a mut B,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a, B> ser::SerializeStructVariant for SerializeStructVariant<'a, B> {
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
        key.serialize(&mut Serializer::new(&mut self.buffer))?;
        value.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTupleStruct<'a, B> {
    buffer: &'a mut B,
    len: usize,
}

impl<'a, B> ser::SerializeTupleStruct for SerializeTupleStruct<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTupleVariant<'a, B> {
    buffer: &'a mut B,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a, B> ser::SerializeTupleVariant for SerializeTupleVariant<'a, B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut Serializer::new(&mut self.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.buffer.flush().map_err(Error::from)
    }
}
