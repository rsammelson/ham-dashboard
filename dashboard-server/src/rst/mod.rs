#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub struct RST {
    readability: RSTVal,
    strength: RSTVal,
    tone: Option<RSTVal>,
}

impl TryFrom<&str> for RST {
    type Error = RSTError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let l = value.len();
        if l > 3 {
            Err(Self::Error::InvalidLength(l))
        } else {
            let mut chars = value.chars();

            let c = chars.next().ok_or(Self::Error::InvalidLength(0))?;
            let readability_int =
                c.to_digit(10)
                    .ok_or_else(|| Self::Error::InvalidCharacter(c))? as u8;
            if readability_int > 5 {
                return Err(Self::Error::ValueTooLarge(readability_int, 5));
            }
            let readability: RSTVal = readability_int.try_into()?;

            let c = chars.next().ok_or(Self::Error::InvalidLength(1))?;
            let strength: RSTVal =
                (c.to_digit(10)
                    .ok_or_else(|| Self::Error::InvalidCharacter(c))? as u8)
                    .try_into()?;

            let tone = chars
                .next()
                .map(|c| {
                    c.to_digit(10)
                        .ok_or_else(|| Self::Error::InvalidCharacter(c))
                        .and_then(|v| RSTVal::try_from(v as u8))
                })
                .transpose()?;

            Ok(Self {
                readability,
                strength,
                tone,
            })
        }
    }
}

impl RST {
    pub fn new(readability: u8, strength: u8, tone: Option<u8>) -> Result<Self, RSTError> {
        if readability > 5 {
            Err(RSTError::ValueTooLarge(readability, 5))
        } else {
            Ok(Self {
                readability: readability.try_into()?,
                strength: strength.try_into()?,
                tone: match tone {
                    Some(t) => Some(t.try_into()?),
                    None => None,
                },
            })
        }
    }

    /// Readability from 1 to 5
    pub fn readability(&self) -> u8 {
        self.readability.into()
    }

    /// Readability from 1 to 9
    pub fn strength(&self) -> u8 {
        self.strength.into()
    }

    /// Tone from 1 to 9
    pub fn tone(&self) -> Option<u8> {
        match self.tone {
            Some(n) => Some(n.into()),
            None => None,
        }
    }
}

impl core::fmt::Display for RST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tone() {
            Some(tone) => f.write_fmt(format_args!(
                "{}{}{}",
                self.readability(),
                self.strength(),
                tone
            )),
            None => f.write_fmt(format_args!("{}{}", self.readability(), self.strength())),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RST {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RSTVisitor;
        impl<'de> serde::de::Visitor<'de> for RSTVisitor {
            type Value = RST;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing an RST signal report")
            }

            fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match val.try_into() {
                    Ok(v) => Ok(v),
                    Err(e) => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(val),
                        &e,
                    )),
                }
            }
        }

        deserializer.deserialize_str(RSTVisitor)
    }
}

#[async_graphql::Scalar]
impl async_graphql::ScalarType for RST {
    fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
        match value {
            async_graphql::Value::Number(n) => Ok(RST::try_from(format!("{}", n).as_str())?),
            async_graphql::Value::String(s) => Ok(RST::try_from(s.as_str())?),
            _ => Err(async_graphql::InputValueError::custom("invalid type")),
        }
    }
    fn to_value(&self) -> async_graphql::Value {
        async_graphql::Value::String(format!("{}", self))
    }
}

impl<DB: diesel::backend::Backend> diesel::deserialize::FromSql<diesel::sql_types::Text, DB> for RST
where
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s: String =
            diesel::deserialize::FromSql::<diesel::sql_types::Text, DB>::from_sql(bytes)?;
        Ok(RST::try_from(s.as_str())?)
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::sqlite::Sqlite> for RST {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        out.set_value(format!("{}", self));
        Ok(diesel::serialize::IsNull::No)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(1)]
#[rustc_layout_scalar_valid_range_end(9)]
#[rustc_nonnull_optimization_guaranteed]
struct RSTVal(u8);

impl TryFrom<u8> for RSTVal {
    type Error = RSTError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 1 {
            Err(Self::Error::ValueTooSmall(value))
        } else if value > 9 {
            Err(Self::Error::ValueTooLarge(value, 9))
        } else {
            // Soundness: it was just checked that 1 <= value <= 9
            Ok(unsafe { Self(value) })
        }
    }
}

impl From<RSTVal> for u8 {
    fn from(value: RSTVal) -> Self {
        value.0
    }
}

impl core::fmt::Debug for RSTVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum RSTError {
    #[error("string length `{0}` was not not between 2 and 3")]
    InvalidLength(usize),
    #[error("character '`{0}`' invalid")]
    InvalidCharacter(char),
    #[error("value `{0}` was less than 1")]
    ValueTooSmall(u8),
    #[error("value `{0}` was greater than `{1}`")]
    ValueTooLarge(u8, u8),
}

impl serde::de::Expected for RSTError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        core::fmt::Display::fmt(self, formatter)
    }
}
