#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Serialize {0}")]
    Serialize(String),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

pub struct View<'a, Buffer> {
    buffer: Buffer,
    ph: std::marker::PhantomData<&'a ()>,
}

impl<'a, B> View<'a, B> {
    fn new(buffer: B) -> Self {
        Self {
            buffer,
            ph: Default::default(),
        }
    }
}

use ::{serde::ser, std::fmt};

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::Serialize(msg.to_string())
    }
}

use ::{serde::Serialize, std::io::Write};

pub struct SerializeSeq<'a, 'b, B> {
    view: &'b mut View<'a, B>,
    len: Option<usize>,
}

impl<'a, 'b, B> ser::SerializeSeq for SerializeSeq<'a, 'b, B>
where
    B: Write + 'a,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.view.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeMap<'a, B> {
    view: &'a mut View<'a, B>,
    len: Option<usize>,
}

impl<'a, B> ser::SerializeMap for SerializeMap<'a, B>
where
    B: Write + 'a,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.view.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTuple<'a, B> {
    view: &'a mut View<'a, B>,
    len: usize,
}

impl<'a, B> ser::SerializeTuple for SerializeTuple<'a, B>
where
    B: Write + 'a,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.view.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeStruct<'a, B> {
    view: &'a mut View<'a, B>,
    name: &'static str,
    len: usize,
}

impl<'a, B> ser::SerializeStruct for SerializeStruct<'a, B>
where
    B: Write + 'a,
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
        key.serialize(&mut View::new(&mut self.view.buffer))?;
        value.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.view.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeStructVariant<'a, B> {
    view: &'a mut View<'a, B>,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a, B> ser::SerializeStructVariant for SerializeStructVariant<'a, B>
where
    B: Write + 'a,
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
        key.serialize(&mut View::new(&mut self.view.buffer))?;
        value.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.view.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTupleStruct<'a, B> {
    view: &'a mut View<'a, B>,
    len: usize,
}

impl<'a, B> ser::SerializeTupleStruct for SerializeTupleStruct<'a, B>
where
    B: Write + 'a,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.view.buffer.flush().map_err(Error::from)
    }
}

pub struct SerializeTupleVariant<'a, B> {
    view: &'a mut View<'a, B>,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
}

impl<'a, B> ser::SerializeTupleVariant for SerializeTupleVariant<'a, B>
where
    B: Write + 'a,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut View::new(&mut self.view.buffer))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.view.buffer.flush().map_err(Error::from)
    }
}

impl<'a, 'b, B> ser::Serializer for &'b mut View<'a, B>
where
    B: Write + 'a,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SerializeSeq<'a, 'b, &'b mut B>;
    type SerializeMap = SerializeMap<'a, B>;
    type SerializeTuple = SerializeTuple<'a, &'a mut B>;
    type SerializeStruct = SerializeStruct<'a, &'a mut B>;
    type SerializeStructVariant = SerializeStructVariant<'a, &'a mut B>;
    type SerializeTupleStruct = SerializeTupleStruct<'a, &'a mut B>;
    type SerializeTupleVariant = SerializeTupleVariant<'a, &'a mut B>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        v.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        log::trace!("serialize_none");

        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        log::trace!("serialize_unit");

        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        name.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        name.serialize(&mut View::new(&mut self.buffer))?;
        variant_index.serialize(&mut View::new(&mut self.buffer))?;
        variant.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        name.serialize(&mut View::new(&mut self.buffer))?;
        value.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
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
        name.serialize(&mut View::new(&mut self.buffer))?;
        variant_index.serialize(&mut View::new(&mut self.buffer))?;
        variant.serialize(&mut View::new(&mut self.buffer))?;
        value.serialize(&mut View::new(&mut self.buffer))?;

        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq {
            view: &mut View::new(&mut self.buffer),
            len,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeTuple {
            view: &mut View::new(&mut self.buffer),
            len,
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeTupleStruct {
            view: &mut View::new(&mut self.buffer),
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
            view: &mut View::new(&mut self.buffer),
            name,
            variant_index,
            variant,
            len,
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            view: &mut View::new(&mut self.buffer),
            len,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct {
            view: &mut View::new(&mut self.buffer),
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
            view: &mut View::new(&mut self.buffer),
            name,
            variant_index,
            variant,
            len,
        })
    }
}
