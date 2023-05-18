use game_common::consts::MAX_ACC;
use game_common::point::Point;
use game_common::{game_state::GameState, game_state::NextTurn, player_move::PlayerMove};

// todo: score decay

const MAX_DEPTH: i32 = 10;
const LINEAR_DEPTH: i32 = 2;
const FIRST_STEP_DIRECTIONS: i32 = 360;
const ACC: f64 = MAX_ACC * 1000f64; // to make computations more precise after rounding

// Make a step to the defined destination and find best possible score
fn best_score(state: GameState, depth: i32) -> i64 {
    if depth == MAX_DEPTH {
        return state.players[0].score;
    }

    if depth >= LINEAR_DEPTH {
        // No target branching to reduce complexity
        match state.next_turn() {
            NextTurn::GameState(next_state) => {
                return next_state.players[0].score;
            }
            NextTurn::FinalResults(results) => {
                return results.players[0].score;
            }
        }
    }

    // branch at the beginning of the movement
    // no branching for mvp
    match state.next_turn() {
        NextTurn::GameState(next_state) => {
            return best_score(next_state, depth + 1);
            // return next_state.players[0].score + best_score(next_state, depth + 1);
        }
        NextTurn::FinalResults(results) => {
            return results.players[0].score;
        }
    }
}

fn angle(point: &Point) -> f64 {
    f64::from(point.y).atan2(f64::from(point.x))
}

fn angle_by_index(index: i32, count: i32) -> f64 {
    f64::from(index) * 2f64 * std::f64::consts::PI / f64::from(count)
}

pub fn best_move(game_state: &GameState) -> PlayerMove {
    let me = &game_state.players[0];
    let mut score_to_go = 0;
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
    log::info!(
        "target direction: {} {}",
        move_to_go.target.x - me.pos.x,
        move_to_go.target.y - me.pos.y
    );
    move_to_go
}
