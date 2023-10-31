// @generated automatically by Diesel CLI.

diesel::table! {
    contacts (id) {
        id -> Nullable<Text>,
        recv_callsign -> Text,
        sent_callsign -> Text,
        recv_signal_report -> Text,
        sent_signal_report -> Text,
        timestamp -> Timestamp,
        mode -> Text,
        freq_rx -> BigInt,
        freq_tx -> BigInt,
        exchange1 -> Nullable<Text>,
        section -> Nullable<Text>,
        prefix_wpx -> Nullable<Text>,
        cq_zone -> SmallInt,
        operator -> Nullable<Text>,
        contest_name -> Nullable<Text>,
        is_mult_1 -> Bool,
        is_mult_2 -> Bool,
        is_mult_3 -> Bool,
        is_run_qso -> Bool,
        is_claimed_qso -> Bool,
        points -> Integer,
        location_source -> Text,
        latitude -> Nullable<Float>,
        longitude -> Nullable<Float>,
    }
}
