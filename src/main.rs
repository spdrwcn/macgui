//#![windows_subsystem = "windows"]

use clap::{App, Arg};
use eframe::egui;
use egui::ImageData;
use image::{DynamicImage, Luma};
use qrcode::QrCode;
use std::fmt::Display;

mod mac;
mod redis;
mod sysinfo;

fn main() {
    let matches = App::new("macgui")
        .version("1.4.5")
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
        .get_matches();
    let ip_address = matches.value_of("ip").unwrap();
    let serial_number = handle_error(
        sysinfo::get_bios_serial_number(),
        String::from("Unknown"),
        "fetching serial number",
    );
    let cpu_name = handle_error(
        sysinfo::cpu_name(),
        String::from("Unknown"),
        "fetching cpu name",
    );
    let ramgb = match sysinfo::ram_info() {
        Ok(total_gb) => format!("{}", total_gb),
        Err(e) => format!("Error getting RAM info: {}", e),
    };
    let disk_info = handle_error(
        sysinfo::get_disk_info(),
        String::from("Unknown"),
        "getting disk info",
    );
    let gpu_name = handle_error(
        sysinfo::get_gpu_info(),
        String::from("Unknown"),
        "getting gpu info",
    );
    let (wired_mac, wireless_mac, bluetooth_mac) = mac::get_mac_addresses();
    println!("处理器: \n{}\n", cpu_name);
    println!("内存: \n{} GB\n", ramgb);
    println!("硬盘: \n{}", disk_info);
    println!("显卡: \n{}", gpu_name);
    println!("序列号: {}", serial_number);
    println!("有线MAC地址: {}", wired_mac);
    println!("无线MAC地址: {}", wireless_mac);
    println!("蓝牙MAC地址: {}\n", bluetooth_mac);

    let redis_status = redis::write_mac_to_redis(
        &ip_address,
        &serial_number,
        &wired_mac,
        &wireless_mac,
        &bluetooth_mac,
    );

    let mac_qr: String = format!(
        "{},{},{},{}",
        serial_number, wired_mac, wireless_mac, bluetooth_mac
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    let _ = eframe::run_simple_native("MAC地址采集客户端", options, move |ctx, _frame| {
        setup_custom_fonts(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left_panel")
                .resizable(false)
                .default_width(200.0)
                .width_range(350.0..=350.0)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading(format!("处理器: \n{}\n", cpu_name));
                        ui.heading(format!("内存: \n{} GB\n", ramgb));
                        ui.heading(format!("硬盘: \n{}", disk_info));
                        ui.heading(format!("显卡: \n{}", gpu_name));
                        ui.heading(format!("序列号: {}", serial_number));
                        ui.heading(format!("有线MAC地址: {}", wired_mac));
                        ui.heading(format!("无线MAC地址: {}", wireless_mac));
                        ui.heading(format!("蓝牙MAC地址: {}\n", bluetooth_mac));
                        ui.heading(&redis_status);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label("SN-MAC地址二维码");
                    let img = ui.ctx().load_texture(
                        "qr_code",
                        generate_qrcode_imagedata(&mac_qr),
                        Default::default(),
                    );
                    ui.add(egui::Image::new(&img));
                });
            });
        });
    });
}

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

fn handle_error<T, E: Display>(result: Result<T, E>, default: T, error_msg: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error {}: {}", error_msg, err);
            default
        }
    }
}
