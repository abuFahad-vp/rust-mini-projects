use std::collections::VecDeque;

pub struct Snake {
    pub body: VecDeque<(i32, i32)>,
    pub direction: Direction,
    pub is_eaten: bool
}

#[derive(PartialEq)]
pub enum Direction {
    UP,
    DOWN,
    RIGHT,
    LEFT
}

impl Snake {
    pub fn new(body: VecDeque<(i32, i32)>) -> Self {
        Snake {body, direction: Direction::RIGHT, is_eaten: false}
    }

    pub fn move_snake(&mut self) {
        let new_head = self.new_head();
        if self.is_eaten {
            self.is_eaten = false;
        } else {
            self.body.pop_back();
        }
        self.body.push_front(new_head);
    } 

    pub fn new_head(&self) -> (i32, i32) {
        let (cx, cy) = self.body.front().unwrap();
        match self.direction {
            Direction::UP => { (*cx, cy - 1) },
            Direction::DOWN => { (*cx, cy + 1) },
            Direction::RIGHT => { (cx + 1, *cy) },
            Direction::LEFT => { (cx - 1, *cy) }
        }
    }

    pub fn is_block_in_body(&self, pos: (i32, i32)) -> bool {
        for bd in &self.body {
            if *bd == pos { return true }
        }
        return false
    }
}