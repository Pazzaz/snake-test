extern crate fnv;

use fnv::FnvHashSet;
use std::collections::HashMap;

struct PositionFinder {
    output: Vec<u8>,
    previous_choises: Vec<u8>,
    tail_length: usize,
    snakes_calculated: HashMap<u8, FnvHashSet<[bool; 16]>>,
    done: bool,
}

enum Moves {
    Up,
    Right,
    Down,
    Left,
}

impl PositionFinder {
    fn new(previous_choises: Vec<u8>, tail_length: usize) -> PositionFinder {
        PositionFinder {
            output: vec![0],
            previous_choises,
            tail_length,
            snakes_calculated: HashMap::new(),
            done: false,
        }
    }
}

impl Iterator for PositionFinder {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        // We don't need to do anything if we're done
        if self.done {
            return None;
        }
        loop {
            // When we've iterated up to 16 then we've gone through all values
            // needed for that positions and can increment the previous position
            if *self.output.last().unwrap() >= 16 {
                self.output.pop();
                *self.output.last_mut().unwrap() += 1;
                if self.output[0] == 16 {
                    self.done = true;
                    return None;
                }

            // If the last element of our output exists elsewhere in our array,
            // it's an invalid value and we need to get a new one
            } else if {
                let (last, rest) = self.output.split_last().unwrap();
                rest.contains(last)
            } {
                *self.output.last_mut().unwrap() += 1;

            // Add another backup value if we need it
            } else if self.output.len() < (self.tail_length + 2)
                && need_backup(
                    &self.previous_choises,
                    *self.output.last().unwrap(),
                    self.tail_length,
                )
                && could_block_all(
                    &self.previous_choises,
                    &self.output,
                    &mut self.snakes_calculated,
                    self.tail_length,
                ) {
                for backup in 0..16 {
                    if !self.output.contains(&backup) {
                        self.output.push(backup);
                        break;
                    }
                }

            // Our output is valid
            } else {
                let out = self.output.clone();
                *self.output.last_mut().unwrap() += 1;
                if self.output[0] == 16 {
                    self.done = true;
                }
                return Some(out);
            }
        }
    }
}

#[inline]
fn need_backup(prev_pos_choises: &[u8], check_pos: u8, tail_length: usize) -> bool {
    let check_pos_x = check_pos % 4;
    let check_pos_y = check_pos / 4;
    prev_pos_choises.iter().any(|choise| {
        let prev_pos_x = choise % 4;
        let prev_pos_y = choise / 4;
        if (prev_pos_x as i8 - check_pos_x as i8).abs()
            + (prev_pos_y as i8 - check_pos_y as i8).abs() <= tail_length as i8
        {
            return true;
        }
        false
    })
}

fn could_block_all(
    head_positions: &[u8],
    chosen_positions: &[u8],
    snakes_calculated: &mut HashMap<u8, FnvHashSet<[bool; 16]>>,
    tail_length: usize,
) -> bool {
    'outer_for_loop: for head in head_positions {
        let possible_snakes = snakes_calculated
            .entry(*head)
            .or_insert_with(|| get_valid_snakes(tail_length, *head));
        let mut simple_format = [false; 16];
        for pos in chosen_positions {
            simple_format[*pos as usize] = true;
        }
        if possible_snakes.contains(&simple_format) {
            return true;
        }
    }
    false
}

fn get_valid_snakes(tail_length: usize, head: u8) -> FnvHashSet<[bool; 16]> {
    let mut move_container: Vec<Vec<u8>> = Vec::with_capacity(tail_length + 1);
    let mut positions_taken: Vec<u8> = Vec::with_capacity(tail_length + 1);
    let head_x = head % 4;
    let head_y = head / 4;
    let mut moves: [u8; 15] = [0; 15];
    'outer: loop {
        let mut current_x = head_x;
        let mut current_y = head_y;
        let mut i = 0;
        moves[i] += 1;
        while (i == 0 && moves[0] == 4) || (i != 0 && moves[i] == 3) {
            moves[i] = 0;
            i += 1;
            if i == tail_length {
                break 'outer;
            }
            moves[i] += 1;
        }
        positions_taken.clear();
        positions_taken.push(head);

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

        for direction in moves.iter().take(tail_length).skip(1) {
            let chosen_move = match (last_move, *direction) {
                (0, 1) | (1, 0) | (3, 2) => Moves::Up,
                (0, 2) | (1, 1) | (2, 0) => Moves::Right,
                (1, 2) | (2, 1) | (3, 0) => Moves::Down,
                (0, 0) | (2, 2) | (3, 1) => Moves::Left,
                _ => unreachable!(),
            };
            last_move = *direction;
            match chosen_move {
                Moves::Up => {
                    if current_y == 0 {
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
        positions_taken.sort();
        move_container.push(positions_taken.clone());
    }
    let mut possible_blocks = FnvHashSet::default();
    for snake in move_container {
        let mut out = [false; 16];
        for square in snake {
            out[square as usize] = true;
        }
        let out_with_perm = generate_permutations(out);
        for perm in out_with_perm {
            possible_blocks.insert(perm);
        }
    }
    possible_blocks
}

fn generate_permutations(mut list: [bool; 16]) -> Vec<[bool; 16]> {
    let original_list = list;
    let mut out: Vec<[bool; 16]> = Vec::new();
    out.push(list);
    let ones = list.iter().filter(|&x| *x).count();
    let mut moves: Vec<u8> = vec![0; ones];
    'outer: loop {
        let mut i = 0;
        moves[i] += 1;
        while moves[i] == 2 {
            moves[i] = 0;
            i += 1;
            if i == moves.len() {
                break 'outer;
            }
            moves[i] += 1;
        }
        let mut moves_iter = moves.iter();
        for (index, item) in original_list.iter().enumerate() {
            list[index] = *item && (*moves_iter.next().unwrap() == 1)
        }
        out.push(list);
    }
    out
}

fn main() {
    let mut f0: u128 = 0;
    for _ in PositionFinder::new(vec![3, 10], 6) {
        f0 += 1;
    }
    println!("{}", f0);
}
// 3813084
// 28648268
// 253887624
// 2206143912
// 17605721016
