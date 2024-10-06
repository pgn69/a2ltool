use a2lfile::{A2lFile, Characteristic, Measurement, RecordLayout};
use std::collections::HashMap;

pub(crate) fn search_measurements<'a>(
    a2l_file: &'a A2lFile,
    regex_strings: &[&str],
    log_messages: &mut Vec<String>,
) -> HashMap<String, &'a Measurement> {
    let mut found = HashMap::new();

    let compiled_regexes = regex_strings
        .iter()
        .map(|re| {
            // extend the regex to match only the whole string, not just a substring
            let extended_regex = if !re.starts_with('^') && !re.ends_with('$') {
                format!("^{re}$")
            } else {
                re.to_string()
            };
            regex::Regex::new(&extended_regex).unwrap()
        })
        .collect::<Vec<_>>();

    for module in &a2l_file.project.module {
        // search all measurements that match any of the regexes
        for measurement in &module.measurement {
            for regex in &compiled_regexes {
                if regex.is_match(&measurement.name) {
                    found.insert(measurement.name.clone(), measurement);
                }
            }
        }
    }

    found
}

pub(crate) fn search_characteristics<'a>(
    a2l_file: &'a A2lFile,
    regex_strings: &[&str],
    log_messages: &mut Vec<String>,
) -> HashMap<String, &'a Characteristic> {
    let mut found = HashMap::new();

    let compiled_regexes = regex_strings
        .iter()
        .map(|re| {
            // extend the regex to match only the whole string, not just a substring
            let extended_regex = if !re.starts_with('^') && !re.ends_with('$') {
                format!("^{re}$")
            } else {
                re.to_string()
            };
            regex::Regex::new(&extended_regex).unwrap()
        })
        .collect::<Vec<_>>();

    for module in &a2l_file.project.module {
        // search all characteristics that match any of the regexes
        for characteristic in &module.characteristic {
            for regex in &compiled_regexes {
                if regex.is_match(&characteristic.name) {
                    found.insert(characteristic.name.clone(), characteristic);
                }
            }
        }
    }

    found
}

pub(crate) fn search_reord_layout<'a>(
    a2l_file: &'a A2lFile,
    regex_strings: &[&str],
    log_messages: &mut Vec<String>,
) -> HashMap<String, &'a RecordLayout> {
    let mut found = HashMap::new();

    let compiled_regexes = regex_strings
        .iter()
        .map(|re| {
            // extend the regex to match only the whole string, not just a substring
            let extended_regex = if !re.starts_with('^') && !re.ends_with('$') {
                format!("^{re}$")
            } else {
                re.to_string()
            };
            regex::Regex::new(&extended_regex).unwrap()
        })
        .collect::<Vec<_>>();

    for module in &a2l_file.project.module {
        // search all characteristics that match any of the regexes
        for record_layout in &module.record_layout {
            for regex in &compiled_regexes {
                if regex.is_match(&record_layout.name) {
                    found.insert(record_layout.name.clone(), record_layout);
                }
            }
        }
    }

    found
}
