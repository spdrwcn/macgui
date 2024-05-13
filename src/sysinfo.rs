use regex::Regex;
use std::error::Error;
use std::process::Command;

fn run_wmic_command_cpu_sn(command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cmd").arg("/c").arg(command).output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();

    // 假设我们只对第二行感兴趣，并提取其最后一个单词
    match lines.get(1) {
        Some(line) => {
            // 直接将第二行转换为String
            Ok(line.trim().to_string())
        }
        None => {
            // 如果第二行不存在，返回一个错误
            Err("Output did not contain the expected lines.".into())
        }
    }
}

pub fn get_bios_serial_number() -> Result<String, Box<dyn std::error::Error>> {
    run_wmic_command_cpu_sn("wmic bios get serialnumber")
}

pub fn cpu_name() -> Result<String, Box<dyn std::error::Error>> {
    run_wmic_command_cpu_sn("wmic cpu get name")
}

pub fn get_gpu_info() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cmd")
        .arg("/c")
        .arg("wmic path Win32_VideoController get name")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let lines = stdout.lines().skip(2);

    let result = lines.fold(String::new(), |acc, line| {
        acc + &format!("{}\n", line.trim())
    });

    Ok(result)
}

pub fn get_disk_info() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cmd")
        .arg("/c")
        .arg("wmic diskdrive get model,size")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 删除第一行
    let lines = stdout.lines().skip(1);

    let mut result = String::new();
    for line in lines {
        // 分割每行的Model和Size
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            // 假设Size是最后一部分且为数字
            let size_str = parts.last().unwrap();
            let size_gb = (size_str.parse::<u64>()? / 1_000_000_000).to_string();
            // 替换Size并重新组合字符串
            let new_line = format!("{} {}GB\n", parts[0], size_gb);
            result.push_str(&new_line);
        }
    }

    Ok(result)
}

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
