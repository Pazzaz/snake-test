#![feature(slice_patterns)]
extern crate num_bigint;
extern crate num_traits;
extern crate pathfinding;

use pathfinding::astar::astar;
use num_bigint::BigUint;
use num_traits::{One, Zero};

struct PositionFinder {
    output: Vec<i8>,
    previous_choises: Vec<i8>,
    tail_length: i8,
    done: bool,
}

enum Moves {
    Up,
    Right,
    Down,
    Left,
}

impl PositionFinder {
    fn new(previous_choises: Vec<i8>, tail_length: i8) -> PositionFinder {
        PositionFinder {
            output: vec![-1],
            previous_choises: previous_choises,
            tail_length: tail_length,
            done: false,
        }
    }
}

impl Iterator for PositionFinder {
    type Item = Vec<i8>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.done {
                return None;
            }
            *self.output.last_mut().unwrap() += 1;
            if self.output[0] == 16 {
                self.done = true;
                return None;
            } else if self.output[0..(self.output.len() - 1)].contains(self.output.last().unwrap())
            {
            } else if *self.output.last().unwrap() >= 16 {
                self.output.pop();
            } else if self.output.len() < (self.tail_length as usize + 2)
                && need_backup(
                    &self.previous_choises,
                    *self.output.last().unwrap(),
                    self.tail_length,
                )
                && could_block_all(&self.previous_choises, &self.output, self.tail_length)
            {
                for backup in 0..16 {
                    if !self.output.contains(&backup) {
                        self.output.push(backup - 1);
                        break;
                    }
                }
            } else {
                return Some(self.output.clone());
            }
        }
    }
}

fn need_backup(prev_pos_choises: &Vec<i8>, check_pos: i8, tail_length: i8) -> bool {
    let check_pos_x = check_pos % 4;
    let check_pos_y = check_pos / 4;
    for choise in prev_pos_choises {
        let prev_pos_x = choise % 4;
        let prev_pos_y = choise / 4;
        if (prev_pos_x - check_pos_x).abs() + (prev_pos_y - check_pos_y).abs() <= tail_length {
            return true;
        }
    }
    false
}

fn could_block_all(head_positions: &Vec<i8>, chosen_positions: &Vec<i8>, tail_length: i8) -> bool {
    for head in head_positions {
        let head_x = head % 4;
        let head_y = head / 4;
        let mut moves: Vec<i8> = vec![0; tail_length as usize];
        'outer: loop {
            let mut current_x = head_x;
            let mut current_y = head_y;
            let mut i = 0;
            moves[i] += 1;
            while (i == 0 && moves[0] == 4) || (i != 0 && moves[i] == 3) {
                moves[i] = 0;
                i += 1;
                if i == tail_length as usize {
                    break 'outer;
                }
                moves[i] += 1;
            }
            let mut positions_taken: Vec<i8> = Vec::with_capacity(tail_length as usize + 1);
            positions_taken.push(*head);

            // Handle the first move differently as it can move in four directions
            let mut last_move;
            match moves[0] {
                0 => {
                    // UP
                    if current_y == 0 {
                        continue 'outer;
                    }
                    current_y -= 1;
                    last_move = 0;
                }
                1 => {
                    // RIGHT
                    if current_x == 3 {
                        continue 'outer;
                    }
                    current_x += 1;
                    last_move = 1;
                }
                2 => {
                    // DOWN
                    if current_y == 3 {
                        continue 'outer;
                    }
                    current_y += 1;
                    last_move = 2;
                }
                3 => {
                    // LEFT
                    if current_x == 0 {
                        continue 'outer;
                    }
                    current_x -= 1;
                    last_move = 3;
                }
                _ => unreachable!(),
            }
            positions_taken.push(current_y * 4 + current_x);
            for direction in moves.iter().skip(1) {
                let chosen_move;
                chosen_move = match (last_move, *direction) {
                    (0, 0) => Moves::Left,
                    (0, 1) => Moves::Up,
                    (0, 2) => Moves::Right,
                    (1, 0) => Moves::Up,
                    (1, 1) => Moves::Right,
                    (1, 2) => Moves::Down,
                    (2, 0) => Moves::Right,
                    (2, 1) => Moves::Down,
                    (2, 2) => Moves::Left,
                    (3, 0) => Moves::Down,
                    (3, 1) => Moves::Left,
                    (3, 2) => Moves::Up,
                    _ => unreachable!(),
                };
                last_move = *direction;
                match chosen_move {
                    Moves::Up => {
                        if current_x == 0 {
                            continue 'outer;
                        }
                        current_y -= 1;
                    }
                    Moves::Right => {
                        if current_x == 3 {
                            continue 'outer;
                        }
                        current_x += 1;
                    }
                    Moves::Down => {
                        if current_y == 3 {
                            continue 'outer;
                        }
                        current_y += 1;
                    }
                    Moves::Left => {
                        if current_x == 0 {
                            continue 'outer;
                        }
                        current_x -= 1;
                    }
                }
                let n = current_y * 4 + current_x;
                if positions_taken.contains(&n) {
                    continue 'outer;
                }
                positions_taken.push(n)
            }
            let mut all_blocked = true;
            for chosen in chosen_positions {
                if !positions_taken.contains(chosen) {
                    all_blocked = false;
                    break;
                }
            }
            if all_blocked {
                return true;
            }
        }
    }
    false
}

fn main() {
    let mut f0: BigUint = Zero::zero();
    let f1: BigUint = One::one();
    for _ in PositionFinder::new(vec![3, 10], 6) {
        f0 += &f1;
    }
    println!("{}", f0);
}