use std::env;
use std::{net::SocketAddr, str::FromStr};
use std::time::{Duration, Instant};

use clap::Parser;
use game_common::game_state::GameState;
use tokio::net::TcpStream;

mod connection;
mod strategy;

use crate::connection::Connection;
use crate::strategy::best_move;
use anyhow::Result;

#[derive(Parser)]
pub struct Args {
    #[clap(long)]
    addr: Option<String>,
    #[clap(long, default_value_t = 1)]
    num_bots: usize,
}

const MY_LOGIN_PREFIX: &str = "progiv-rust-";

async fn try_one_game(addr: &str, login: &str, password: &str) -> Result<()> {
    log::info!("Trying to connect to {addr}");
    let stream = TcpStream::connect(addr.clone()).await?;
    let mut conn = Connection::new(stream, SocketAddr::from_str(&addr).unwrap());

    conn.read_expect("HELLO").await?;
    conn.write("PLAY").await?;
    conn.write(format!("{login} {password}")).await?;
    let mut last_seen_turn = usize::MAX;
    loop {
        let mut state = vec![];
        loop {
            let next_token: String = conn.read().await?;
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

                let now = Instant::now();
                let my_move = best_move(&game_state);
                let elapsed_time = now.elapsed();
                log::info!("best_move calculation took {} milliseconds", elapsed_time.as_millis());

                conn.write(&format!("GO {} {}", my_move.target.x, my_move.target.y))
                    .await?;
            }
            Err(err) => {
                anyhow::bail!("Error while parsing state: {}", err);
            }
        }
    }
}

async fn one_client(addr: String) {
    let login = format!("{}{}", MY_LOGIN_PREFIX, "main");
    let password =
        env::var("AGENT_PASSWORD").expect("Failed to get AGENT_PASSWORD environment variable");
    loop {
        match try_one_game(&addr, &login, &password).await {
            Ok(()) => {}
            Err(err) => {
                log::error!("Connection finished with error: {}", err);
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting client");
    let args = Args::parse();

    let addr = args.addr.unwrap_or(format!("127.0.0.1:7877"));
    let mut handles = vec![];
    for _ in 0..args.num_bots {
        handles.push(tokio::spawn(one_client(addr.clone())));
    }
    for handle in handles {
        handle.await?;
    }
    Ok(())
}
