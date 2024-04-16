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
        .get_matches();

    let ip_address = matches.value_of("ip").unwrap();
    let serial_number = get_bios_serial_number()?;
    let wiredk: Vec<&str> = matches.values_of("wired").unwrap().collect();
    let wirelessk: Vec<&str> = matches.values_of("wireless").unwrap().collect();
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

    let mut mac_found = false;
    let reader = io::BufReader::new(output);
    let mut wired_mac = String::new();
    let mut wireless_mac = String::new();
    // 获取MAC地址
    for line in reader.lines() {  
        let line = line.unwrap();  
        let line_lower = line.to_lowercase();  
      
        // 检查是否包含所有有线关键字  
        let mut wired_all_matched = true;  
        for wiredkw in &wiredk {  
            if !line_lower.contains(wiredkw) {  
                wired_all_matched = false;  
                break;  
            }  
        }  
      
        // 检查是否包含所有无线关键字  
        let mut wireless_all_matched = true;  
        for wirelessw in &wirelessk {  
            if !line_lower.contains(wirelessw) {  
                wireless_all_matched = false;  
                break;  
            }  
        }  

        if wired_all_matched && wired_mac.is_empty() {  
            wired_mac = extract_mac_address(&line);  
            mac_found = true;  
        } else if wireless_all_matched && wireless_mac.is_empty() {  
            wireless_mac = extract_mac_address(&line);  
            if !mac_found {  
                mac_found = true;  
            }  
        }  
    }
    // 判断MAC地址
    let mac_err;
    if !wired_mac.is_empty() && !wireless_mac.is_empty() {
        mac_err = format!("已成功采集到MAC地址");
    } else if !wired_mac.is_empty() {
        mac_err = format!("已成功采集到有线MAC地址");
    } else if !wireless_mac.is_empty() {
        mac_err = format!("已成功采集到无线MAC地址");
    } else {
        mac_err = format!("未采集到MAC地址");
    }
    println!("{}", mac_err);

    let mut redis_ok = String::new();
    let mut redis_error = String::new();
    if mac_found {
        println!("Redis address: {}", ip_address);
        println!("SN: {}", serial_number);
        println!("有线MAC地址: {}", wired_mac);
        println!("无线MAC地址: {}", wireless_mac);

        // redis写入MAC地址
        let macs_joined4: String = format!("{} {}", wired_mac, wireless_mac);
        if let Ok(mut client) = simple_redis::create(ip_address) {
            let set_result = client.set(&*serial_number, &*macs_joined4);
            if set_result.is_ok() {
                redis_ok = format!("MAC地址: 写入成功");
                println!("{}", redis_ok);
            } else {
                redis_error = format!("MAC地址: 写入失败");
                println!("{}", redis_error);
            }

            let quit_result = client.quit();
            if quit_result.is_ok() {
                println!("退出数据库.");
            } else {
                redis_error = format!("Error: {}", quit_result.err().unwrap());
                println!("Error: {}", redis_error);
            }
        } else {
            redis_error = format!("Redis 服务端: 连接失败");
            println!("{}", redis_error);
        }
        let redis_error1: String = format!("{}", redis_error);
        let macs_joined3: String = format!(
            "SN:                         {}\n \n有线MAC地址:   {}\n无线MAC地址:   {}\n",
            serial_number, wired_mac, wireless_mac
        );
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
                    &macs_joined3,
                    &redis_error1,
                    &redis_ok,
                    &mac_err,
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
    greeting: String,
    redis_error1: String,
    redis_ok: String,
    mac_err: String,
}
impl MyEguiApp {
    fn new(
        cc: &CreationContext<'_>,
        greeting: &str,
        redis_error1: &str,
        redis_ok: &str,
        mac_err: &str,
    ) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            greeting: greeting.to_string(),
            redis_error1: redis_error1.to_string(),
            redis_ok: redis_ok.to_string(),
            mac_err: mac_err.to_string(),
        }
    }
}
impl App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.greeting);
            ui.heading(&self.mac_err);
            ui.heading(&self.redis_ok);
            ui.heading(&self.redis_error1);
        });
    }
}
// 获取序列号
fn get_bios_serial_number() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("wmic")
        .arg("bios")
        .arg("get")
        .arg("serialnumber")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    let serial_line = lines.get(1);
    if let Some(serial_line) = serial_line {
        let serial_number_part = serial_line.split_whitespace().last();
        if let Some(serial_number) = serial_number_part {
            return Ok(serial_number.to_string());
        }
    }
    Err(format!(
        "Failed to find BIOS serial number in WMIC output: {}",
        stdout
    )
    .into())
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
