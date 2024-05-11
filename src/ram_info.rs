use regex::Regex;
use std::error::Error;
use std::process::Command;

pub fn ram_info() -> Result<f64, Box<dyn Error>> {
    let output = Command::new("cmd")
        .arg("/c")
        .arg("wmic memorychip get Capacity")
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to convert output to UTF-8: {}", e))?;

    // 假设只匹配纯数字（没有单位）
    let re = Regex::new(r"\b\d+\b").map_err(|e| format!("Failed to compile regex: {}", e))?;

    let mut total_capacity: u64 = 0;

    // 遍历所有匹配项
    for cap in re.captures_iter(&stdout) {
        if let Some(cap_text) = cap.get(0) {
            // 将捕获的字符串转换为u64，并累加到总容量中
            let capacity: u64 = cap_text
                .as_str()
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse number: {}", e))?;
            total_capacity = total_capacity
                .checked_add(capacity)
                .ok_or("Capacity overflow")?;
        }
    }

    // 转换为 GB（但注意，这里我们假设所有字节都是没有单位的）
    let total_gb = (total_capacity as f64) / (1024.0 * 1024.0 * 1024.0);

    Ok(total_gb)
}
