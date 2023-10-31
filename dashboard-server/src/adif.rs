use crate::rst;

pub fn read_adif(
    adif: &str,
) -> anyhow::Result<adif::reader::DeserializeRecordsIter<N1MMAdifRecord>> {
    let adif = adif::reader::Reader::from_str(adif)?;
    println!("header: {:?}", adif.header());

    Ok(adif.deserialize()?)
}

#[derive(serde::Deserialize, Debug)]
pub struct N1MMAdifRecord<'s> {
    #[serde(rename = "call")]
    pub recv_callsign: &'s str,

    #[serde(with = "adif_date")]
    pub qso_date: time::Date,

    #[serde(with = "adif_time")]
    pub time_on: time::Time,

    #[serde(with = "adif_time")]
    pub time_off: time::Time,

    #[serde(rename = "freq")]
    pub freq_tx: f64,
    pub freq_rx: f64,

    pub section: Option<&'s str>,
    pub band: &'s str,
    #[serde(rename = "station_callsign")]
    pub sent_callsign: &'s str,
    #[serde(rename = "contest_id")]
    pub contest_name: Option<&'s str>,
    pub mode: &'s str,

    #[serde(rename = "rst_sent")]
    pub sent_signal_report: rst::RST,
    #[serde(rename = "rst_rcvd")]
    pub recv_signal_report: rst::RST,

    #[serde(rename = "cqz")]
    /// CQ Zone of contacted station
    pub cq_zone: u8,

    #[serde(rename = "pfx")]
    /// WPX prefix of contacted station
    pub prefix_wpx: Option<&'s str>,

    #[serde(rename = "stx")]
    /// Transmitted serial number
    serial_number_tx: u32,

    pub operator: Option<&'s str>,

    #[serde(rename = "app_n1mm_isrunqso")]
    pub n1mm_is_run_qso: bool,
    #[serde(rename = "app_n1mm_claimedqso")]
    pub n1mm_is_claimed_qso: bool,
    #[serde(rename = "app_n1mm_points")]
    pub n1mm_points: i32,

    #[serde(rename = "app_n1mm_mult1")]
    pub n1mm_is_mult_1: bool,
    #[serde(rename = "app_n1mm_mult2")]
    pub n1mm_is_mult_2: bool,
    #[serde(rename = "app_n1mm_mult3")]
    pub n1mm_is_mult_3: bool,

    #[serde(rename = "app_n1mm_exchange1")]
    pub n1mm_exchange1: Option<&'s str>,
    #[serde(rename = "app_n1mm_misctext")]
    pub n1mm_miscellaneous_text: Option<&'s str>,
    #[serde(rename = "app_n1mm_continent")]
    pub n1mm_continent: &'s str,

    #[serde(rename = "app_n1mm_radio_nr")]
    pub n1mm_radio_number: i32,
    #[serde(rename = "app_n1mm_netbiosname")]
    pub n1mm_netbios_name: &'s str,

    #[serde(rename = "app_n1mm_id")]
    pub n1mm_id: &'s str,
}

time::serde::format_description!(adif_date, Date, "[year][month][day][end]");
time::serde::format_description!(adif_time, Time, "[hour][minute][second][end]");
