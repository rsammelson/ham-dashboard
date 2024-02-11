use diesel::{deserialize::FromSql, serialize::ToSql};
use serde::{Deserialize, Serialize};

use crate::rst;

#[derive(
    Debug,
    Clone,
    async_graphql::SimpleObject,
    diesel::Queryable,
    diesel::Selectable,
    diesel::Insertable,
)]
#[graphql(complex)]
#[diesel(table_name = crate::schema::contacts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContactData {
    #[diesel(column_name = "id")]
    #[graphql(name = "id")]
    n1mm_id: Option<String>,

    pub recv_callsign: String,
    pub sent_callsign: String,

    pub recv_signal_report: rst::RST,
    pub sent_signal_report: rst::RST,
    #[graphql(skip)]
    pub timestamp: time::PrimitiveDateTime,

    pub mode: String,
    // band: String,
    pub freq_rx: i64,
    pub freq_tx: i64,

    pub exchange1: Option<String>,
    pub section: Option<String>,
    pub prefix_wpx: Option<String>,
    pub cq_zone: i16,

    pub operator: Option<String>,
    pub contest_name: Option<String>,

    pub is_mult_1: bool,
    pub is_mult_2: bool,
    pub is_mult_3: bool,

    pub is_run_qso: bool,
    pub is_claimed_qso: bool,
    pub points: i32,

    pub location_source: LocationSource,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
}

#[async_graphql::ComplexObject]
impl ContactData {
    async fn timestamp(&self) -> String {
        self.timestamp
            .format(time::macros::format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
            ))
            .unwrap()
    }
}

impl ContactData {
    pub fn id(&self) -> Option<&str> {
        self.n1mm_id.as_deref()
    }
}

impl From<crate::xml::ContactInfo<'_>> for ContactData {
    fn from(value: crate::xml::ContactInfo) -> Self {
        Self {
            n1mm_id: Some(value.id.to_owned()),

            recv_callsign: value.recv_callsign.to_owned(),
            sent_callsign: value.sent_callsign.to_owned(),

            recv_signal_report: value.recv_signal_report,
            sent_signal_report: value.sent_signal_report,
            timestamp: value.timestamp,

            mode: value.mode.to_owned(),
            // band: value.band.to_owned(),
            freq_rx: value.freq_rx,
            freq_tx: value.freq_tx,

            exchange1: value.exchange1.map(|s| s.to_owned()),
            section: value.section.map(|s| s.to_owned()),
            prefix_wpx: value.prefix_wpx.map(|s| s.to_owned()),
            cq_zone: value.cq_zone.into(),

            contest_name: value.contest_name.map(|s| s.to_owned()),
            operator: value.operator.map(|s| s.to_owned()),

            is_mult_1: value.is_mult_1,
            is_mult_2: value.is_mult_2,
            is_mult_3: value.is_mult_3,

            is_run_qso: value.is_run_qso,
            is_claimed_qso: value.is_claimed_qso,
            points: value.points,

            location_source: LocationSource::NoLocation,
            latitude: None,
            longitude: None,
        }
    }
}

impl From<crate::adif::N1MMAdifRecord<'_>> for ContactData {
    fn from(value: crate::adif::N1MMAdifRecord) -> Self {
        assert_eq!(value.time_on, value.time_off);

        Self {
            n1mm_id: Some(value.n1mm_id.to_owned()),

            recv_callsign: value.recv_callsign.to_owned(),
            sent_callsign: value.sent_callsign.to_owned(),

            recv_signal_report: value.recv_signal_report,
            sent_signal_report: value.sent_signal_report,
            timestamp: value.qso_date.with_time(value.time_on),

            mode: value.mode.to_owned(),
            // band: value.band.to_owned(),
            freq_rx: (value.freq_rx * 1000000.0).round() as i64,
            freq_tx: (value.freq_tx * 1000000.0).round() as i64,

            exchange1: value.n1mm_exchange1.map(|s| s.to_owned()),
            section: value.section.map(|s| s.to_owned()),
            prefix_wpx: value.prefix_wpx.map(|s| s.to_owned()),
            cq_zone: value.cq_zone.into(),

            contest_name: value.contest_name.map(|s| s.to_owned()),
            operator: value.operator.map(|s| s.to_owned()),

            is_mult_1: value.n1mm_is_mult_1,
            is_mult_2: value.n1mm_is_mult_2,
            is_mult_3: value.n1mm_is_mult_3,

            is_run_qso: value.n1mm_is_run_qso,
            is_claimed_qso: value.n1mm_is_claimed_qso,
            points: value.n1mm_points,

            location_source: LocationSource::NoLocation,
            latitude: None,
            longitude: None,
        }
    }
}

#[derive(Debug, Clone, diesel::AsExpression, diesel::FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum LocationSource {
    NoLocation,
    Prefix,
    HamQTH,
}

impl<DB: diesel::backend::Backend> FromSql<diesel::sql_types::VarChar, DB> for LocationSource
where
    String: FromSql<diesel::sql_types::VarChar, DB>,
{
    fn from_sql(
        bytes: <DB as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "NoLocation" => Ok(LocationSource::NoLocation),
            "Prefix" => Ok(LocationSource::Prefix),
            "HamQTH" => Ok(LocationSource::HamQTH),
            s => todo!(),
        }
    }
}

impl<DB: diesel::backend::Backend> ToSql<diesel::sql_types::VarChar, DB> for LocationSource
where
    str: ToSql<diesel::sql_types::VarChar, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        match self {
            LocationSource::NoLocation => "NoLocation".to_sql(out),
            LocationSource::Prefix => "Prefix".to_sql(out),
            LocationSource::HamQTH => "HamQTH".to_sql(out),
        }
    }
}

async_graphql::scalar!(LocationSource);
