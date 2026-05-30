use std::env;
use std::net::{UdpSocket, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::time::Duration;
use std::thread;

// Функция для проверки одного конкретного UDP-порта
fn scan_udp_port(ip: IpAddr, port: u16) -> bool {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return false,
    };

    if socket.set_read_timeout(Some(Duration::from_secs(1))).is_err() {
        return false;
    }

    if socket.connect((ip, port)).is_err() {
        return false;
    }

    let empty_payload = [];
    if socket.send(&empty_payload).is_err() {
        return false;
    }

    let mut buf = [0; 1];
    match socket.recv(&mut buf) {
        // Ошибка означает, что получен ICMP-ответ "Port Unreachable" -> порт закрыт
        Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionRefused || 
                      e.kind() == std::io::ErrorKind::ConnectionReset => {
            false
        }
        // Во всех остальных случаях (таймаут или данные) порт считается открытым/фильтруемым
        _ => true,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let target_ip: IpAddr;

    if args.len() < 2 {
        println!("Внимание: IP-адрес не указан. Используется локальный 127.0.0.1");
        println!("Пример для сканирования внешнего IP: cargo run -- 8.8.8.8\n");
        target_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    } else {
        target_ip = match IpAddr::from_str(&args[1]) {
            Ok(ip) => ip,
            Err(_) => {
                println!("Ошибка: предоставлен некорректный формат IP-адреса!");
                return;
            }
        };
    }

    // Диапазон портов для сканирования
    let ports_to_scan: Vec<u16> = (1..150).collect();

    println!("Запуск UDP-сканирования для узла {}...\n", target_ip);

    for port in ports_to_scan {
        if scan_udp_port(target_ip, port) {
            println!("[+] Порт {} [UDP] открыт / фильтруется", port);
        }
        
        // Пауза, чтобы сетевой стек не захлебывался
        thread::sleep(Duration::from_millis(15));
    }

    println!("\nСканирование успешно завершено.");
}
