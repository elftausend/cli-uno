use std::{net::TcpStream, io::{Read, BufRead, Write}, sync::{Mutex, Arc}};

const IPPORT: &str = "127.0.0.1:11000";
const BUFFER: usize = 256;


fn main() {
    let stream = std::net::TcpStream::connect(IPPORT).unwrap();

    listen(stream);

}

pub struct Player {
    cards: Vec<String>,
    shown: String,
    stream: Arc<TcpStream>,
}

impl Player {
    pub fn new(stream: Arc<TcpStream>) -> Player {
        Player {
            cards: Vec::new(),
            shown: String::new(),
            stream,
        }
    }
}

pub fn card_check(selected: &str, shown: &str) -> bool {
    if selected == shown {
        return true;
    }

    let sel_chars = selected.as_bytes();
    let shown_chars = shown.as_bytes();

    if sel_chars[0] == shown_chars[0] {
        return true;
    } else if sel_chars.len() != shown_chars.len() {
        return false;
    } else if sel_chars[1..sel_chars.len()] == shown_chars[1..shown_chars.len()] {
        return true;
    } 
    false

}

fn set_cards(player: Arc<Mutex<Player>>, a: String) {
    let mut player_guard = player.lock().unwrap();
    let mut cards = a.split('|').map(|card| card.to_string()).collect::<Vec<String>>();
    cards.remove(cards.len()-1);
    println!("cards: {:?}", cards);
    {
        player_guard.cards = cards;
    }
    println!("set cards");
    
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
        {
            player.lock().unwrap().shown = card;
        }
        println!("set shown");
    });
}

fn receive_selecting(player: Arc<Mutex<Player>>) {
    std::thread::spawn(move || {
        
        println!("select");
        
        loop {
            println!("Select a card [card from deck or 'abheben']:");
            let guard = player.lock().unwrap();
            let cards = guard.cards.clone();
            let shown = guard.shown.clone();
            
            let a = Arc::as_ptr(&guard.stream);
            let stream = unsafe {&mut *(a as *mut TcpStream)};

            drop(guard);

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input == "abheben" {                
                stream.write_all(&[6, 6, 6, 6, 6, 6, 6, 6,]).unwrap();

            } else {                
                if cards.contains(&input.to_string()) {
                    if card_check(input, &shown) {

                        stream.write_all(input.as_bytes()).unwrap();
                        break;
                    } else {
                        println!("This card cannot be placed on the shown card {}!", shown);
                    }
                } else {
                    println!("Invalid card!");
                }
            }
        }
    });
}


pub fn listen(stream: TcpStream) {
    let mut vec = vec![0u8; BUFFER];

    let stream = Arc::new(stream);

    let a = Arc::as_ptr(&stream);
    let s = a as *mut TcpStream;
    
    
    let player = Arc::new(Mutex::new(Player::new(stream.clone())));
    
    loop {
        let player_mutex_clone = Arc::clone(&player);
        let n = unsafe {
            let n = match Read::read(&mut *s, &mut vec) {
                Ok(n) => {
                    n
                },
                Err(_) => todo!(),
            };
            n
        };
        
    
        if vec[n-1] == 1 {
            receive_cards(player_mutex_clone.clone(), vec.clone(), n);
        }
        
        if vec[n-1] == 2 {
            receive_shown(player_mutex_clone.clone(), vec.clone(), n);
        }

        if vec[n-1] == 3 {
            receive_selecting(player_mutex_clone);
        }
        /* 
        let player1 = player.clone();
        match player1.lock().unwrap().stream.read(&mut vec) {
            Ok(n) => {
                {
                    let a = &player.lock().unwrap().cards;
                    println!("a: {:?}", a);
                }
                if vec[n-1] == 1 {
                    receive_cards(player.clone(), vec.clone(), n);
                }

                if vec[n-1] == 2 {
                    receive_shown(player.clone(), vec.clone(), n);
                }

                if vec[n-1] == 3 {
                    //receive_selecting(player1.clone());
                }

            },
            Err(_) => todo!(),
        };
        */
        /*
        let mut guard = player.lock().unwrap();
        match guard.stream.read(&mut vec) {
            Ok(n) => {
                //let player1 = player.clone();
                drop(guard);
                {
                    let a = &player.lock().unwrap().cards;
                    println!("a: {:?}", a);
                }
                if vec[n-1] == 1 {
                    receive_cards(player.clone(), vec.clone(), n);
                }

                if vec[n-1] == 2 {
                    receive_shown(player.clone(), vec.clone(), n);
                }

                if vec[n-1] == 3 {
                    //receive_selecting(player1.clone());
                }

            },
            Err(_) => todo!(),
        }
        */
    }
}

