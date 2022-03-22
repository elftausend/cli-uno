use std::{net::TcpStream, io::{Read, Write, BufRead}};


//const IPPORT: &str = "10.30.0.137:12000";
const IPPORT: &str = "127.0.0.1:11000";

/*
pub async fn wait_till_clear(stream: &mut TcpStream) {
    let mut buf = [0u8; 1];
    loop {
        match stream.read(&mut buf) {
            Ok(n) => {
                if n != 0 {
                    if buf.first().unwrap() == &5 {
                        break;
                    }
                } else {
                    break;
                }
            }
            Err(_) => {}
        }
    }
}
*/

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

pub static mut PLAYER: Option<Player> = None;
const BUFFER: usize = 256;

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

fn set_cards(a: String) {
    let mut cards = a.split('|').map(|card| card.to_string()).collect::<Vec<String>>();
    cards.remove(cards.len()-1);
    println!("cards: {:?}", cards);
    unsafe {
        PLAYER.as_mut().unwrap().cards = cards;
    }
}


fn listen(stream: &mut TcpStream) {
    let mut vec = vec![0u8; BUFFER];

    loop {
        match stream.read(&mut vec) {
            Ok(n) => {
                if vec[n-1] == 1 {
                    let a = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    set_cards(a);
                    stream.write_all(&vec!(5u8)).unwrap();
                    
                }
                if vec[n-1] == 2 {
                    let card = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    println!("Shown card: {}", card);
                    unsafe {
                        PLAYER.as_mut().unwrap().shown = card;
                    }
                    stream.write_all(&vec!(5u8)).unwrap();
                    
                }
                if vec[n-1] == 3 {
                    unsafe {
                        let cards = &PLAYER.as_ref().unwrap().cards;
                        let shown = &PLAYER.as_ref().unwrap().shown;
                        
                        //println!("cards: {:?}, shown: {}", cards, shown);
                        loop {
                            println!("Select a card [card from deck or 'abheben']:");
                        
                            let mut input = String::new();
                            std::io::stdin().lock().read_line(&mut input).unwrap();
                            let input = input.trim();
                            
                            if input == "abheben" {
                                let mut read = vec![0u8; BUFFER];
                                
                                PLAYER.as_mut().unwrap().stream.write_all(&[6, 6, 6, 6, 6, 6, 6, 6,]).unwrap();
                                let n = PLAYER.as_mut().unwrap().stream.read(&mut read).unwrap();
                                stream.write_all(&vec!(5u8)).unwrap();

                                let a = String::from_utf8_lossy(&read[..n-1]).to_string();
                                set_cards(a);

                            } else {
                                if cards.contains(&input.to_string()) {
                                    if card_check(input, &PLAYER.as_ref().unwrap().shown) {
                                        PLAYER.as_mut().unwrap().stream.write_all(input.as_bytes()).unwrap();
                                        break;
                                    } else {
                                        println!("This card cannot be placed on the shown card {}!", PLAYER.as_ref().unwrap().shown);
                                    }
                                
                                } else {
                                    println!("Invalid card!");
                                    continue;
                                }
                            }
                         
                        }
       
                    }
                    
                    //let cards = unsafe {&PLAYER.as_ref().unwrap().cards};
                }
                //println!("pass");
                vec = vec![0u8; BUFFER];
            },
            Err(_) => todo!(),
        }
    }
}

fn main() {
    

    let stream = std::net::TcpStream::connect(IPPORT).unwrap();

    
    unsafe {
        /* 
        let builder = std::thread::Builder::new();
        builder.spawn(move || {
            loop {
                //let cards = &PLAYER.as_ref().unwrap().cards;
                //let shown = &PLAYER.as_ref().unwrap().shown;
                //println!("cards: {:?}, shown: {}", cards, shown);
                std::thread::sleep(std::time::Duration::from_secs(4));
            }

                
        }).unwrap();

        let builder = std::thread::Builder::new();
        builder.spawn(move || {
            loop {
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                let cards = &PLAYER.as_ref().unwrap().cards;
//                println!("cards2: {:?}", cards);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }

                
        }).unwrap();
        */
    
        PLAYER = Some(Player::new(stream));
    
        listen(&mut PLAYER.as_mut().unwrap().stream)
        

        /* 
        std::thread::spawn(move || {
            
        });
        */
       // tokio::spawn(async move {
            
        //});
        
    }
    
}
