// #![windows_subsystem = "windows"]

use clap::{App as ClapApp, Arg};
use eframe::egui;
use egui::ImageData;
use image::{DynamicImage, Luma};
use qrcode::QrCode;
use serde_json::json;
use simple_redis;
use std::io::{self, BufRead};
use std::process::Command;
use std::result::Result;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = ClapApp::new("macgui")
        .version("1.2.1")
        .author("h13317136163@163.com")
        .about("MAC地址采集程序")
        .arg(
            Arg::with_name("ip")
                .short("i")
                .long("ip")
                .value_name("IP_ADDRESS")
                .help("Redis数据库地址")
                .default_value("redis://127.0.0.1:6379/0"),
        )
        .arg(
            Arg::with_name("wired")
                .short("w")
                .long("wired")
                .value_name("Value")
                .multiple(true)
                .help("有线网卡匹配参数 [default: gbe true]"),
        )
        .arg(
            Arg::with_name("wireless")
                .short("l")
                .long("wireless")
                .value_name("Value")
                .multiple(true)
                .help("无线网卡匹配参数 [default: wi-fi true]"),
        )
        .arg(
            Arg::with_name("bluetooth")
                .short("b")
                .long("bluetooth")
                .value_name("Value")
                .multiple(true)
                .help("蓝牙匹配参数 [default: bluetooth true]"),
        )
        .get_matches();
    let ip_address = matches.value_of("ip").unwrap();
    let serial_number = get_bios_serial_number()?;
    let wiredk: Vec<&str> = if let Some(values) = matches.values_of("wired") {
        values.collect()
    } else {
        vec!["gbe", "true"]
    };
    let wirelessk: Vec<&str> = if let Some(values) = matches.values_of("wireless") {
        values.collect()
    } else {
        vec!["wi-fi", "true"]
    };
    let bluetoothk: Vec<&str> = if let Some(values) = matches.values_of("bluetooth") {
        values.collect()
    } else {
        vec!["bluetooth", "true"]
    };
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
    check_and_assign_if_empty(&mut wired_mac);
    check_and_assign_if_empty(&mut wireless_mac);
    check_and_assign_if_empty(&mut bluetooth_mac);

    let mut redis_ok = String::new();
    let mut redis_error = String::new();
    // redis写入MAC地址
    if mac_found {
        let json_data = json!({
            "wired_mac": wired_mac,
            "wireless_mac": wireless_mac,
            "bluetooth_mac": bluetooth_mac
        });
        let json_str = json_data.to_string();
        let mac_qr: String = format!("序列号:{}\n有线MAC地址:{}\n无线MAC地址:{}\n蓝牙MAC地址:{}",
            serial_number, wired_mac, wireless_mac, bluetooth_mac
        );
        if let Ok(mut client) = simple_redis::create(ip_address) {
            let set_result = client.set(&serial_number, &*json_str);
            if set_result.is_ok() {
                redis_ok = format!("MAC地址: 写入成功");
            } else {
                redis_error = format!("MAC地址: 写入失败");
            }
            let quit_result = client.quit();
            if quit_result.is_ok() {
            } else {
                redis_error = format!("Error: {}", quit_result.err().unwrap());
            }
        } else {
            redis_error = format!("Redis 服务端: 连接失败");
        }
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([450.0, 650.0]),
            ..Default::default()
        };
        let _ =
            eframe::run_simple_native("MAC地址采集客户端", options, move |ctx, _frame| {
                setup_custom_fonts(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("SN-MAC地址二维码");
                    let img = ui.ctx().load_texture(
                        "qr_code",
                        generate_qrcode_imagedata(&mac_qr),
                        Default::default(),
                    );
                    ui.add(egui::Image::new(&img));
                    ui.heading(format!("序列号: {}", serial_number));
                    ui.heading(format!("有线MAC地址: {}", wired_mac));
                    ui.heading(format!("无线MAC地址: {}", wireless_mac));
                    ui.heading(format!("蓝牙MAC地址: {}", bluetooth_mac));
                    ui.heading(&redis_ok);
                    ui.heading(&redis_error);
                });
            });
    }
    Ok(())
}
//MAC变量检查
fn check_and_assign_if_empty(s: &mut String) {
    if s.is_empty() {
        *s = "未采集".to_string();
    }
}
//MAC地址处理
fn extract_mac_address(line: &str) -> String {
    line.chars().take(17).collect::<String>()
}
//二维码生成
fn generate_qrcode_imagedata(content: &str) -> ImageData {
    let qr_code = QrCode::new(content).unwrap();
    let qr_code = qr_code.render::<Luma<u8>>().build();
    let qr_code = DynamicImage::ImageLuma8(qr_code);

    let size = [qr_code.width() as usize, qr_code.height() as usize];

    let qr_code = qr_code.to_rgba8();
    let qr_code = qr_code.as_flat_samples();

    ImageData::from(egui::ColorImage::from_rgba_unmultiplied(
        size,
        qr_code.as_slice(),
    ))
}
// 获取序列号
fn get_bios_serial_number() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("wmic")
        .arg("bios")
        .arg("get")
        .arg("serialnumber")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.split('\n').collect::<Vec<_>>();
    match lines.get(1) {
        Some(line) => {
            let serial = line.trim().split_whitespace().last().map(|s| s.to_string());
            match serial {
                Some(s) => Ok(s),
                None => Err("Failed to parse BIOS serial number from WMIC output.".into()),
            }
        }
        None => Err("WMIC output did not contain the expected lines.".into()),
    }
}
// 自定义字体
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
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
