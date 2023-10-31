use serde::Deserialize;

pub fn empty_str_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<&str>::deserialize(de)?;
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(serde::de::value::BorrowedStrDeserializer::new(s)).map(Some),
    }
}
