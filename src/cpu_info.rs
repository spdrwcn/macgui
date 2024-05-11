use std::process::Command;

pub fn cpu_name() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cmd")
        .arg("/c")
        .arg("wmic cpu get name")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.split('\n').collect::<Vec<_>>();
    match lines.get(1) {
        Some(line) => {
            let serial = line.trim().split_whitespace().last().map(|s| s.to_string());
            match serial {
                Some(s) => Ok(s),
                None => Err("Failed to parse CPU NAME from WMIC output.".into()),
            }
        }
        None => Err("WMIC CPU NAME output did not contain the expected lines.".into()),
    }
}
