use game_common::game_state::next_turn_player_state;
use game_common::game_state::Item;

use game_common::game_state::{GameState, Player};

// World
// 1. Constant, shared:
// depth -> Item -> score_left
// Item -> pos

// 2. Mutable, cloned
// Index -> [item_availability]

pub const MAX_DEPTH: usize = 16usize;

pub struct GamePrecompute {
    pub width: i32,
    pub height: i32,
    pub turn: usize,
    pub items: Vec<Item>,
    pub turn_item_score: Vec<Vec<f32>>,
    pub max_turns: usize,
}

impl GamePrecompute {
    pub fn new(orig_state: &GameState) -> Self {
        let mut state = orig_state.clone();
        state.players.swap_remove(0); // Player Me is not for precompute

        let mut turn_item_eaten: Vec<Vec<f32>> = vec![];
        for _step in 0..MAX_DEPTH {
            let mut item_eaten: Vec<f32> = vec![0.0; state.items.len()];
            for player in state.players.iter_mut() {
                next_turn_player_state(player, state.width, state.height);
            }
            for item_id in 0..state.items.len() {
                for player in state.players.iter_mut() {
                    if state.items[item_id].intersects(&player) {
                        item_eaten[item_id] += 1.0;
                    }
                }
            }
            turn_item_eaten.push(item_eaten);
        }

        let mut turn_item_score: Vec<Vec<f32>> = vec![vec![1f32; state.items.len()]; MAX_DEPTH];
        for step in 0..MAX_DEPTH {
            for i in 0..state.items.len() {
                turn_item_score[step][i] = if step == 0 {
                    1.0 / (1f32 + turn_item_eaten[step][i])
                } else if turn_item_eaten[step - 1][i] > 0f32 {
                    // it was probably eaten before, but who knows?
                    turn_item_score[step - 1][i] / 10.0
                } else {
                    turn_item_score[step - 1][i] / (1f32 + turn_item_eaten[step][i])
                };
            }
        }

        Self {
            width: orig_state.width,
            height: orig_state.height,
            turn: orig_state.turn,
            items: orig_state.items.clone(),
            turn_item_score: turn_item_score,
            max_turns: orig_state.max_turns,
        }
    }

    pub fn step(&self, mut me: &mut Player, items: &mut Vec<bool>, depth: usize) -> f32 {
        let mut score: f32 = 0.0;
        next_turn_player_state(&mut me, self.width, self.height);
        for i in 0..items.len() {
            if items[i] && self.items[i].intersects(&me) {
                score += self.turn_item_score[depth][i];
                items[i] = false;
            }
        }
        score
    }
}
