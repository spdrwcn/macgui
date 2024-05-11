use serde_json::json;
use simple_redis;

pub fn write_mac_to_redis(
    ip_address: &str,
    serial_number: &str,
    wired_mac: &str,
    wireless_mac: &str,
    bluetooth_mac: &str,
) -> String {
    // 创建JSON数据
    let json_data = json!({
        "wired_mac": wired_mac,
        "wireless_mac": wireless_mac,
        "bluetooth_mac": bluetooth_mac
    });
    let json_str = json_data.to_string();

    // 尝试连接到Redis并设置数据
    let redis_status = match simple_redis::create(ip_address) {
        Ok(mut client) => match client.set(serial_number, &*json_str) {
            Err(error) => format!("MAC地址写入失败 \nRedis: {}", error),
            _ => format!("MAC地址写入成功"),
        },
        Err(_) => todo!(),
    };

    redis_status
}
