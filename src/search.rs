use a2lfile::{A2lFile,Characteristic, Measurement};
use std::collections::HashMap;

pub(crate) fn search_measurements<'a>(
    a2l_file: &'a A2lFile,
    regex_strings: &[&str],
    log_messages: &mut Vec<String>,
) -> HashMap<String, &'a Measurement> {
    let mut found_measurements = HashMap::new();

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

        // search all measurements that match any of the regexes
        for measurement in &module.measurement {
            for regex in &compiled_regexes {
                if regex.is_match(&measurement.name) {
                    found_measurements.insert(measurement.name.clone(), measurement);
                }
            }
        }

    found_measurements
}

pub(crate) fn search_characteristics<'a>(
    a2l_file: &'a A2lFile,
    regex_strings: &[&str],
    log_messages: &mut Vec<String>,
) -> HashMap<String, &'a Characteristic> {
    let mut found_characteristics = HashMap::new();

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
                    found_characteristics.insert(characteristic.name.clone(), characteristic);
                }
            }
        }
    }

    found_characteristics
}
