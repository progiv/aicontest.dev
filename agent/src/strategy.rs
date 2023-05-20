use game_common::consts::{MAX_ACC, MAX_SPEED};
use game_common::point::Point;
use game_common::{game_state::GameState, player_move::PlayerMove};

use std::f64::consts::PI;
use std::time::Instant;

use rayon::prelude::*;

const MAX_DEPTH: usize = 14;
const OTHER_PLAYERS_REMOVE_DEPTH: usize = 7;
const FIRST_STEP_DIRECTIONS: i32 = 15;
const STEP_DIRECTIONS: [i32; 6] = [7, 5, 5, 5, 5, 5];
const ACC: f64 = MAX_ACC * 1000f64; // to make computations more precise after rounding
const SCORE_DECAY_FACTOR: f64 = 0.85;
const SPEED_SCORE_FACTOR: f64 = 0.05;
// TODO overflows: 32 -> 64
// TODO Profiler report:
// next_turn 74%
// among them next_turn_player_state 32%

fn decay(score_increment: i64, next_steps_score: f64) -> f64 {
    score_increment as f64 + next_steps_score * SCORE_DECAY_FACTOR
}

fn blow_items(state: &mut GameState, increment: i32) {
    for item in &mut state.items {
        item.radius += increment;
    }
}

fn blow_players(state: &mut GameState, increment: i32) {
    for player in &mut state.players[1..] {
        player.radius += increment;
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
    const MAX_PLAYERS: usize = 3;
    player_ids.sort_by_key(|i| (state.players[*i].pos - poi).len2());
    player_ids[MAX_PLAYERS..].sort();
    for i in player_ids[MAX_PLAYERS..].iter().rev() {
        state.players.swap_remove(*i);
    }

    const MAX_ITEMS: usize = 24;
    state.items.sort_by_key(|it| (it.pos - poi).len2());
    state.items.truncate(MAX_ITEMS);
}

fn remove_others(state: &mut GameState) {
    state.players.truncate(1)
}

fn speed_score(speed: &Point) -> f64 {
    speed.len() * SPEED_SCORE_FACTOR / MAX_SPEED
}

// Make a step to the defined destination and find best possible score
fn best_score(mut state: GameState, depth: usize) -> f64 {
    if state.turn > state.max_turns {
        return 0f64;
    }
    if depth > OTHER_PLAYERS_REMOVE_DEPTH {
        remove_others(&mut state);
    }
    if depth == 0 {
        blow_items(&mut state, -2);
    } else if depth == 3 {
        blow_items(&mut state, 5);
        blow_players(&mut state, 2);
    } else if depth == 4 {
        blow_items(&mut state, 5);
        blow_players(&mut state, 3);
    } else if depth == 5 {
        blow_items(&mut state, 5);
        blow_players(&mut state, 3);
    } else if depth >= MAX_DEPTH - 10 {
        blow_items(&mut state, 15)
    }

    let prev_score = state.players[0].score;
    state = state.next_turn();
    let score_increment = state.players[0].score - prev_score;

    if depth == MAX_DEPTH {
        return score_increment as f64;
    }

    if depth >= STEP_DIRECTIONS.len() {
        return decay(
            score_increment,
            speed_score(&state.players[0].speed) + best_score(state, depth + 1),
        );
    }

    let me = &state.players[0];
    let mut score_to_go = 0f64;
    for i in 0..STEP_DIRECTIONS[depth] {
        let mut temp_state = state.clone();
        let angle = angle_by_index_semiforward(i, STEP_DIRECTIONS[depth], 0.9) + angle(&me.speed);
        let current_move = PlayerMove {
            name: me.name.clone(),
            target: Point {
                x: me.pos.x + (ACC * f64::sin(angle)) as i32,
                y: me.pos.y + (ACC * f64::cos(angle)) as i32,
            },
        };
        temp_state.apply_move(current_move.clone());
        let score = speed_score(&temp_state.players[0].speed) + best_score(temp_state, depth + 1);
        if score > score_to_go {
            score_to_go = score;
        }
    }
    decay(score_increment, score_to_go)
}

pub fn angle(point: &Point) -> f64 {
    f64::from(point.y).atan2(f64::from(point.x))
}

// [0; count - 1] -> [-pi * fraction; pi * fraction]
fn angle_by_index_semiforward(index: i32, count: i32, fraction: f64) -> f64 {
    if count == 1 {
        return 0f64;
    }
    f64::from(2 * index - count + 1) * fraction * PI / f64::from(count - 1)
}

struct Move {
    score: f64,
    player_move: PlayerMove,
}

pub fn best_move(game_state: &GameState) -> PlayerMove {
    let now = Instant::now();
    let me = &game_state.players[0];
    let best_move = (0..FIRST_STEP_DIRECTIONS)
        .into_par_iter()
        .map(|i| {
            let mut temp_state = game_state.clone();
            filter_state(&mut temp_state);
            let angle =
                angle_by_index_semiforward(i, FIRST_STEP_DIRECTIONS, 0.9) + angle(&me.speed);
            let current_move = PlayerMove {
                name: me.name.clone(),
                target: Point {
                    x: me.pos.x + (ACC * f64::sin(angle)) as i32,
                    y: me.pos.y + (ACC * f64::cos(angle)) as i32,
                },
            };
            temp_state.apply_move(current_move.clone());
            let score = best_score(temp_state, 0);
            Move {
                score: score,
                player_move: current_move,
            }
        })
        .max_by_key(|mv| (mv.score * 1000000f64) as i64)
        .unwrap();
    let elapsed_time = now.elapsed();
    let speed = me.speed.len();
    let angle = (angle(&(best_move.player_move.target - me.pos)) - angle(&me.speed)) * 180f64
        / std::f64::consts::PI;
    log::info!(
        "score {:.2} ts {} ms, angle: {}, speed: {:.1}",
        best_move.score,
        elapsed_time.as_millis(),
        angle.round(),
        speed
    );

    best_move.player_move
}

#[test]
fn angle_by_index_test() {
    assert_eq!(angle_by_index_semiforward(0, 1, 0.6f64), 0f64);
    assert_eq!(angle_by_index_semiforward(2, 5, 0.6f64), 0f64);
    assert_eq!(angle_by_index_semiforward(0, 5, 0.6f64), -0.6f64 * PI);
    assert_eq!(angle_by_index_semiforward(5 - 1, 5, 0.6f64), 0.6f64 * PI);
}
