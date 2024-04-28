use regex::Regex;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};

pub fn get_mac_addresses() -> (String, String, String) {
    let output = Command::new("wmic")
        .args(&[
            "path",
            "win32_networkadapter",
            "get",
            "name,macaddress,physicaladapter",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute wmic command")
        .stdout
        .unwrap();

    let conditions = [
        (
            "wired",
            vec![
                vec!["gbe", "true"]
            ],
        ),
        (
            "wireless",
            vec![
                vec!["wi-fi", "true"],
                vec!["wi-fi", "ax"],
                vec!["wireless", "true"]
            ],
        ),
        (
            "bluetooth",
            vec![
                vec!["bluetooth", "true"]
            ],
        ),
    ];

    let reader = BufReader::new(output);

    let mut mac_addresses = ["未采集"; 3].map(String::from);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line_lower = line.to_lowercase();

        for (index, (_adapter_type, keywords_groups)) in conditions.iter().enumerate() {
            let mut matched = false;
            for keywords in keywords_groups {
                if keywords.iter().all(|kw| line_lower.contains(kw)) {
                    matched = true;
                    break;
                }
            }
            if matched {
                if let Some(mac) = extract_mac_address(&line) {
                    mac_addresses[index] = mac;
                    break; // No need to check further conditions for this line
                }
            }
        }
    }

    (
        mac_addresses[0].clone(),
        mac_addresses[1].clone(),
        mac_addresses[2].clone(),
    )
}

fn extract_mac_address(line: &str) -> Option<String> {
    let mac_regex = Regex::new(r"([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})").unwrap();
    mac_regex
        .captures(line)?
        .get(0)?
        .as_str()
        .to_string()
        .into()
}
