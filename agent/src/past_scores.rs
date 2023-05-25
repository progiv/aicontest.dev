const MAX_SCORES_SIZE: usize = 10;

pub struct PastScores {
    scores: Vec<i32>,
    last_game_id: String,
}

impl PastScores {
    pub fn new() -> Self {
        PastScores {
            scores: vec![],
            last_game_id: "NULL0".to_string(),
        }
    }

    pub fn push(&mut self, item: i32, game_id: &String) {
        if self.last_game_id == *game_id {
            // update last score
            if self.scores.len() == 0 {
                panic!("Lost game score update")
            }
            self.scores.pop();
            self.scores.push(item);
        } else {
            self.last_game_id = game_id.clone();
            if self.scores.len() == MAX_SCORES_SIZE {
                self.scores.remove(0);
            }
            self.scores.push(item);
        }
    }
}

impl std::fmt::Display for PastScores {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.scores)
    }
}
