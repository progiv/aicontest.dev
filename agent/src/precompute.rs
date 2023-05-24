use game_common::game_state::next_turn_player_state;
use game_common::game_state::Item;

use game_common::game_state::{GameState, Player};

use game_common::consts::MAX_ITEM_R;

// World
// 1. Constant, shared:
// depth -> Item -> score_left
// Item -> pos

// 2. Mutable, cloned
// Index -> [item_availability]

pub const MAX_DEPTH: usize = 16usize;

pub struct GamePrecompute<'state> {
    pub width: i32,
    pub height: i32,
    pub turn: usize,
    pub items: &'state Vec<Item>,
    pub turn_item_score: Vec<Vec<f32>>,
    pub max_turns: usize,
    item_index: ItemIndex<'state>,
}

impl<'state> GamePrecompute<'state> {
    pub fn new(orig_state: &'state GameState) -> Self {
        let mut state = orig_state.clone();
        state.players.swap_remove(0); // Player Me is not for precompute
        let item_index = ItemIndex::new(&orig_state.items, orig_state.width, orig_state.height);

        let mut turn_item_eaten: Vec<Vec<f32>> = vec![vec![0f32; state.items.len()]; MAX_DEPTH];
        for step in 0..MAX_DEPTH {
            for player in state.players.iter_mut() {
                next_turn_player_state(player, state.width, state.height);
            }
            for player in state.players.iter() {
                for item_id in item_index.intersections(player) {
                    turn_item_eaten[step][item_id] += 1f32;
                }
            }
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
            items: &orig_state.items,
            turn_item_score: turn_item_score,
            max_turns: orig_state.max_turns,
            item_index: item_index,
        }
    }

    pub fn step(&self, mut me: &mut Player, items: &mut Vec<bool>, depth: usize) -> f32 {
        let mut score: f32 = 0.0;
        next_turn_player_state(&mut me, self.width, self.height);
        for i in self.item_index.intersections(&me) {
            if items[i] {
                score += self.turn_item_score[depth][i];
                items[i] = false;
            }
        }
        score
    }
}

const INDEX_GRID_SIZE: i32 = 200;

struct ItemIndex<'items> {
    // width: i32,
    // height: i32,
    g_width: usize,
    g_height: usize,
    items: &'items Vec<Item>,
    index: Vec<Vec<Vec<usize>>>,
}

#[inline]
fn grid_size(size: i32) -> usize {
    ((size + INDEX_GRID_SIZE - 1) / INDEX_GRID_SIZE) as usize
}

#[inline]
fn clamp1(input: i32, max: usize) -> usize {
    if input < 0 {
        0usize
    } else if input >= max as i32 {
        max - 1
    } else {
        input as usize
    }
}

#[inline]
fn row_index(x: i32) -> i32 {
    x / INDEX_GRID_SIZE
}

impl<'items> ItemIndex<'items> {
    pub fn new(items: &'items Vec<Item>, width: i32, height: i32) -> Self {
        let g_width = grid_size(width);
        let g_height = grid_size(height);
        let mut index: Vec<Vec<Vec<usize>>> = vec![vec![vec![]; g_height]; g_width];
        for i in 0..items.len() {
            index[row_index(items[i].pos.x) as usize][row_index(items[i].pos.y) as usize].push(i);
        }
        Self {
            // width: width,
            // height: height,
            g_width: g_width,
            g_height: g_height,
            items: items,
            index: index,
        }
    }

    fn index_safe(&self, x: i32, y: i32) -> (usize, usize) {
        (
            clamp1(row_index(x), self.g_width),
            clamp1(row_index(y), self.g_height),
        )
    }

    pub fn intersections(&self, player: &Player) -> Vec<usize> {
        let max_distance = player.radius + MAX_ITEM_R;
        let (x_min, y_min) =
            self.index_safe(player.pos.x - max_distance, player.pos.y - max_distance);
        let (x_max, y_max) =
            self.index_safe(player.pos.x + max_distance, player.pos.y + max_distance);
        let mut intersections: Vec<usize> = vec![];
        for x in x_min..=x_max {
            for y in y_min..=y_max {
                for item_idx in self.index[x][y].iter() {
                    if self.items[*item_idx].intersects(player) {
                        intersections.push(*item_idx);
                    }
                }
            }
        }
        intersections
    }
}
