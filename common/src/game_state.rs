use std::collections::VecDeque;
use std::str::FromStr;

use crate::consts::{MAX_ACC, MAX_SPEED};
use crate::point::Point;
use anyhow::{anyhow, bail};

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: Point,
    pub speed: Point,
    pub target: Point,
    pub score: i32,
    pub radius: i32,
    // TODO: contact info?
}

#[derive(Clone, PartialEq)] // Eq
pub struct Item {
    pub pos: Point,
    pub radius: i32,
}

impl Item {
    pub fn intersects(&self, player: &Player) -> bool {
        let dist2 = self.pos.dist2(&player.pos);
        let max_ok_dist = self.radius + player.radius;
        dist2 <= max_ok_dist * max_ok_dist
    }
}

#[derive(Clone)]
pub struct GameState {
    pub width: i32,
    pub height: i32,
    pub turn: usize,
    pub max_turns: usize,
    pub players: Vec<Player>,
    pub items: Vec<Item>,
    pub game_id: String,
}

fn clamp(pos: &mut i32, speed: &mut i32, min_pos: i32, max_pos: i32) {
    if *pos < min_pos {
        *pos = 2 * min_pos - *pos;
        *speed = -*speed;
    } else if *pos >= max_pos {
        *pos = 2 * max_pos - *pos;
        *speed = -*speed;
    }
}

struct TokenReader {
    tokens: VecDeque<String>,
}

impl TokenReader {
    pub fn new(s: &str) -> Self {
        Self {
            tokens: s.split_ascii_whitespace().map(|s| s.to_string()).collect(),
        }
    }

    pub fn next<T>(&mut self, err_msg: &str) -> anyhow::Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        Ok(self
            .tokens
            .pop_front()
            .ok_or_else(|| anyhow!(err_msg.to_owned()))?
            .parse()
            .map_err(|err| anyhow!("Failed to parse '{err_msg}': {err:?}"))?)
    }
}

const MAX_ACC_2: i32 = (MAX_ACC * MAX_ACC) as i32;
const MAX_SPEED_2: i32 = (MAX_SPEED * MAX_SPEED) as i32;

pub fn next_turn_player_state(player: &mut Player, width: i32, height: i32) {
    let mut acc = player.target - player.pos;
    if acc.len2() > MAX_ACC_2 {
        acc = acc.scale(MAX_ACC);
    }
    player.speed += acc;
    if player.speed.len2() > MAX_SPEED_2 {
        player.speed = player.speed.scale(MAX_SPEED);
    }
    player.pos += player.speed;
    clamp(
        &mut player.pos.x,
        &mut player.speed.x,
        player.radius,
        width - player.radius,
    );
    clamp(
        &mut player.pos.y,
        &mut player.speed.y,
        player.radius,
        height - player.radius,
    );
}

impl GameState {
    pub fn to_string(&self) -> String {
        let mut res = String::new();
        res += &format!(
            "TURN {turn} {max_turns} {width} {height} {game_id}\n",
            turn = self.turn,
            max_turns = self.max_turns,
            width = self.width,
            height = self.height,
            game_id = self.game_id,
        );
        res += &format!("{}\n", self.players.len());
        for player in self.players.iter() {
            res += &format!(
                "{score} {x} {y} {r} {vx} {vy} {target_x} {target_y}\n",
                score = player.score,
                x = player.pos.x,
                y = player.pos.y,
                r = player.radius,
                vx = player.speed.x,
                vy = player.speed.y,
                target_x = player.target.x,
                target_y = player.target.y,
            );
        }
        res += &format!("{}\n", self.items.len());
        for item in self.items.iter() {
            res += &format!(
                "{x} {y} {r}\n",
                x = item.pos.x,
                y = item.pos.y,
                r = item.radius
            );
        }
        res += "END_STATE\n";
        res
    }

    pub fn from_string(s: &str) -> anyhow::Result<Self> {
        let mut tokens = TokenReader::new(s);
        let cmd_word: String = tokens.next("TURN")?;
        if cmd_word != "TURN" {
            bail!("Expected TURN, got {}", cmd_word);
        }
        let turn = tokens.next("turn")?;
        let max_turns = tokens.next("max_turn")?;
        let width = tokens.next("width")?;
        let height = tokens.next("height")?;
        let game_id = tokens.next("game_id")?;
        let mut res = Self {
            width,
            height,
            turn,
            max_turns,
            players: vec![],
            items: vec![],
            game_id,
        };
        let num_players = tokens.next("num_players")?;
        for _ in 0..num_players {
            // We dont use names
            let _name: String = tokens.next("player name")?;
            let score = tokens.next("player score")?;
            let x = tokens.next("player x")?;
            let y = tokens.next("player y")?;
            let r = tokens.next("player r")?;
            let vx = tokens.next("player vx")?;
            let vy = tokens.next("player vy")?;
            let target_x = tokens.next("player target_x")?;
            let target_y = tokens.next("player target_y")?;
            res.players.push(Player {
                //name,
                score,
                pos: Point { x, y },
                speed: Point { x: vx, y: vy },
                target: Point {
                    x: target_x,
                    y: target_y,
                },
                radius: r,
            });
        }
        let num_items = tokens.next("num items")?;
        for _ in 0..num_items {
            let x = tokens.next("item x")?;
            let y = tokens.next("item y")?;
            let r: i32 = tokens.next("item r")?;
            res.items.push(Item {
                pos: Point { x, y },
                radius: r,
            });
        }
        let end_state: String = tokens.next("END_STATE")?;
        if end_state != "END_STATE" {
            bail!("Expected END_STATE, got {}", end_state);
        }
        Ok(res)
    }
}

#[test]
fn next_turn_state() {
    let mut player = Player {
        pos: Point { x: 100, y: 100 },
        speed: Point { x: 10, y: 0 },
        target: Point { x: 150, y: 200 }, // sent by `GO 150 200` command
        score: 0,
        radius: 1,
    };
    next_turn_player_state(&mut player, 1000, 1000);
    // acceleration direction is (150, 200) - (100, 100) = (50, 100)
    // the length of vector (50, 100) is sqrt(50^2 + 100^2) = 111.8, which is bigger than MAX_ACC=20.0, so real acceleration is:
    // (50, 100) * 20.0 / 111.8 = (8.9, 17.8)
    // after that acceleration is rounded to integers: (9, 18)
    // new speed is (10, 0) + (9, 18) = (19, 18)

    assert_eq!(player.speed, Point { x: 19, y: 18 });
    // assert_float_absolute_eq!(player.speed.x, 19f32, 0.5);
    // assert_float_absolute_eq!(player.speed.y, 18f32, 0.5);
    // new position is (100, 100) + (19, 18) = (119, 118)
    assert_eq!(player.pos, Point { x: 119, y: 118 });
    // assert_float_absolute_eq!(player.pos.x, 119f32, 0.5);
    // assert_float_absolute_eq!(player.pos.y, 118f32, 0.5);
}
