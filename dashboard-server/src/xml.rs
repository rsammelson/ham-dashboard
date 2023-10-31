use crate::{helpers, rst};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UdpData<'s> {
    #[serde(rename = "AppInfo")]
    AppInfo {
        app: &'s str,

        #[serde(rename = "dbname")]
        db_name: &'s str,

        #[serde(rename = "contestnr")]
        contest_number: i32,

        #[serde(rename = "contestname")]
        contest_name: &'s str,

        #[serde(rename = "StationName")]
        station_name: &'s str,
    },
    ContactInfo(ContactInfo<'s>),
    ContactReplace(ContactInfo<'s>),
    ContactDelete(ContactDelete<'s>),
    LookupInfo(ContactInfo<'s>),
    #[serde(rename = "RadioInfo")]
    RadioInfo(RadioInfo<'s>),
    Spot,
    DynamicResults(ScoreInfo<'s>),
}

#[derive(Debug, Deserialize)]
pub struct ContactInfo<'s> {
    app: &'s str,

    #[serde(rename = "contestnr")]
    pub contest_number: i32,

    #[serde(rename = "contestname")]
    pub contest_name: Option<&'s str>,

    #[serde(with = "n1mm_date_format")]
    pub timestamp: time::PrimitiveDateTime,

    #[serde(rename = "mycall")]
    pub sent_callsign: &'s str,

    #[serde(rename = "call")]
    pub recv_callsign: &'s str,
    pub band: f32,
    pub mode: &'s str,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    pub operator: Option<&'s str>,

    #[serde(rename = "rxfreq", deserialize_with = "hz_from_freq")]
    pub freq_rx: i64,
    #[serde(rename = "txfreq", deserialize_with = "hz_from_freq")]
    pub freq_tx: i64,

    #[serde(rename = "countryprefix")]
    pub country_prefix: &'s str,

    #[serde(rename = "wpxprefix", deserialize_with = "helpers::empty_str_as_none")]
    pub prefix_wpx: Option<&'s str>,

    pub continent: &'s str,

    #[serde(rename = "snt")]
    pub sent_signal_report: rst::RST,
    #[serde(rename = "sntnr")]
    sent_number: u32,
    #[serde(rename = "rcv")]
    pub recv_signal_report: rst::RST,
    #[serde(rename = "rcvnr")]
    recv_number: u32,

    #[serde(rename = "gridsquare", deserialize_with = "helpers::empty_str_as_none")]
    grid_square: Option<&'s str>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    pub exchange1: Option<&'s str>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    pub section: Option<&'s str>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    comment: Option<&'s str>,

    #[serde(rename = "qth", deserialize_with = "helpers::empty_str_as_none")]
    location: Option<&'s str>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    name: Option<&'s str>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    power: Option<&'s str>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    misctext: Option<&'s str>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    prec: Option<&'s str>,

    #[serde(rename = "zone")]
    pub cq_zone: u8,
    ck: u32,

    #[serde(rename = "ismultiplier1", deserialize_with = "bool_from_int")]
    pub is_mult_1: bool,
    #[serde(rename = "ismultiplier2", deserialize_with = "bool_from_int")]
    pub is_mult_2: bool,
    #[serde(rename = "ismultiplier3", deserialize_with = "bool_from_int")]
    pub is_mult_3: bool,

    #[serde(rename = "radionr")]
    pub radio_number: i32,
    #[serde(rename = "RadioInterfaced")]
    radio_interfaced: i32,
    #[serde(rename = "NetworkedCompNr")]
    pub networked_computer_number: i32,

    run1run2: i32,

    #[serde(
        rename = "RoverLocation",
        deserialize_with = "helpers::empty_str_as_none"
    )]
    rover_location: Option<&'s str>,

    #[serde(rename = "IsOriginal", deserialize_with = "bool_from_string")]
    is_original: bool,

    #[serde(
        rename = "NetBiosName",
        deserialize_with = "helpers::empty_str_as_none"
    )]
    net_bios_name: Option<&'s str>,

    #[serde(rename = "IsRunQSO", deserialize_with = "bool_from_int")]
    pub is_run_qso: bool,

    #[serde(rename = "IsClaimedQso", deserialize_with = "bool_from_int")]
    pub is_claimed_qso: bool,
    pub points: i32,

    #[serde(rename = "StationName")]
    station_name: &'s str,

    #[serde(rename = "ID")]
    pub id: &'s str,
}

