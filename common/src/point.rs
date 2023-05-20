#[derive(Copy, Clone, Debug, PartialEq)] // Eq
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Point { x: 0f32, y: 0f32 };

    pub fn len2(&self) -> f32 {
        let x = self.x;
        let y = self.y;
        x * x + y * y
    }
    pub fn len(&self) -> f32 {
        self.len2().sqrt()
    }

    pub fn scale(&self, target_len: f32) -> Self {
        if self.x == 0f32 && self.y == 0f32 {
            return *self;
        }
        let mult = target_len / self.len();
        Point {
            x: self.x * mult,
            y: self.y * mult,
        }
    }

    pub fn dist2(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
