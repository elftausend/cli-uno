use std::sync::Arc;

use rand::Rng;
use tokio::{net::{TcpListener, TcpStream}, sync::{Mutex, RwLock}, io::{AsyncWriteExt, AsyncReadExt}};

const IPPORT: &str = "172.23.1.152:11000";
const MIN_USERS: usize = 2;

pub const COLOR_ARRAY: [&str; 4] = ["r", "b", "y", "g"];
pub const CARD_COUNT: usize = 1;

pub fn rand_card() -> String {
    let mut rng = rand::thread_rng();
    let num: usize = rng.gen_range(0..10);
    let color = COLOR_ARRAY[rng.gen_range(0..4)];
    num.to_string() + color
}

#[derive(Debug)]
pub struct Player {
    stream: TcpStream,
    ready: bool,
    no_cards: bool,
    cards: Vec<String>,
    name: String,
}

impl Player {
    pub fn new(stream: TcpStream, name: String) -> Player {
        let mut cards = Vec::new();

        for _ in 0..CARD_COUNT {            
            cards.push(rand_card());
        }
        
        Player {
            stream,
            ready: false,
            no_cards: false,
            cards, 
            name
        }
    }
}

async fn sync_card(player: Arc<Mutex<Player>>) {
    let mut cards = Vec::<Vec<u8>>::new();
    
    for card in &player.lock().await.cards {
        cards.push(card.as_bytes().to_vec());
        cards.push("|".as_bytes().to_vec())
    }
    cards.push(vec![1]);
    player.lock().await.stream.write_all(&cards.concat()).await.unwrap();
    wait_till_clear(&mut player.lock().await.stream).await;
}

type Players = Arc<RwLock<Vec<Arc<Mutex<Player>>>>>;

async fn check_for_ready(players: Players) -> bool {
    let players = players.read().await;

    std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
    let mut ready = true;
    
    for player in players.iter() {
        if !player.lock().await.ready {
            ready = false;
        }
    }
    if players.len() >= MIN_USERS {
        ready
    } else {
        false
    }

}

async fn sync_shown_p(player: Arc<Mutex<Player>>, card: &str) {
    
    let mut bytes = card.as_bytes().to_vec();
    bytes.push(2);
    player.lock().await.stream.write_all(&bytes).await.unwrap();
    wait_till_clear(&mut player.lock().await.stream).await;
    

}

async fn sync_shown(players: Players, card: &str) {
    for player in players.write().await.iter_mut() {
        let mut bytes = card.as_bytes().to_vec();
        bytes.push(2);
        player.lock().await.stream.write_all(&bytes).await.unwrap();
        wait_till_clear(&mut player.lock().await.stream).await;
     //   wait_till_clear(stream).await;
    }   
}

async fn send_usernames(players: Players) {

    let mut bytes = Vec::new();
    let players_read = players.read().await;
    for (idx, player) in players_read.iter().enumerate() {
        let guard = player.lock().await;
        bytes.push(guard.name.as_bytes().to_vec());
        if idx < players_read.len()-1 {
            bytes.push(vec![';' as u8]);
        }
        
    }
    drop(players_read);
    for player in players.write().await.iter_mut() {
        bytes.push(vec![11]);
        player.lock().await.stream.write_all(&bytes.concat()).await.unwrap();
        wait_till_clear(&mut player.lock().await.stream).await;
    }   
}

async fn send_current_player(players: Players, username: &str) {
    for player in players.write().await.iter_mut() {
        let mut bytes = username.as_bytes().to_vec();
        bytes.push(10);
        player.lock().await.stream.write_all(&bytes).await.unwrap();
        wait_till_clear(&mut player.lock().await.stream).await;
    }   
}

async fn send_winner_info(players: Players, winner_name: &str) {
    for player in players.write().await.iter_mut() {
        let mut bytes = winner_name.as_bytes().to_vec();
        bytes.push(12);
        player.lock().await.stream.write_all(&bytes).await.unwrap();
        wait_till_clear(&mut player.lock().await.stream).await;
    }
}

