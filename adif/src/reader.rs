#[derive(Debug)]
pub struct Reader<'de> {
    header: Option<&'de str>,
    records: &'de str,
}

impl<'de> Reader<'de> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'de str) -> Result<Self, Error> {
        if input.starts_with('<') {
            return Ok(Reader {
                header: None,
                records: input,
            });
        }

        while let Some(i) = input.find('<') {
            if Some("eoh>")
                == input
                    .get((i + 1)..(i + 5))
                    .map(|t| t.to_ascii_lowercase())
                    .as_deref()
            {
                return Ok(Reader {
                    header: Some(&input[..i]),
                    records: &input[i + 6..],
                });
            }
        }
        Err(Error::NoData)
    }

    pub fn header(&self) -> Option<&'de str> {
        self.header
    }

    pub fn deserialize<D: serde::Deserialize<'de>>(
        &self,
    ) -> Result<DeserializeRecordsIter<'de, D>, Error> {
        Ok(DeserializeRecordsIter::new(self.records))
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("no data found")]
    NoData,

    #[error("cannot infer type")]
    CannotInferType,

    #[error("invalid type, `{0}` requested")]
    InvalidType(&'static str),

    #[error("unexpected end of input while `{0}`")]
    UnexpectedEndOfInput(&'static str),

    #[error("issue parsing integer: `0`")]
    ParseIntError(core::num::ParseIntError),

    #[error("issue parsing float: `0`")]
    ParseFloatError(core::num::ParseFloatError),

    #[error("`{0}`")]
    OtherError(&'static str),

    #[error("`{0}`")]
    Custom(String),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        Self::Custom(format!("{}", msg))
    }
}

impl From<core::num::ParseIntError> for Error {
    fn from(value: core::num::ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<core::num::ParseFloatError> for Error {
    fn from(value: core::num::ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}

#[derive(Debug)]
pub struct DeserializeRecordsIter<'de, D> {
    records: &'de str,
    _priv: std::marker::PhantomData<D>,
}

impl<'de, D> DeserializeRecordsIter<'de, D> {
    fn new(records: &'de str) -> Self {
        Self {
            records,
            _priv: std::marker::PhantomData,
        }
    }
}

impl<'de, D> Iterator for DeserializeRecordsIter<'de, D>
where
    D: serde::Deserialize<'de>,
{
    type Item = Result<D, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut deserializer = RecordDeserializer {
            records: &mut self.records,
            read_string: None,
            characters_to_read: None,
            next_type: None,
        };

        let item = D::deserialize(&mut deserializer);

        match item {
            Err(Error::NoData) => None,
            other => Some(other),
        }
    }
}

struct RecordDeserializer<'de, 's> {
    records: &'s mut &'de str,
    read_string: Option<String>,
    characters_to_read: Option<usize>,
    next_type: Option<char>,
}

impl<'de> RecordDeserializer<'de, '_> {
    fn get_str(&mut self) -> Result<&'de str, Error> {
        if self.read_string.is_some() {
            Err(Error::OtherError("unexpected deserializer state"))
        } else if let Some(length) = self.characters_to_read.take() {
            let val = &self.records[..length];
            *self.records = &self.records[length..];
            Ok(val)
        } else {
            Err(Error::InvalidType("str"))
        }
    }

    fn get_char(&mut self) -> Result<char, Error> {
        let s = self.get_str()?;
        if s.len() > 1 {
            Err(Error::InvalidType("char"))
        } else if let Some(c) = s.chars().next() {
            Ok(c)
        } else {
            Err(Error::InvalidType("char"))
        }
    }
}

impl<'de> serde::Deserializer<'de> for &mut RecordDeserializer<'de, '_> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Some(s) = self.read_string.take() {
            visitor.visit_string(s)
        } else if self.characters_to_read.is_some() {
            Err(Error::InvalidType("any"))
            // self.deserialize_str(visitor)
        } else {
            self.deserialize_map(visitor)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.read_string.take().is_some() {
            visitor.visit_none()
        } else if let Some(length) = self.characters_to_read.take() {
            *self.records = &self.records[length..];
            visitor.visit_none()
        } else {
            self.deserialize_map(visitor)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(match self.get_str()?.to_ascii_lowercase().as_str() {
            "0" => Ok(false),
            "1" => Ok(true),
            "n" => Ok(false),
            "y" => Ok(true),
            "no" => Ok(false),
            "yes" => Ok(true),
            "t" => Ok(false),
            "f" => Ok(true),
            "false" => Ok(false),
            "true" => Ok(true),
            _ => Err(Error::InvalidType("bool")),
        }?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(self.get_str()?.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i16(self.get_str()?.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(self.get_str()?.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i64(self.get_str()?.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.get_str()?.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(self.get_str()?.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(self.get_str()?.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u64(self.get_str()?.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f32(self.get_str()?.parse()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f64(self.get_str()?.parse()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(self.get_char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Some(s) = self.read_string.take() {
            visitor.visit_string(s)
        } else {
            visitor.visit_borrowed_str(self.get_str()?)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Some(s) = self.read_string.take() {
            visitor.visit_string(s)
        } else {
            visitor.visit_borrowed_str(self.get_str()?)
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.get_str()?.as_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.get_str()?.as_bytes())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::InvalidType("unit"))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::InvalidType("unit struct"))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::InvalidType("newtype struct"))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::InvalidType("seq"))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::InvalidType("tuple"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::InvalidType("tuple struct"))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::InvalidType("enum"))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Some(s) = self.read_string.take() {
            visitor.visit_string(s)
        } else {
            todo!()
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for RecordDeserializer<'de, '_> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.characters_to_read.is_some() {
            return Err(Error::OtherError("did not read previous value"));
        }

        *self.records = self.records.trim_start();
        if !self.records.starts_with('<') {
            if self.records.is_empty() {
                return Err(Error::NoData);
            } else {
                return Err(Error::OtherError("could not find start of data specifier"));
            }
        }
        let Some(idx) = self.records[1..].find('>') else {
            return Err(Error::UnexpectedEndOfInput(
                "while looking for the end of data specifier",
            ));
        };

        let specifier = &self.records[1..(idx + 1)]; // +1 since search was after first character

        let Some(name_end_idx) = specifier.find(':') else {
            if specifier.to_ascii_lowercase() == "eor" {
                *self.records = &self.records[(idx + 2)..];
                return Ok(None);
            } else {
                return Err(Error::OtherError("no colon in data specifier"));
            }
        };

        let name = specifier[..name_end_idx].to_lowercase();

        let mut length_str = &specifier[(name_end_idx + 1)..];
        let data_type = if let Some(length_end_idx) = length_str.find(':') {
            let mut ts = length_str[length_end_idx..].chars();
            length_str = &length_str[..length_end_idx];

            let typ = ts.next();
            if ts.next().is_some() {
                return Err(Error::OtherError("invalid type specifier"));
            }
            typ
        } else {
            None
        };

        let Ok(length): Result<usize, _> = str::parse(length_str) else {
            return Err(Error::OtherError(
                "could not parse length from data specifier",
            ));
        };

        *self.records = &self.records[(idx + 2)..];
        self.read_string = Some(name);
        self.characters_to_read = Some(length);
        self.next_type = data_type;

        Ok(Some(seed.deserialize(self)?))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
}
