use std::net::UdpSocket;

fn main() {
    println!("Запускаем тестовый UDP-сервер на порту 42...");
    // Занимаем (открываем) порт 42 на локальном компьютере
    let _socket = UdpSocket::bind("127.0.0.1:42").unwrap();

    println!("Порт 42 успешно открыт! Нажмите Ctrl+C для закрытия.");
    // Заставляем программу спать вечно, чтобы порт оставался открытым
    std::thread::sleep(std::time::Duration::from_secs(999999));
}
