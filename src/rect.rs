use super::Position;

// ------------------------------------------------------------------------------------------------------------------ //
#[derive(Default)]
pub struct Rect {
    pub min: Position,
    pub max: Position,
}

// ------------------------------------------------------------------------------------------------------------------ //
impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        let min = Position { x, y };
        Rect {
            min,
            max: min + Position { x: w, y: h },
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn width(&self) -> i32 {
        self.max.x - self.min.x
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn height(&self) -> i32 {
        self.max.y - self.min.y
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn contains(&self, position: &Position) -> bool {
        *position >= self.min && *position < self.max
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn intersect(&self, other: &Rect) -> bool {
        self.contains(&other.min) || self.contains(&other.max)
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn center(&self) -> (i32, i32) {
        ((self.min.x + self.max.x) / 2, (self.min.y + self.max.y) / 2)
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width()) + x) as usize
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn idx_position(&self, idx: usize) -> Position {
        let i = idx as i32;
        Position {
            x: self.min.x + (i % self.width()),
            y: self.min.y + (i / self.width()),
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ //
    pub fn print_debug(&self) {
        println!("rect: {:?} -> {:?}", self.min, self.max);
    }
}
