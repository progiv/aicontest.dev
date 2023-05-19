use game_common::consts::{MAX_ACC, MAX_SPEED};
use game_common::point::Point;
use game_common::{game_state::GameState, player_move::PlayerMove};

use std::f64::consts::PI;

const MAX_DEPTH: usize = 14;
const FIRST_STEP_DIRECTIONS: i32 = 16;
const STEP_DIRECTIONS: [i32; 5] = [7, 7, 5, 5, 5];
const ACC: f64 = MAX_ACC * 1000f64; // to make computations more precise after rounding
const SCORE_DECAY_FACTOR: f64 = 0.8;
// TODO parallel first loop

fn decay(score_increment: i64, next_steps_score: f64) -> f64 {
    score_increment as f64 + next_steps_score * SCORE_DECAY_FACTOR
}

fn blow_items(state: &mut GameState, increment: i32) {
    for item in &mut state.items {
        item.radius += increment;
    }
}

fn clamp(pos: &mut i32, min_pos: i32, max_pos: i32) {
    if *pos < min_pos {
        *pos = 2 * min_pos - *pos;
    } else if *pos >= max_pos {
        *pos = 2 * max_pos - *pos;
    }
}

fn filter_state(state: &mut GameState) {
    let my_pos = state.players[0].pos;
    let radius = state.players[0].radius;

    // point in front of us
    let mut poi = my_pos + state.players[0].speed.clone().scale(MAX_SPEED * 3f64);
    clamp(&mut poi.x, radius, state.width - radius);
    clamp(&mut poi.y, radius, state.height - radius);

    // Remove players but MAX_PLAYERS closest to poi
    let mut player_ids: Vec<usize> = (1..state.players.len()).collect();
    const MAX_PLAYERS: usize = 2;
    player_ids.sort_by_key(|i| (state.players[*i].pos - poi).len2());
    player_ids[MAX_PLAYERS..].sort();
    for i in player_ids[MAX_PLAYERS..].iter().rev() {
        state.players.swap_remove(*i);
    }

    const MAX_ITEMS: usize = 32;
    state.items.sort_by_key(|it| (it.pos - poi).len2());
    state.items.truncate(MAX_ITEMS);
}

// Make a step to the defined destination and find best possible score
fn best_score(mut state: GameState, depth: usize) -> f64 {
    if depth == 0 {
        blow_items(&mut state, -1);
    } else if depth == 3 {
        blow_items(&mut state, 5);
    } else if depth == 4 {
        blow_items(&mut state, 10);
    } else if depth == 5 {
        blow_items(&mut state, 20);
    } else if depth >= MAX_DEPTH - 10 {
        blow_items(&mut state, 20)
    }

    let prev_score = state.players[0].score;
    state = state.next_turn();
    let score_increment = state.players[0].score - prev_score;

    if depth == MAX_DEPTH {
        return score_increment as f64;
    }

    if depth >= STEP_DIRECTIONS.len() {
        return decay(score_increment, best_score(state, depth + 1));
    }

    let me = &state.players[0];
    let mut score_to_go = 0f64;
    for i in 0..STEP_DIRECTIONS[depth] {
        let mut temp_state = state.clone();
        let angle = angle_by_index_semiforward(i, STEP_DIRECTIONS[depth]) + angle(&me.speed);
        let current_move = PlayerMove {
            name: me.name.clone(),
            target: Point {
                x: me.pos.x + (ACC * f64::sin(angle)) as i32,
                y: me.pos.y + (ACC * f64::cos(angle)) as i32,
            },
        };
        temp_state.apply_move(current_move.clone());
        let score = best_score(temp_state, depth + 1);
        if score > score_to_go {
            score_to_go = score;
        }
    }
    decay(score_increment, score_to_go)
}

pub fn angle(point: &Point) -> f64 {
    f64::from(point.y).atan2(f64::from(point.x))
}

// [0; count - 1] -> [0; 2 * pi)
fn angle_by_index_round(index: i32, count: i32) -> f64 {
    f64::from(index) * 2f64 * PI / f64::from(count)
}

fn angle_by_index_semiforward(index: i32, count: i32) -> f64 {
    f64::from(2 * index - count + 1) * 0.7f64 * PI / f64::from(count - 1)
}

pub fn best_move(game_state: &GameState) -> PlayerMove {
    let me = &game_state.players[0];
    let mut score_to_go = 0f64;
    let mut move_to_go = PlayerMove {
        name: me.name.clone(),
        target: Point {
            x: me.pos.x + me.speed.x * MAX_ACC as i32,
            y: me.pos.y + me.speed.y * MAX_ACC as i32,
        },
    };
    for i in 0..FIRST_STEP_DIRECTIONS {
        let mut temp_state = game_state.clone();
        filter_state(&mut temp_state);
        let angle = angle_by_index_round(i, FIRST_STEP_DIRECTIONS) + angle(&me.speed);
        let current_move = PlayerMove {
            name: me.name.clone(),
            target: Point {
                x: me.pos.x + (ACC * f64::sin(angle)) as i32,
                y: me.pos.y + (ACC * f64::cos(angle)) as i32,
            },
        };
        temp_state.apply_move(current_move.clone());
        let score = best_score(temp_state, 0);
        if score > score_to_go {
            log::debug!(
                "New best score {} for i={} ({}, {})",
                score,
                i,
                current_move.target.x - me.pos.x,
                current_move.target.y - me.pos.y
            );
            score_to_go = score;
            move_to_go = current_move;
        }
    }
    log::info!("best_score {:.2}", score_to_go);
    move_to_go
}

#[test]
fn angle_by_index_test() {
    assert_eq!(angle_by_index_round(0, 10), 0f64);
    assert_eq!(angle_by_index_round(10, 10), 2f64 * PI);
    assert_eq!(angle_by_index_round(5, 10), PI);
}