#[derive(Debug, Deserialize)]
pub struct ContactDelete<'s> {
    app: &'s str,

    #[serde(with = "n1mm_date_format")]
    pub timestamp: time::PrimitiveDateTime,

    #[serde(rename = "contestnr")]
    pub contest_number: i32,

    pub call: &'s str,

    #[serde(rename = "StationName")]
    pub station_name: &'s str,

    #[serde(rename = "ID")]
    pub id: &'s str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RadioInfo<'s> {
    #[serde(rename = "app")]
    app: &'s str,

    station_name: &'s str,

    #[serde(rename = "RadioNr")]
    pub radio_number: i32,
    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    pub radio_name: Option<&'s str>,
    pub is_connected: Option<()>,

    #[serde(rename = "Freq", deserialize_with = "hz_from_freq")]
    pub rx_frequency: i64,
    #[serde(rename = "TXFreq", deserialize_with = "hz_from_freq")]
    pub tx_frequency: i64,

    pub mode: &'s str,

    #[serde(rename = "OpCall")]
    pub operator_callsign: &'s str,

    #[serde(deserialize_with = "bool_from_string")]
    pub is_running: bool,

    antenna: Option<i32>,

    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    rotors: Option<&'s str>,

    #[serde(rename = "FocusRadioNr")]
    focus_radio_number: i32,

    #[serde(rename = "ActiveRadioNr")]
    active_radio_number: i32,

    #[serde(deserialize_with = "bool_from_string")]
    is_stereo: bool,

    #[serde(deserialize_with = "bool_from_string")]
    is_split: bool,

    /// If N1MM is transmitting (false if the op is transmitting manually)
    #[serde(deserialize_with = "bool_from_string")]
    is_transmitting: bool,

    /// If N1MM is transmitting, this is the label for the function key used to transmit
    #[serde(deserialize_with = "helpers::empty_str_as_none")]
    function_key_caption: Option<&'s str>,

    #[serde(rename = "AuxAntSelected")]
    aux_antenna_selected: i32,
    #[serde(
        rename = "AuxAntSelectedName",
        deserialize_with = "helpers::empty_str_as_none"
    )]
    aux_antenna_selected_name: Option<&'s str>,
}

#[derive(Debug, Deserialize)]
pub struct ScoreInfo<'s> {
    #[serde(rename = "contest")]
    contest_name: &'s str,
    class: Class<'s>,
}

#[derive(Debug, Deserialize)]
pub struct Class<'s> {
    #[serde(rename = "@power")]
    power: &'s str,
    #[serde(rename = "@assisted")]
    assisted: &'s str,
    #[serde(rename = "@transmitter")]
    transmitter: &'s str,
    #[serde(rename = "@ops")]
    allowed_operators: &'s str,
    #[serde(rename = "@bands")]
    allowed_bands: &'s str,
    #[serde(rename = "@mode")]
    allowed_mode: &'s str,
    #[serde(rename = "@overlay")]
    overlay: &'s str,
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    match <&str>::deserialize(deserializer)? {
        "False" => Ok(false),
        "True" => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(other),
            &"False or True",
        )),
    }
}

fn hz_from_freq<'de, D>(de: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(i64::deserialize(de)? * 10)
}

time::serde::format_description!(
    n1mm_date_format,
    PrimitiveDateTime,
    "[year]-[month]-[day] [hour]:[minute]:[second][end]"
);
