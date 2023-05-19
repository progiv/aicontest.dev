use game_common::consts::MAX_ACC;
use game_common::point::Point;
use game_common::{game_state::GameState, player_move::PlayerMove};

const MAX_DEPTH: usize = 15;
const FIRST_STEP_DIRECTIONS: i32 = 16;
const STEP_DIRECTIONS : [i32; 4] = [5, 5, 5, 3];
const ACC: f64 = MAX_ACC * 1000f64; // to make computations more precise after rounding
const SCORE_DECAY_FACTOR: f64 = 0.92;


fn decay(score_increment: i64, next_steps_score: f64) -> f64 {
    score_increment as f64 + next_steps_score * SCORE_DECAY_FACTOR
}

// Make a step to the defined destination and find best possible score
fn best_score(mut state: GameState, depth: usize) -> f64 {
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
        let angle = angle_by_index(i, STEP_DIRECTIONS[depth]) + angle(&me.speed);
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

fn angle(point: &Point) -> f64 {
    f64::from(point.y).atan2(f64::from(point.x))
}

fn angle_by_index(index: i32, count: i32) -> f64 {
    f64::from(index) * 2f64 * std::f64::consts::PI / f64::from(count - 1)
    // TODO try different angle sampling
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
        let angle = angle_by_index(i, FIRST_STEP_DIRECTIONS) + angle(&me.speed);
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
    move_to_go
}
