use clap::{App as ClapApp, Arg};
use eframe::egui;
use eframe::{run_native, App, CreationContext, Frame};
use simple_redis;
use std::io::{self, BufRead};
use std::process::Command;
use std::result::Result;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = ClapApp::new("macgui")
        .version("1.0.0")
        .author("h13317136163@163.com")
        .about("MAC地址采集程序")
        .arg(
            Arg::with_name("ip")
                .short("i")
                .long("ip")
                .value_name("IP_ADDRESS")
                .help("Redis数据库地址 例: redis://127.0.0.1:6379/0")
                .required(true),
        )
        .arg(
            Arg::with_name("wired")
                .short("w")
                .long("wired")
                .value_name("Value")
                .multiple(true)
                .help("有线网卡匹配参数"),
        )
        .arg(
            Arg::with_name("wireless")
                .short("l")
                .long("wireless")
                .value_name("Value")
                .multiple(true)
                .help("无线网卡匹配参数"),
        )
        .arg(
            Arg::with_name("bluetooth")
                .short("b")
                .long("bluetooth")
                .value_name("Value")
                .multiple(true)
                .help("蓝牙匹配参数"),
        )
        .get_matches();
    let ip_address = matches.value_of("ip").unwrap();
    let serial_number = get_bios_serial_number()?;
    let wiredk: Vec<&str> = matches.values_of("wired").unwrap().collect();
    let wirelessk: Vec<&str> = matches.values_of("wireless").unwrap().collect();
    let bluetoothk: Vec<&str> = matches.values_of("bluetooth").unwrap().collect();
    let output = Command::new("wmic")
        .args(&[
            "path",
            "win32_networkadapter",
            "get",
            "name,macaddress,physicaladapter",
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute wmic command")
        .stdout
        .unwrap();
    let reader = io::BufReader::new(output);
    let mut wired_mac = String::new();
    let mut wireless_mac = String::new();
    let mut bluetooth_mac = String::new();
    // 获取MAC地址
    let mut mac_found = false;
    for line in reader.lines() {
        let line = line.unwrap();
        let line_lower = line.to_lowercase();
        let contains_all = |keywords: &[&str]| keywords.iter().all(|kw| line_lower.contains(kw));
        if contains_all(&wiredk) {
            if wired_mac.is_empty() {
                wired_mac = extract_mac_address(&line);
                mac_found = true;
            }
        } else if contains_all(&wirelessk) {
            if wireless_mac.is_empty() {
                wireless_mac = extract_mac_address(&line);
                mac_found |= !mac_found;
            }
        } else if contains_all(&bluetoothk) {
            if bluetooth_mac.is_empty() {
                bluetooth_mac = extract_mac_address(&line);
                mac_found |= !mac_found;
            }
        }
    }
    let mut redis_ok = String::new();
    let mut redis_error = String::new();
    if mac_found {
        let macs_joined = format!("{} {} {}", wired_mac, wireless_mac, bluetooth_mac);
        match simple_redis::create(ip_address) {
            Ok(mut client) => {
                match client.set(&serial_number, &macs_joined) {
                    Ok(_) => {
                        redis_ok = "MAC地址: 写入成功".to_string();
                    }
                    Err(e) => {
                        redis_error = format!("MAC地址: 写入失败，原因: {}", e);
                    }
                }
                match client.quit() {
                    Ok(_) => {
                    }
                    Err(e) => {
                        if redis_error.is_empty() {
                            redis_error = format!("Error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                redis_error = format!("Redis 服务端: 连接失败，原因: {}", e);
            }
        }
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 600.0]),
            ..Default::default()
        };
        let _ = run_native(
            "MAC地址采集程序",
            options,
            Box::new(move |cc| {
                Box::new(MyEguiApp::new(
                    cc,
                    &redis_error,
                    &redis_ok,
                    &bluetooth_mac,
                    &serial_number,
                    &wired_mac,
                    &wireless_mac,
                ))
            }),
        );
    }
    Ok(())
}

// MAC格式处理
fn extract_mac_address(line: &str) -> String {
    line.chars().take(17).collect::<String>()
}

#[derive(Default)]
struct MyEguiApp {
    redis_error: String,
    redis_ok: String,
    bluetooth_mac: String,
    serial_number: String,
    wired_mac: String,
    wireless_mac: String,
}
impl MyEguiApp {
    fn new(
        cc: &CreationContext<'_>,
        redis_error: &str,
        redis_ok: &str,
        bluetooth_mac: &str,
        serial_number: &str,
        wired_mac: &str,
        wireless_mac: &str,
    ) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            redis_error: redis_error.to_string(),
            redis_ok: redis_ok.to_string(),
            bluetooth_mac: bluetooth_mac.to_string(),
            serial_number: serial_number.to_string(),
            wired_mac: wired_mac.to_string(),
            wireless_mac: wireless_mac.to_string(),
        }
    }
}
impl App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("序列号:     {}", self.serial_number));
            ui.heading(format!("有线MAC地址:{}", self.wired_mac));
            ui.heading(format!("无线MAC地址:{}", self.wireless_mac));
            ui.heading(format!("蓝牙MAC地址:{}", self.bluetooth_mac));
            ui.heading(format!("{}", self.redis_ok));
            ui.heading(&self.redis_error);
        });
    }
}
// 获取序列号
fn get_bios_serial_number() -> Result<String, Box<dyn Error>> {
    let output = Command::new("wmic")
        .args(&["bios", "get", "serialnumber"])
        .output()
        .map_err(|e| format!("Failed to execute WMIC command: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 2 {
        return Err(format!("Insufficient lines in WMIC output: {}", stdout).into());
    }
    let serial_number_part = lines[1].split_whitespace().last();
    serial_number_part
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Failed to find BIOS serial number in WMIC output.").into())?
}
// 自定义字体
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    //安装的字体支持.ttf和.otf文件
    //main.rs的同级目录）
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("cn.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());
    ctx.set_fonts(fonts);
}
