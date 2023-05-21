use game_common::consts::{MAX_ACC, MAX_SPEED};
use game_common::point::Point;
use game_common::{game_state::GameState};

use std::f32::consts::PI;
use std::time::Instant;

use rayon::prelude::*;

const MAX_DEPTH: usize = 14;
const OTHER_PLAYERS_REMOVE_DEPTH: usize = 7;
const FIRST_STEP_DIRECTIONS: i32 = 15;
const STEP_DIRECTIONS: [i32; 6] = [7, 5, 5, 5, 5, 5];
const ACC: f32 = MAX_ACC * 1000f32; // to make computations more precise after rounding
const SCORE_DECAY_FACTOR: f32 = 0.85;
const SPEED_SCORE_FACTOR: f32 = 0.05;
// TODO Profiler report:
// next_turn 74%
// among them next_turn_player_state 32%

fn decay(score_increment: i32, next_steps_score: f32) -> f32 {
    score_increment as f32 + next_steps_score * SCORE_DECAY_FACTOR
}

fn blow_items(state: &mut GameState, increment: f32) {
    for item in &mut state.items {
        item.radius += increment;
    }
}

fn blow_players(state: &mut GameState, increment: f32) {
    for player in &mut state.players[1..] {
        player.radius += increment;
    }
}

fn clamp(pos: &mut f32, min_pos: f32, max_pos: f32) {
    if *pos < min_pos {
        *pos = 2f32 * min_pos - *pos;
    } else if *pos >= max_pos {
        *pos = 2f32 * max_pos - *pos;
    }
}

fn filter_state(state: &mut GameState) {
    let my_pos = state.players[0].pos;
    let radius = state.players[0].radius;

    // point in front of us
    let mut poi = my_pos + state.players[0].speed.clone().scale(MAX_SPEED * 3f32);
    clamp(&mut poi.x, radius, state.width - radius);
    clamp(&mut poi.y, radius, state.height - radius);

    // Remove players but MAX_PLAYERS closest to poi
    let mut player_ids: Vec<usize> = (1..state.players.len()).collect();
    const MAX_PLAYERS: usize = 3;
    player_ids.sort_by_key(|i| (state.players[*i].pos - poi).len2() as i64);
    player_ids[MAX_PLAYERS..].sort();
    for i in player_ids[MAX_PLAYERS..].iter().rev() {
        state.players.swap_remove(*i);
    }

    const MAX_ITEMS: usize = 24;
    state.items.sort_by_key(|it| (it.pos - poi).len2() as i64);
    state.items.truncate(MAX_ITEMS);
}

fn remove_others(state: &mut GameState) {
    state.players.truncate(1)
}

fn speed_score(speed: &Point) -> f32 {
    speed.len() * SPEED_SCORE_FACTOR / MAX_SPEED
}

// Make a step to the defined destination and find best possible score
fn best_score(mut state: GameState, depth: usize) -> f32 {
    if state.turn > state.max_turns {
        return 0f32;
    }
    if depth > OTHER_PLAYERS_REMOVE_DEPTH {
        remove_others(&mut state);
    }
    if depth == 0 {
        blow_items(&mut state, -2.);
    } else if depth == 3 {
        blow_items(&mut state, 5.);
        blow_players(&mut state, 2.);
    } else if depth == 4 {
        blow_items(&mut state, 5.);
        blow_players(&mut state, 3.);
    } else if depth == 5 {
        blow_items(&mut state, 5.);
        blow_players(&mut state, 3.);
    } else if depth >= MAX_DEPTH - 10 {
        blow_items(&mut state, 15.)
    }

    let prev_score = state.players[0].score;
    state = state.next_turn();
    let score_increment = state.players[0].score - prev_score;

    if depth == MAX_DEPTH {
        return score_increment as f32;
    }

    if depth >= STEP_DIRECTIONS.len() {
        return decay(
            score_increment,
            speed_score(&state.players[0].speed) + best_score(state, depth + 1),
        );
    }

    let me = &state.players[0];
    let mut score_to_go = 0f32;
    for i in 0..STEP_DIRECTIONS[depth] {
        let mut temp_state = state.clone();
        let angle = angle_by_index_semiforward(i, STEP_DIRECTIONS[depth], 0.9) + angle(&me.speed);
        temp_state.apply_my_target(Point {
                x: me.pos.x + ACC * f32::sin(angle),
                y: me.pos.y + ACC * f32::cos(angle),
            });
        let score = speed_score(&temp_state.players[0].speed) + best_score(temp_state, depth + 1);
        if score > score_to_go {
            score_to_go = score;
        }
    }
    decay(score_increment, score_to_go)
}

pub fn angle(point: &Point) -> f32 {
    point.y.atan2(point.x)
}

// [0; count - 1] -> [-pi * fraction; pi * fraction]
fn angle_by_index_semiforward(index: i32, count: i32, fraction: f32) -> f32 {
    if count == 1 {
        return 0f32;
    }
    (2 * index - count + 1) as f32 * fraction * PI / (count - 1) as f32
}

struct Move {
    score: f32,
    target: Point,
}

pub fn best_target(game_state: &GameState) -> Point {
    let now = Instant::now();
    let me = &game_state.players[0];
    let best_move = (0..FIRST_STEP_DIRECTIONS)
        .into_par_iter()
        .map(|i| {
            let mut temp_state = game_state.clone();
            filter_state(&mut temp_state);
            let angle =
                angle_by_index_semiforward(i, FIRST_STEP_DIRECTIONS, 0.9) + angle(&me.speed);
            let target = Point {
                    x: me.pos.x + ACC * angle.sin(),
                    y: me.pos.y + ACC * angle.cos(),
                };
            temp_state.apply_my_target(target);
            let score = best_score(temp_state, 0);
            Move {
                score: score,
                target: target,
            }
        })
        .max_by_key(|mv| (mv.score * 1000000f32) as i64)
        .unwrap();
    let elapsed_time = now.elapsed();
    let speed = me.speed.len();
    let angle = (angle(&(best_move.target - me.pos)) - angle(&me.speed)) * 180f32 / PI;
    log::info!(
        "score {:.2} ts {}ms, angle: {}, speed: {:.1}",
        best_move.score,
        elapsed_time.as_millis(),
        angle.round(),
        speed
    );

    best_move.target
}

#[test]
fn angle_by_index_test() {
    assert_eq!(angle_by_index_semiforward(0, 1, 0.6f32), 0f32);
    assert_eq!(angle_by_index_semiforward(2, 5, 0.6f32), 0f32);
    assert_eq!(angle_by_index_semiforward(0, 5, 0.6f32), -0.6f32 * PI);
    assert_eq!(angle_by_index_semiforward(5 - 1, 5, 0.6f32), 0.6f32 * PI);
}
