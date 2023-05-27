use game_common::consts::MAX_SPEED;
use game_common::game_state::{GameState, Player}; //, next_turn_player_state};
use game_common::point::Point;

use std::f32::consts::PI;
use std::time::Instant;

use rayon::prelude::*;

use crate::precompute::{blow_player, GamePrecompute, MAX_DEPTH};

const STEP_DIRECTIONS: [i32; 7] = [16, 8, 8, 8, 4, 4, 4];
const STEP_BLOW: [i32; 7] = [0, 0, 0, 2, 3, 5, 5];
const ACC: f32 = MAX_SPEED * (MAX_DEPTH - STEP_DIRECTIONS.len()) as f32; // to make computations more precise after rounding
const SCORE_DECAY_FACTOR: f32 = 0.92;
const SPEED_SCORE_FACTOR: f32 = 0.1;
const BOUNDARY_SIZE: i32 = 500;
const NON_BOUNDARY_BONUS_FACTOR: f32 = 0.1;

fn speed_bonus(player: &Player) -> f32 {
    player.speed.len() / MAX_SPEED * SPEED_SCORE_FACTOR
}

// TODO: stay away from corner/boundary bonus
pub struct Strategy<'state> {
    game_state: &'state GameState,
    precompute: GamePrecompute<'state>,
    begin: Instant,
}

#[inline]
fn in_boundary(position: i32, size: i32) -> f32 {
    let boundary_distance: i32 = if position < BOUNDARY_SIZE {
        position
    } else if position > size - BOUNDARY_SIZE {
        size - position
    } else {
        BOUNDARY_SIZE
    };
    boundary_distance as f32 / BOUNDARY_SIZE as f32
}

impl<'state> Strategy<'state> {
    pub fn new(state: &'state GameState) -> Self {
        Self {
            begin: Instant::now(),
            precompute: GamePrecompute::new(state),
            game_state: state,
        }
    }

    fn non_boundary_bonus(&self, player: &Player) -> f32 {
        NON_BOUNDARY_BONUS_FACTOR
            * (in_boundary(player.pos.x, self.game_state.width)
                + in_boundary(player.pos.y, self.game_state.height))
    }

    pub fn best_target(&self) -> Point {
        let me = self.game_state.players[0].clone();
        let Move { score, target } =
            self.best_move(&me, vec![true; self.game_state.items.len()], 0usize);

        let speed = me.speed.len();
        let angle = normalize_angle(angle(&(target - me.pos)) - angle(&me.speed)) / PI;
        let elapsed_time = self.begin.elapsed();
        log::info!(
            "score {:.2} {}ms angle: {:.2}, speed: {:.1}",
            score,
            elapsed_time.as_millis(),
            angle,
            speed
        );
        target
    }

    fn best_move(&self, me: &Player, items: Vec<bool>, depth: usize) -> Move {
        if depth >= MAX_DEPTH {
            return Move {
                score: self.non_boundary_bonus(me),
                target: me.target,
            };
        }
        if depth >= STEP_DIRECTIONS.len() {
            let mut new_items = items.clone();
            let mut new_me = me.clone();
            blow_player(&mut new_me, 15);
            let score_inc = self.precompute.step(&mut new_me, &mut new_items, depth);
            let Move { score, target } = self.best_move(&new_me, new_items, depth + 1);
            return Move {
                score: speed_bonus(&new_me)
                    + self.non_boundary_bonus(&new_me)
                    + decay_f32(score_inc, score),
                target: target,
            };
        }

        let best_move = (0..STEP_DIRECTIONS[depth])
            .into_par_iter()
            .map(|i| {
                let angle = angle_by_index(i, STEP_DIRECTIONS[depth]) + angle(&me.speed);
                let target = me.pos
                    + Point {
                        x: (ACC * f32::sin(angle)) as i32,
                        y: (ACC * f32::cos(angle)) as i32,
                    };
                let mut player = me.clone();
                player.target = target;
                if depth < STEP_BLOW.len() {
                    blow_player(&mut player, STEP_BLOW[depth]);
                }
                let mut new_items = items.clone();

                let score_inc = self.precompute.step(&mut player, &mut new_items, depth);
                let Move { score, .. } = self.best_move(&player, new_items, depth + 1);
                return Move {
                    score: speed_bonus(&player)
                        + self.non_boundary_bonus(&player)
                        + decay_f32(score_inc, score),
                    target: target,
                };
            })
            .max_by_key(|mv| (mv.score * 1000000f32) as i64)
            .unwrap();
        best_move
    }
}

#[inline]
fn decay_f32(score_increment: f32, next_steps_score: f32) -> f32 {
    score_increment + next_steps_score * SCORE_DECAY_FACTOR
}

pub fn angle(point: &Point) -> f32 {
    (point.y as f32).atan2(point.x as f32)
}

// [0; count - 1] -> [-pi * fraction; pi * fraction]
// fn angle_by_index_semiforward(index: i32, count: i32, fraction: f32) -> f32 {
//     if count == 1 {
//         return 0f32;
//     }
//     (2 * index - count + 1) as f32 * fraction * PI / (count - 1) as f32
// }

fn angle_by_index(index: i32, count: i32) -> f32 {
    (2 * index) as f32 * PI / count as f32
}

fn normalize_angle(angle: f32) -> f32 {
    if angle < -PI {
        angle + PI
    } else if angle > PI {
        angle - PI
    } else {
        angle
    }
}

struct Move {
    score: f32,
    target: Point,
}

// #[test]
// fn angle_by_index_test() {
//     assert_eq!(angle_by_index_semiforward(0, 1, 0.6f32), 0f32);
//     assert_eq!(angle_by_index_semiforward(2, 5, 0.6f32), 0f32);
//     assert_eq!(angle_by_index_semiforward(0, 5, 0.6f32), -0.6f32 * PI);
//     assert_eq!(angle_by_index_semiforward(5 - 1, 5, 0.6f32), 0.6f32 * PI);
// }
