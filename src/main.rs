use std::env;
use std::net::{UdpSocket, IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex, mpsc};

#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawSocket;

#[cfg(target_os = "windows")]
const SIO_UDP_CONNRESET: u32 = 0x9800000C;

fn scan_udp_port(ip: IpAddr, port: u16) -> bool {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return false,
    };

    #[cfg(target_os = "windows")]
    {

        extern "system" {
            fn ioctlsocket(s: usize, cmd: i32, argp: *mut u32) -> i32;
        }

        let raw_socket = socket.as_raw_socket() as usize;
        let mut flag: u32 = 1; // 1 = включить сброс при ошибке, 0 = выключить
        
        unsafe {
            ioctlsocket(raw_socket, SIO_UDP_CONNRESET as i32, &mut flag);
        }
    }

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
        Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionRefused || 
                      e.kind() == std::io::ErrorKind::ConnectionReset => {
            false
        }
        _ => true,
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let target_ip: IpAddr;

    if args.len() < 2 {
        println!("Внимание: IP-адрес не указан. Используется локальный 127.0.0.1");
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

    let ports_to_scan: Vec<u16> = (1..30000).collect();
    let total_ports = ports_to_scan.len();

    let ports_queue = Arc::new(Mutex::new(ports_to_scan));
    let (tx, rx) = mpsc::channel();

    // Оптимальный баланс скорости и точности для Windows после исправления сокета
    let num_threads = 6; 
    let mut thread_handles = Vec::new();

    println!("Запуск Windows-оптимизированного сканирования для {}...", target_ip);
    println!("Потоков: {}, Ожидайте результаты...\n", num_threads);

    for _ in 0..num_threads {
        let ports = Arc::clone(&ports_queue);
        let tx_clone = tx.clone();

        let handle = thread::spawn(move || {
            loop {
                let port = {
                    let mut queue = ports.lock().unwrap();
                    if queue.is_empty() {
                        break;
                    }
                    queue.remove(0)
                };

                if scan_udp_port(target_ip, port) {
                    let _ = tx_clone.send(port);
                }

                // Небольшая задержка, чтобы ОС успевала обрабатывать буферы прерываний
                thread::sleep(Duration::from_millis(30));
            }
        });
        thread_handles.push(handle);
    }

    drop(tx);

    for open_port in rx {
        println!("[+] Порт {} [UDP] открыт / фильтруется", open_port);
    }

    for handle in thread_handles {
        let _ = handle.join();
    }

    println!("\nСканирование всех {} портов успешно завершено.", total_ports);
}
