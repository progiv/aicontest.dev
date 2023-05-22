use game_common::consts::{MAX_ACC, MAX_SPEED};
use game_common::game_state::{GameState, Player};
use game_common::point::Point;

use std::f32::consts::PI;
use std::time::Instant;

// use rayon::prelude::*;

use crate::precompute::GamePrecompute;
use crate::precompute::MAX_DEPTH;

// const OTHER_PLAYERS_REMOVE_DEPTH: usize = 7;
// const FIRST_STEP_DIRECTIONS: i32 = 15;
const STEP_DIRECTIONS: [i32; 6] = [15, 9, 5, 5, 5, 5];
const ACC: f32 = MAX_ACC * 1000f32; // to make computations more precise after rounding
const SCORE_DECAY_FACTOR: f32 = 0.85;
// const SPEED_SCORE_FACTOR: f32 = 0.05;
// TODO Profiler report:
// next_turn 74%
// among them next_turn_player_state 32%

fn decay_f32(score_increment: f32, next_steps_score: f32) -> f32 {
    score_increment + next_steps_score * SCORE_DECAY_FACTOR
}

fn clamp(pos: &mut f32, min_pos: f32, max_pos: f32) {
    if *pos < min_pos {
        *pos = 2f32 * min_pos - *pos;
    } else if *pos >= max_pos {
        *pos = 2f32 * max_pos - *pos;
    }
}

pub fn filter_state(state: &mut GameState) {
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

    const MAX_ITEMS: usize = 30;
    state.items.sort_by_key(|it| (it.pos - poi).len2() as i64);
    state.items.truncate(MAX_ITEMS);
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

// TODO:
// 1. speed bonus
// 2. item shrink (1st step) and increase(later, optional)
// 3. Competitors blow up on precompute.
// 4. Move score decay to precompute step
// 5. intersects takes more than 55% of CPU cycles
fn best_move(me: Player, precompute: &GamePrecompute, items: Vec<bool>, depth: usize) -> Move {
    if depth >= MAX_DEPTH {
        return Move {
            score: 0.0,
            target: me.target,
        };
    } else if depth >= STEP_DIRECTIONS.len() {
        let (score_inc, new_me, new_items) = precompute.step(me, &items, depth);
        let Move { score, target } = best_move(new_me, precompute, new_items, depth + 1);
        return Move {
            score: decay_f32(score_inc, score),
            target: target,
        };
    }

    let best_move = (0..STEP_DIRECTIONS[depth])
        // .into_par_iter()
        .map(|i| {
            let angle =
                angle_by_index_semiforward(i, STEP_DIRECTIONS[depth], 0.9) + angle(&me.speed);
            let target = Point {
                x: me.pos.x + ACC * f32::sin(angle),
                y: me.pos.y + ACC * f32::cos(angle),
            };
            let mut player = me.clone();
            player.target = target.clone();
            let (score_inc, player, new_items) = precompute.step(player, &items, depth);
            let Move { score, .. } = best_move(player, precompute, new_items, depth + 1);
            return Move {
                score: decay_f32(score_inc, score),
                target: target,
            };
        })
        .max_by_key(|mv| (mv.score * 1000000f32) as i64)
        .unwrap();
    best_move
}

pub fn best_target(state: &GameState) -> Point {
    let now = Instant::now();

    let me = &state.players[0];
    let precompute = GamePrecompute::new(state);
    let Move { score, target } = best_move(
        state.players[0].clone(),
        &precompute,
        vec![true; state.items.len()],
        0usize,
    );

    let speed = me.speed.len();
    let angle = (angle(&(target - me.pos)) - angle(&me.speed)) * 180f32 / PI;
    let elapsed_time = now.elapsed();
    log::info!(
        "score {:.2} ts {}ms, angle: {}, speed: {:.1}",
        score,
        elapsed_time.as_millis(),
        angle.round(),
        speed,
    );
    target
}

#[test]
fn angle_by_index_test() {
    assert_eq!(angle_by_index_semiforward(0, 1, 0.6f32), 0f32);
    assert_eq!(angle_by_index_semiforward(2, 5, 0.6f32), 0f32);
    assert_eq!(angle_by_index_semiforward(0, 5, 0.6f32), -0.6f32 * PI);
    assert_eq!(angle_by_index_semiforward(5 - 1, 5, 0.6f32), 0.6f32 * PI);
}
