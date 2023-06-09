use std::env;
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::Duration;

use clap::Parser;
use game_common::game_state::GameState;

use crate::connection::Connection;
use crate::strategy::Strategy;
use anyhow::Result;

use crate::past_scores::PastScores;

mod connection;
mod past_scores;
mod precompute;
mod strategy;

#[derive(Parser)]
pub struct Args {
    #[clap(long)]
    addr: Option<String>,
}

const MY_LOGIN_PREFIX: &str = "progiv-rust-";

fn try_one_game(addr: &str, login: &str, password: &str) -> Result<()> {
    log::info!("Trying to connect to {addr}");
    let stream = TcpStream::connect(addr.clone())?;
    let mut conn = Connection::new(stream, SocketAddr::from_str(&addr).unwrap());

    conn.read_expect("HELLO")?;
    conn.write("PLAY")?;
    conn.write(format!("{login} {password}"))?;
    let mut last_seen_turn = usize::MAX;
    let mut past_scores = PastScores::new();
    loop {
        let mut state = vec![];
        loop {
            let next_token: String = conn.read()?;
            let should_end = next_token == "END_STATE";
            state.push(next_token);
            if should_end {
                break;
            }
        }
        match GameState::from_string(&state.join(" ")) {
            Ok(game_state) => {
                let turn = game_state.turn;
                if turn < last_seen_turn {
                    log::info!("New game started. Current turn: {turn}");
                }
                last_seen_turn = turn;

                let strategy = Strategy::new(&game_state);
                let my_target = strategy.best_target();
                conn.write(&format!("GO {} {}", my_target.x, my_target.y))?;

                past_scores.push(game_state.players[0].score, &game_state.game_id);
                log::info!("score: {}", past_scores);
            }
            Err(err) => {
                anyhow::bail!("Error while parsing state: {}", err);
            }
        }
    }
}

fn one_client(addr: String) {
    let login = format!("{}{}", MY_LOGIN_PREFIX, "main");
    let password =
        env::var("AGENT_PASSWORD").expect("Failed to get AGENT_PASSWORD environment variable");
    loop {
        match try_one_game(&addr, &login, &password) {
            Ok(()) => {}
            Err(err) => {
                log::error!("Connection finished with error: {}", err);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn main() {
    env_logger::init();
    log::info!("Starting client");
    let args = Args::parse();

    let addr = args.addr.unwrap_or(format!("127.0.0.1:7877"));
    one_client(addr);
}
