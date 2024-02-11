use std::ops::Deref as _;

use axum::extract::FromRef;
use bitvec::prelude::*;
use time::format_description::well_known::Iso8601;

use crate::contact_data;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    start: time::PrimitiveDateTime,
    end: time::PrimitiveDateTime,
    minutes: BitVec<u8, Lsb0>,
}

impl Activity {
    /// Gets an activity report from a sorted (oldest first) list of all contacts
    pub fn from_contacts(contacts: Vec<contact_data::ContactData>) -> Self {
        if contacts.len() == 0 {
            return Self {
                start: time::PrimitiveDateTime::MIN,
                end: time::PrimitiveDateTime::MIN,
                minutes: bitvec![u8, Lsb0; 0],
            };
        }

        let mut start = contacts.first().unwrap().timestamp.clone();
        let start = start
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap();

        let end = contacts.last().unwrap().timestamp.clone();
        let end = end
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap();

        let minutes: BitVec<u8, Lsb0> = contacts
            .iter()
            .map(|c| c.timestamp)
            .map_windows(|[t1, t2]| get_active_time(t1.clone(), t2.clone()))
            .flatten()
            .chain(core::iter::once(true))
            .collect();

        let activity = Self {
            start,
            end,
            minutes,
        };
        activity.check_length();
        activity
    }

