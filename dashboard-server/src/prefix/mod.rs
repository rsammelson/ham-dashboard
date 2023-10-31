use std::collections::HashMap;

const PREFIX_DATA: &str = include_str!("area-ok1rr.tbl");

lazy_static::lazy_static! {
    static ref PREFIX_DATA_PARSED: Vec<(regex::Regex, &'static str)> = {
        parse_prefix_data()
    };
}

fn parse_prefix_data() -> Vec<(regex::Regex, &'static str)> {
    let mut vec = Vec::new();
    for line in PREFIX_DATA.split('\n') {
        if line.trim_start().is_empty() {
            continue;
        }

        let (prefixes, data) = line.split_once('|').unwrap();
        for prefix in prefixes.split(' ') {
            if prefix.is_empty() {
                continue;
            }

            let p: String = std::iter::once('^')
                .chain(prefix.chars())
                .flat_map(|chr| {
                    let iter: Box<dyn Iterator<Item = char>> = match chr {
                        '#' => Box::new("\\d".chars()),
                        '%' => Box::new(".".chars()),
                        c => Box::new([c].into_iter()),
                    };
                    iter
                })
                .collect();
            vec.push((regex::Regex::new(&p).unwrap(), data));
        }
    }
    vec
}

pub fn get_location_for_callsign(callsign: &str) -> Option<(f32, f32)> {
    log::debug!("Resolving location of {} using prefixes", callsign);

    let mut matches: Vec<(usize, &str)> = PREFIX_DATA_PARSED
        .iter()
        .filter_map(|(re, s)| re.shortest_match(callsign).map(|i| (i, *s)))
        .collect();

    matches.sort_by(|(x, _), (y, _)| y.cmp(x));

    log::debug!(
        "Got {} matches for {} - {:?}",
        matches.len(),
        callsign,
        matches
    );

    let data = match (matches.get(0), matches.get(1)) {
        (None, _) => return None,
        (Some((_, d)), None) => *d,
        (Some((x, d)), Some((y, _))) => {
            if x > y {
                *d
            } else {
                log::warn!(
                    "Got multiple entries of the same match length for {}",
                    callsign
                );
                *d
            }
        }
    };

    let parts: Vec<&str> = data.split('|').collect();

    let lat_idx = parts[3].char_indices().last().unwrap();
    let lat = match lat_idx.1 {
        'N' => 1f32,
        'S' => -1f32,
        _ => panic!(),
    } * str::parse::<f32>(&parts[3][..lat_idx.0]).unwrap();

    let lng_idx = parts[4].char_indices().last().unwrap();
    let lng = match lng_idx.1 {
        'E' => 1f32,
        'W' => -1f32,
        _ => panic!(),
    } * str::parse::<f32>(&parts[4][..lng_idx.0]).unwrap();

    Some((lat, lng))
}
