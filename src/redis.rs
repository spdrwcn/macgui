use serde_json::json;
use simple_redis;

pub fn write_mac_to_redis(
    ip_address: &str,
    serial_number: &str,
    wired_mac: &str,
    wireless_mac: &str,
    bluetooth_mac: &str,
) -> (String, String) {
    let mut redis_ok = String::new();
    let mut redis_error = String::new();

    // 创建JSON数据
    let json_data = json!({
        "wired_mac": wired_mac,
        "wireless_mac": wireless_mac,
        "bluetooth_mac": bluetooth_mac
    });
    let json_str = json_data.to_string();

    // 尝试连接到Redis并设置数据
    if let Ok(mut client) = simple_redis::create(ip_address) {
        let set_result = client.set(serial_number, &*json_str);
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

    (redis_ok, redis_error)
}
