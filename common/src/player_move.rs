use crate::point::Point;

#[derive(Clone, Debug)]
pub struct PlayerMove {
    pub name: String,
    pub target: Point,
}
