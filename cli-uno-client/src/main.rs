use std::{net::TcpStream, io::{Read, Write, BufRead}};

//const IPPORT: &str = "10.30.0.137:12000";
const IPPORT: &str = "172.23.1.152:11000";
const BUFFER: usize = 1024;

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
    } 
    if sel_chars[1] == shown_chars[1] {
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

fn print_shown(card: &str) {
    print!("Shown card: ");
    colored_card_print(card, "");
    println!();
    println!();
}

fn listen(mut player: Player) {
    let mut vec = vec![0u8; BUFFER];

    loop {
        match player.stream.read(&mut vec) {
            Ok(n) => {
                //receive cards
                if vec[n-1] == 1 {
                    let a = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    set_cards(&mut player, a);
                    player.stream.write_all(&[5u8]).unwrap();
                    
                }
                //receive shown card
                if vec[n-1] == 2 {
                    let card = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    
                    print_shown(&card);
                    
                    player.shown = card;
                    player.stream.write_all(&[5u8]).unwrap();
                    
                }

                //receive current player
                if vec[n-1] == 10 {
                    let current_player = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    println!("\x1B[1;94m{}\x1B[0m plays..", current_player.trim_end());
                    player.stream.write_all(&[5u8]).unwrap();
                }

                //receive all players
                if vec[n-1] == 11 {
                    let all_players = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    print!("\x1B[1;37mPlayers: \x1B[0m");
                    for player in all_players.split(';') {
                        print!("{}, ", player.trim()); 
                    }
                    println!();
                    player.stream.write_all(&[5u8]).unwrap();
                }

                //winning
                if vec[n-1] == 12 {
                    let current_player = String::from_utf8_lossy(&vec[..n-1]).to_string();
                    println!("\x1B[1;93m{}\x1B[0m has won this round!", current_player.trim_end());
                    player.stream.write_all(&[5u8]).unwrap();
                }

                //game "logic"
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
                            player.stream.write_all(&[5u8]).unwrap();

                            let cards = String::from_utf8_lossy(&read[..n-1]).to_string();
                            set_cards(&mut player, cards);

                            let n = player.stream.read(&mut read).unwrap();
                            let shown = String::from_utf8_lossy(&read[..n-1]).to_string();
                            print_shown(&shown);
                            player.stream.write_all(&[5u8]).unwrap();

                        } else if player.cards.contains(&input.to_string()) {
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

                //terminate client
                if vec[n-1] == 255 {
                    std::process::exit(0);
                }
                vec = vec![0u8; BUFFER];
            },
            Err(_) => todo!(),
        }
    }
}

fn main() {

    let mut stream = std::net::TcpStream::connect(IPPORT).unwrap();

    let mut username = String::new();
    println!("Enter username: ");
    std::io::stdin().read_line(&mut username).unwrap();
    stream.write_all(username.as_bytes()).unwrap();

    listen(Player::new(stream));
    
}