    fn from_map(
        map: &async_graphql::indexmap::IndexMap<async_graphql::Name, async_graphql::Value>,
    ) -> Option<Self> {
        if let async_graphql::Value::Number(len) = map.get("len")? {
            if let Some(len) = len.as_u64().and_then(|n| usize::try_from(n).ok()) {
                if let async_graphql::Value::Binary(data) = map.get("val")? {
                    if let async_graphql::Value::String(start_time) = map.get("start")? {
                        if let async_graphql::Value::String(end_time) = map.get("end")? {
                            return Some(Self {
                                start: time::PrimitiveDateTime::parse(
                                    start_time,
                                    &Iso8601::DEFAULT,
                                )
                                .ok()?,
                                end: time::PrimitiveDateTime::parse(end_time, &Iso8601::DEFAULT)
                                    .ok()?,
                                minutes: data.iter().map(|b| *b != 0).collect(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    pub fn adjust_start(&self, start: time::PrimitiveDateTime) -> Option<Self> {
        let v = self._adjust_start(start);
        if let Some(ref a) = v {
            a.check_length();
        }
        v
    }

    fn _adjust_start(&self, start: time::PrimitiveDateTime) -> Option<Self> {
        let start = start
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap();

        match start.cmp(&self.start) {
            std::cmp::Ordering::Less => {
                let mins: usize = (self.start - start).whole_minutes().try_into().ok()?;

                let mut minutes = BitVec::with_capacity(mins + self.minutes.len());
                minutes.resize(mins, false);
                minutes.extend_from_bitslice(&self.minutes);

                Some(Self {
                    start,
                    end: self.end.clone(),
                    minutes,
                })
            }
            std::cmp::Ordering::Equal => Some(self.clone()),
            std::cmp::Ordering::Greater => {
                if start <= self.end {
                    let mins: usize = (start - self.start).whole_minutes().try_into().ok()?;
                    Some(Self {
                        start,
                        end: self.end.clone(),
                        minutes: self.minutes[mins..].to_bitvec(),
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn adjust_end(&self, end: time::PrimitiveDateTime) -> Option<Self> {
        let v = self._adjust_end(end);
        if let Some(ref a) = v {
            a.check_length();
        }
        v
    }

    fn _adjust_end(&self, end: time::PrimitiveDateTime) -> Option<Self> {
        let end = end
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap();

        match end.cmp(&self.end) {
            std::cmp::Ordering::Less => {
                if self.start <= end {
                    let mins: usize = (self.end - end).whole_minutes().try_into().ok()?;
                    Some(Self {
                        start: self.start.clone(),
                        end,
                        minutes: self.minutes[..self.minutes.len() - mins].to_bitvec(),
                    })
                } else {
                    None
                }
            }
            std::cmp::Ordering::Equal => Some(self.clone()),
            std::cmp::Ordering::Greater => {
                let mins: usize = (end - self.end).whole_minutes().try_into().ok()?;

                let mut minutes = BitVec::with_capacity(self.minutes.len() + mins);
                minutes.extend_from_bitslice(&self.minutes);
                minutes.resize(self.minutes.len() + mins, false);

                Some(Self {
                    start: self.start.clone(),
                    end,
                    minutes,
                })
            }
        }
    }

    fn check_length(&self) {
        assert_eq!(
            Some(self.minutes.len() - 1),
            (self.end - self.start).whole_minutes().try_into().ok(),
        );
    }
}

fn get_active_time(
    t1: time::PrimitiveDateTime,
    t2: time::PrimitiveDateTime,
) -> Box<dyn Iterator<Item = bool>> {
    assert!(t1 <= t2);

    let t1 = t1.replace_second(0).unwrap().replace_nanosecond(0).unwrap();
    let t2 = t2.replace_second(0).unwrap().replace_nanosecond(0).unwrap();

    let d = t2 - t1;
    let mins: usize = d.whole_minutes().try_into().unwrap();

    if mins == 0 {
        Box::new(core::iter::empty())
    } else {
        Box::new(
            core::iter::once(true)
                .chain(core::iter::repeat(d <= time::Duration::minutes(10)).take(mins - 1)),
        )
    }
}

#[async_graphql::Scalar]
impl async_graphql::ScalarType for Activity {
    fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
        if let async_graphql::Value::Object(ref map) = value {
            if let Some(o) = Self::from_map(map) {
                Ok(o)
            } else {
                Err(async_graphql::InputValueError::expected_type(value))
            }
        } else {
            Err(async_graphql::InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> async_graphql::Value {
        let mut map = async_graphql::indexmap::IndexMap::with_capacity(4);

        map.insert(
            async_graphql::Name::new("start"),
            async_graphql::Value::String(
                self.start.assume_utc().format(&Iso8601::DEFAULT).unwrap(),
            ),
        );

        map.insert(
            async_graphql::Name::new("end"),
            async_graphql::Value::String(self.end.assume_utc().format(&Iso8601::DEFAULT).unwrap()),
        );

        map.insert(
            async_graphql::Name::new("len"),
            async_graphql::Value::Number(self.minutes.len().into()),
        );
        map.insert(
            async_graphql::Name::new("val"),
            async_graphql::Value::Binary(
                self.minutes
                    .iter()
                    .by_vals()
                    .map(|b| u8::from(b))
                    .collect::<Vec<_>>()
                    .into(),
            ),
        );

        async_graphql::Value::Object(map)
    }
}

fn value_to_u64(v: &async_graphql::Value) -> Option<u64> {
    if let async_graphql::Value::Number(n) = v {
        n.as_u64()
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use async_graphql::ScalarType;
    use bitvec::prelude::*;

    use crate::{
        adif::read_adif,
        contact_data::{self, ContactData},
    };

    use super::Activity;

    const ADI: &'static str = include_str!("test.adi");

    #[test]
    fn conversion() {
        let contacts: Vec<_> = read_adif(ADI)
            .unwrap()
            .map(|r| contact_data::ContactData::from(r.unwrap()))
            .collect();

        let activity = Activity::from_contacts(contacts);
        assert_eq!(
            activity.minutes,
            core::iter::repeat(true)
                .take(11)
                .chain(core::iter::repeat(false).take(10))
                .chain(core::iter::once(true))
                .collect::<BitVec<u8, Lsb0>>()
        );

        let value = activity.to_value();
        assert_eq!(Activity::parse(value).unwrap(), activity);
    }
}
