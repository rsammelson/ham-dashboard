CREATE TABLE contacts (
  id VARCHAR PRIMARY KEY,

  recv_callsign VARCHAR NOT NULL,
  sent_callsign VARCHAR NOT NULL,

  recv_signal_report VARCHAR NOT NULL,
  sent_signal_report VARCHAR NOT NULL,

  timestamp TIMESTAMP NOT NULL,

  mode VARCHAR NOT NULL,
  freq_rx BIGINT NOT NULL,
  freq_tx BIGINT NOT NULL,

  exchange1 VARCHAR,
  section VARCHAR,
  prefix_wpx VARCHAR,
  cq_zone SMALLINT NOT NULL,

  operator VARCHAR,
  contest_name VARCHAR,

  is_mult_1 BOOL NOT NULL,
  is_mult_2 BOOL NOT NULL,
  is_mult_3 BOOL NOT NULL,

  is_run_qso BOOL NOT NULL,
  is_claimed_qso BOOL NOT NULL,
  points INT NOT NULL,

  location_source VARCHAR NOT NULL,
  latitude FLOAT,
  longitude FLOAT
)