pub async fn wait_till_clear(stream: &mut TcpStream) {
    let mut buf = [0u8; 1];
    loop {
        match stream.read(&mut buf).await {
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


async fn game_loop(players: Players) {
    let card = rand_card();
    let mut ready = false;
    
    loop {
        if ready {  
            send_usernames(players.clone()).await;
            sync_shown(players.clone(), &card).await;
            break;
        }
        ready = check_for_ready(players.clone()).await;
    }

    let mut pre_shown = card;
    
    loop {
        
        let players_guard = players.read().await;
        let a = players_guard.clone();

        drop(players_guard);
        for player in a.iter() {
            let username = player.lock().await.name.clone();
            send_current_player(players.clone(), &username).await;

            let mut abheben = true;
            player.lock().await.stream.write_all(&[3]).await.unwrap();
            while abheben {

                let mut card = [0; 8];
                player.lock().await.stream.read(&mut card).await.unwrap();
                let card = String::from_utf8_lossy(&card).to_string();
                
                let card = card.trim_end_matches(char::from(0)).to_string();
    
                if card.len() >= 5 {
                    let abheben_card = rand_card();
                    player.lock().await.cards.push(abheben_card);
                    sync_card(player.clone()).await;
                    sync_shown_p(player.clone(), &pre_shown).await;
                    println!("abheben");
                    continue;
                }
    
                if player.lock().await.cards.contains(&card) {
                    abheben = false;
                    println!("selected card: {:?}", card);   
                    
                    //sync_shown(&card).await;
                    let guard = player.lock().await;
                    for (idx, value) in guard.cards.iter().enumerate() {
                        if value == &card {
                            drop(guard);    
                            player.lock().await.cards.remove(idx);
                            break;
                        }
                    }    
                }
                sync_shown(players.clone(), &card).await;
                pre_shown = card;
                sync_card(player.clone()).await;

                if let Some(player) = get_winner(players.clone()).await {
                    let username = player.lock().await.name.clone();
                    send_winner_info(players.clone(), &username).await;
                    send_terminate(players.clone()).await;
                    return;
                }
            }
        }
        //std::thread::sleep(std::time::Duration::from_secs(1));
    }

}

async fn send_terminate(players: Players) {
    for player in players.write().await.iter_mut() {
        player.lock().await.stream.write_all(&[255]).await.unwrap();
    }
}

async fn get_winner(players: Players) -> Option<Arc<Mutex<Player>>> {
    for player in players.read().await.iter() {
        if player.lock().await.cards.len() == 0 {
            return Some(player.clone());
        }
    }
    None
}


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(IPPORT).await.unwrap();

    let players = Arc::new(RwLock::new(Vec::<Arc<Mutex<Player>>>::new()));

    let game_loop_players = players.clone();
    tokio::spawn(async move {
        loop {
            game_loop(game_loop_players.clone()).await;
            let mut players = game_loop_players.write().await;
            players.clear();
        }
        
    });

    loop {
        let players = players.clone();
        let (mut stream, _) = listener.accept().await.unwrap();

        let username = read_username(&mut stream).await;
        println!("username: {}", username);

        let player = Arc::new(Mutex::new(Player::new(stream, username)));
        players.write().await.push(player.clone());
        
        //let players1 = players.clone().lock().await[];
        tokio::spawn(async move {     
            handle_client(player).await
        }); 
    }
}

async fn read_username(stream: &mut TcpStream) -> String {
    let mut username_bytes = [0u8; 255];
    stream.read(&mut username_bytes).await.unwrap();
    let username = String::from_utf8_lossy(&username_bytes).to_string();
    username.trim_end_matches(char::from(0)).to_string()
}

async fn handle_client(player: Arc<Mutex<Player>>) {
    sync_card(player.clone()).await;

    std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
    player.lock().await.ready = true;
    
}

