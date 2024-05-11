use std::process::Command;

pub fn get_disk_info() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("wmic")
        .arg("diskdrive")
        .arg("get")
        .arg("model,size")
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
