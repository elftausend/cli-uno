use std::sync::Arc;

use rand::Rng;
use tokio::{net::{TcpListener, TcpStream}, sync::Mutex, io::AsyncWriteExt};

const IPPORT: &str = "127.0.0.1:11000";
const MIN_USERS: usize = 1;
pub const COLOR_ARRAY: [&str; 4] = ["r", "b", "ge", "gr"];
pub const CARD_COUNT: usize = 5;

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
}

impl Player {
    pub fn new(stream: TcpStream) -> Player {
        let mut cards = Vec::new();

        for _ in 0..CARD_COUNT {            
            cards.push(rand_card());
        }
        
        Player {
            stream,
            ready: false,
            no_cards: false,
            cards, 
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
    //wait_till_clear(&mut player.stream).await;
}

type Players = Arc<Mutex<Vec<Arc<Mutex<Player>>>>>;

async fn check_for_ready(players: Players) -> bool {
    let players = players.lock().await;

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

async fn sync_shown(players: Players, card: &str) {
    for player in players.lock().await.iter_mut() {
        let mut bytes = card.as_bytes().to_vec();
        bytes.push(2);
        player.lock().await.stream.write_all(&bytes).await.unwrap();
        
     //   wait_till_clear(stream).await;
    }
    
}


async fn game_loop(players: Players) {
    let card = rand_card();
    let mut ready = false;
    
    loop {
        if ready {  
            sync_shown(players, &card).await;
            break;
        }
        ready = check_for_ready(players.clone()).await;
    }
    println!("ready!");
}


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(IPPORT).await.unwrap();

    let players = Arc::new(Mutex::new(Vec::<Arc<Mutex<Player>>>::new()));

    let game_loop_players = players.clone();
    tokio::spawn(async move {
        game_loop(game_loop_players).await
    });

    loop {
        let players = players.clone();
        let (stream, _) = listener.accept().await.unwrap();
        let player = Arc::new(Mutex::new(Player::new(stream)));
        players.lock().await.push(player.clone());
        
        //let players1 = players.clone().lock().await[];
        tokio::spawn(async move {     
            handle_client(player).await
        }); 
    }
}

async fn handle_client(player: Arc<Mutex<Player>>) {
    sync_card(player.clone()).await;

    std::thread::sleep(std::time::Duration::from_secs(2));
    player.lock().await.ready = true;
    println!("spawn");
    
}

