const IPPORT: &str = "127.0.0.1:11000";

fn main() {
    let stream = std::net::TcpStream::connect(IPPORT).unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

}
