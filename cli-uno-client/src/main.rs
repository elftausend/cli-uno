use std::{net::TcpStream, io::Read, sync::{Mutex, Arc}};

const IPPORT: &str = "127.0.0.1:11000";
const BUFFER: usize = 256;

pub struct Player {
    cards: Vec<String>,
    shown: String,
    stream: TcpStream,
}

impl Player {
    pub fn new(stream: TcpStream) -> Player {
        Player {
            cards: Vec::new(),
            shown: String::new(),
            stream,
        }
    }
}

fn set_cards(player: Arc<Mutex<Player>>, a: String) {
    let mut cards = a.split('|').map(|card| card.to_string()).collect::<Vec<String>>();
    cards.remove(cards.len()-1);
    println!("cards: {:?}", cards);
    player.lock().unwrap().cards = cards;
}

fn receive_cards(player: Arc<Mutex<Player>>, card_data: Vec<u8>, n: usize) {
    std::thread::spawn(move || {
        let a = String::from_utf8_lossy(&card_data[..n-1]).to_string();
        set_cards(player, a);
    });
}

fn receive_shown(player: Arc<Mutex<Player>>, shown_data: Vec<u8>, n: usize) {
    std::thread::spawn(move || {
        let card = String::from_utf8_lossy(&shown_data[..n-1]).to_string();
        println!("Shown card: {}", card);
        player.lock().unwrap().shown = card;
    });
}

pub fn listen(stream: TcpStream) {
    let mut vec = vec![0u8; BUFFER];

    let player = Arc::new(Mutex::new(Player::new(stream)));

    loop {
        match player.lock().unwrap().stream.read(&mut vec) {
            Ok(n) => {
                let player1 = player.clone();
                if vec[n-1] == 1 {
                    receive_cards(player1.clone(), vec.clone(), n);
                }

                if vec[n-1] == 2 {
                    receive_shown(player1.clone(), vec.clone(), n);
                }

            },
            Err(_) => todo!(),
        }
    }
}

fn main() {
    let stream = std::net::TcpStream::connect(IPPORT).unwrap();

    listen(stream);

}
