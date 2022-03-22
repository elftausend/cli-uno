use std::{net::TcpStream, io::{Read, Write, BufRead}};

//const IPPORT: &str = "10.30.0.137:12000";
const IPPORT: &str = "127.0.0.1:11000";
const BUFFER: usize = 256;

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



pub fn card_check(selected: &str, shown: &str) -> bool {
    if selected == shown {
        return true;
    }

    let sel_chars = selected.as_bytes();
    let shown_chars = shown.as_bytes();

    if sel_chars[0] == shown_chars[0] {
        return true;
    } else if sel_chars[1] == shown_chars[1] {
        return true;
    }
    false

}

fn colored_card_print(card: &str, split: &str) {
    //print!("Your cards: ");{
    let mut chars = card.chars();
    print!("{}{}", split, chars.next().unwrap());
    let char = chars.next().unwrap();
    if char == 'r' {
        print!("\x1B[1;31mr\x1B[0m");
    }
    if char == 'b' {
        print!("\x1B[1;34mb\x1B[0m");
    }
    if char == 'g' {
        print!("\x1B[1;32mg\x1B[0m");
    }
    if char == 'y' {            
        print!("\x1B[1;33my\x1B[0m");
    }
}

fn set_cards(player: &mut Player, a: String) {
    let mut cards = a.split('|').map(|card| card.to_string()).collect::<Vec<String>>();
    cards.remove(cards.len()-1);

    print!("Your cards:");
    for card in cards.iter() {
        colored_card_print(card, " | ");
    }
    print!(" |");
    println!();
    
    player.cards = cards;
}

fn listen_no(mut player: Player) {
    let mut vec = vec![0u8; BUFFER];

    loop {
        match player.stream.read(&mut vec) {
            Ok(n) => {
                if vec[n-1] == 1 {
                    let a = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    set_cards(&mut player, a);
                    player.stream.write_all(&vec!(5u8)).unwrap();
                    
                }
                if vec[n-1] == 2 {
                    let card = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    print!("Shown card: ");
                    colored_card_print(&card, "");
                    println!();
                    
                    player.shown = card;
                    player.stream.write_all(&vec!(5u8)).unwrap();
                    
                }
                if vec[n-1] == 3 {                    
                    loop {
                        println!("Select a card [card from deck or 'abheben']:");
                    
                        let mut input = String::new();
                        std::io::stdin().lock().read_line(&mut input).unwrap();
                        let input = input.trim();
                        
                        if input == "abheben" {
                            let mut read = vec![0u8; BUFFER];
                            
                            player.stream.write_all(&[6, 6, 6, 6, 6, 6, 6, 6,]).unwrap();
                            let n = player.stream.read(&mut read).unwrap();
                            player.stream.write_all(&vec!(5u8)).unwrap();

                            let a = String::from_utf8_lossy(&read[..n-1]).to_string();
                            set_cards(&mut player, a);

                        } else {
                            if player.cards.contains(&input.to_string()) {
                                if card_check(input, &player.shown) {
                                    player.stream.write_all(input.as_bytes()).unwrap();
                                    break;
                                } else {
                                    println!("This card cannot be placed on the shown card {}!", player.shown);
                                }
                            
                            } else {
                                println!("Invalid card!");
                                continue;
                            }
                        }
                        
                    }
               
                }
                vec = vec![0u8; BUFFER];
            },
            Err(_) => todo!(),
        }
    }
}

fn main() {
    

    let stream = std::net::TcpStream::connect(IPPORT).unwrap();

    listen_no(Player::new(stream));
    
}
