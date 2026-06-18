use std::net::UdpSocket;
// Нужно для демонстрации открытого порта

fn main() {
    println!("Запускаем тестовый UDP-сервер на порту 42...");
    let _socket = UdpSocket::bind("127.0.0.1:25000").unwrap();

    println!("Порт 42 успешно открыт! Нажмите Ctrl+C для закрытия.");
    std::thread::sleep(std::time::Duration::from_secs(999999));
}