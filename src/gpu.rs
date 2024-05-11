use std::process::Command;  
  
pub fn get_gpu_info() -> Result<String, Box<dyn std::error::Error>> {  
    let output = Command::new("wmic")  
        .arg("path")  
        .arg("Win32_VideoController")  
        .arg("get")  
        .arg("name")  
        .output()?;  
  
    let stdout = String::from_utf8_lossy(&output.stdout);  
       
    let lines = stdout.lines().skip(2);  
      
    let result = lines.fold(String::new(), |acc, line| acc + &format!("{}\n", line.trim()));  
  
    Ok(result)  
}