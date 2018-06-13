#![feature(test)]
#![feature(nll)]
extern crate fnv;

extern crate test;

use fnv::FnvHashMap;
use std::time::Instant;

const MAP_WIDTH: usize = 3;
const TOTAL_POSITIONS: usize = MAP_WIDTH * MAP_WIDTH;
const SEARCH_LENGTH: usize = 7;

fn main() {
    // Prepare Hashmap
    let start = Instant::now();
    let calculated_snakes = prepare_snakes();
    let mut hashed_branches = FnvHashMap::default();
    println!("GENERATED HASHMAP");
    let test = count_down_tree(0, 0, &calculated_snakes, &mut hashed_branches);
    println!("{:?}", Instant::now().duration_since(start));
    println!("{}", test);
}

fn prepare_snakes() -> Vec<Vec<[bool; 0b1_0000_0000_0000_0000]>> {
    let mut out: Vec<Vec<[bool; 0b1_0000_0000_0000_0000]>> = Vec::with_capacity(TOTAL_POSITIONS);
    let mut snakes;
    for snake_head_position in 0..(TOTAL_POSITIONS as u8) {
        out.push(Vec::with_capacity(SEARCH_LENGTH));
        for tail_length in 1..=(SEARCH_LENGTH) {
            snakes = possible_snakes(tail_length, 1 << snake_head_position);
            out[snake_head_position as usize].push(snakes);
        }
    }
    out
}

fn count_down_tree(
    tail_length: usize,
    previous_layer: u32,
    calculated_snakes: &[Vec<[bool; 0b1_0000_0000_0000_0000]>],
    hashed_branches: &mut FnvHashMap<(u16, u8), u128>,
) -> u128 {
    match hashed_branches.get(&(previous_layer as u16, tail_length as u8)) {
        Some(hashed_sum) => *hashed_sum,
        None => {
            let iter = BranchIterator::new(previous_layer, tail_length, calculated_snakes);
            let mut sum;
            if tail_length == SEARCH_LENGTH {
                sum = iter.count() as u128
            } else {
                sum = 0;
                for layer in iter {
                    sum +=
                        count_down_tree(tail_length + 1, layer, calculated_snakes, hashed_branches)
                }
            };
            for variant in &variations(previous_layer) {
                hashed_branches.insert((*variant as u16, tail_length as u8), sum);
            }
            sum
        }
    }
}

struct BranchIterator<'a> {
    previous_choises: u32,
    tail_length: usize,
    calculated_snakes: &'a [Vec<[bool; 0b1_0000_0000_0000_0000]>],
    output: [u32; TOTAL_POSITIONS],
    output_n: usize,
    cached_combinations: [u32; TOTAL_POSITIONS],
}

impl<'a> BranchIterator<'a> {
    fn new(
        previous_choises: u32,
        tail_length: usize,
        calculated_snakes: &'a [Vec<[bool; 0b1_0000_0000_0000_0000]>],
    ) -> BranchIterator<'a> {
        let mut output_array = [0; TOTAL_POSITIONS];
        output_array[0] = 1;
        BranchIterator {
            previous_choises,
            tail_length,
            calculated_snakes,
            output: output_array,
            output_n: 0,
            cached_combinations: [0; TOTAL_POSITIONS],
        }
    }
}

impl<'a> Iterator for BranchIterator<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.output[self.output_n] == (1 << TOTAL_POSITIONS) {
                // When we've iterated up to TOTAL_POSITIONS then we've gone through all values
                // possible for that position and can increment the previous position
                if self.output_n == 0 {
                    return None;
                }
                self.output[self.output_n] = 0;
                self.output_n -= 1;
                self.output[self.output_n] <<= 1;
                continue;
            }
            if self.previous_choises == self.output[self.output_n]
                || self.cached_combinations[self.output_n] & self.output[self.output_n] != 0
            {
                // We can't choose the same value as what we know was the value the last time
                // and if the last element of our output exists elsewhere in our array,
                // it's an invalid value and we need to get a new one
                self.output[self.output_n] <<= 1;
                continue;
            }
            let combination = combine_positions(&self.output);
            if self.output_n <= self.tail_length
                && could_block_all(
                    self.previous_choises,
                    combination,
                    self.calculated_snakes,
                    self.tail_length,
                ) {
                // Add another backup value
                let mut new_value = 1;
                loop {
                    if combination & new_value == 0 {
                        self.output_n += 1;
                        self.output[self.output_n] = new_value;
                        break;
                    }
                    new_value <<= 1;
                }
                self.cached_combinations[self.output_n] = combination;
            } else {
                // Our output is valid
                self.output[self.output_n] <<= 1;
                return Some(combination);
            }
        }
    }
}

fn combine_positions(positions: &[u32; TOTAL_POSITIONS]) -> u32 {
    let mut combination = 0;
    for i in positions {
        combination |= i;
    }
    combination
}

fn could_block_all(
    previous_choises: u32,
    chosen_positions: u32,
    calculated_snakes: &[Vec<[bool; 0b1_0000_0000_0000_0000]>],
    tail_length: usize,
) -> bool {
    for head in 0..(TOTAL_POSITIONS as u8) {
        if (1 << head) & previous_choises != 0
            && calculated_snakes[head as usize][tail_length - 1][chosen_positions as usize]
        {
            return true;
        }
    }
    false
}

fn possible_snakes(tail_length: usize, head: u32) -> [bool; 0b1_0000_0000_0000_0000] {
    let mut possible_blocks: [bool; 0b1_0000_0000_0000_0000] = [false; 0b1_0000_0000_0000_0000];

    assert!(tail_length < TOTAL_POSITIONS - 1);
    // This will filter through all permutations of moves and keep the ones that a
    // snake could have taken without dying.
    let mut moves = [0; TOTAL_POSITIONS - 1];
    moves[0] = -1;
    'outer: loop {
        let mut i = 0;
        moves[0] += 1;

        while i != 0 && moves[i] == 3 || moves[0] == 4 {
            moves[i] = 0;
            i += 1;
            if i == tail_length {
                break 'outer;
            }
            moves[i] += 1;
        }

        let mut current_pos = head;
        let mut positions_taken = head;
        let mut first_move = true;

        // rustc believes this must be initialized here.
        let mut current_direction: i32 = 0;

        for input_direction in &moves[0..tail_length] {
            current_direction = if first_move {
                first_move = false;
                // The first input_direction can be any direction 0..=3
                *input_direction
            } else {
                // We will enter here on every iteration of the for loop except the first one.
                // The following `input_direction`s will be given relative to our current
                // direction so they will have a value 0..=2
                match (current_direction, input_direction) {
                    (0, 1) | (1, 0) | (3, 2) => 0,
                    (0, 2) | (1, 1) | (2, 0) => 1,
                    (1, 2) | (2, 1) | (3, 0) => 2,
                    (0, 0) | (2, 2) | (3, 1) => 3,
                    _ => unreachable!(),
                }
            };
            match current_direction {
                0 => {
                    if current_pos < 1 << MAP_WIDTH {
                        continue 'outer;
                    }
                    current_pos >>= MAP_WIDTH;
                }
                1 => {
                    if current_pos.trailing_zeros() as usize % MAP_WIDTH == MAP_WIDTH - 1 {
                        continue 'outer;
                    }
                    current_pos <<= 1;
                }
                2 => {
                    if current_pos >= 1 << (MAP_WIDTH * (MAP_WIDTH - 1)) {
                        continue 'outer;
                    }
                    current_pos <<= MAP_WIDTH;
                }
                3 => {
                    if current_pos.trailing_zeros() as usize % MAP_WIDTH == 0 {
                        continue 'outer;
                    }
                    current_pos >>= 1;
                }
                _ => unreachable!(),
            }
            // We cant occupy a position twice
            if positions_taken & current_pos != 0 {
                continue 'outer;
            }

            positions_taken |= current_pos;
        }
        // All possible positions that a snake can block are generated to allow faster
        // lookup.
        for perm in 0..(1 << TOTAL_POSITIONS) {
            possible_blocks[positions_taken as usize & perm] = true;
        }
    }
    possible_blocks
}

fn rotate_right(n: u32) -> u32 {
    if TOTAL_POSITIONS == 9 {
        (n << 6 & 0b_001_000_000)
            | (n << 2 & 0b_100_001_000)
            | (n >> 2 & 0b_000_100_001)
            | (n << 4 & 0b_010_000_000)
            | (n & 0b_000_010_000)
            | (n >> 4 & 0b_000_000_010)
            | (n >> 6 & 0b_000_000_100)
    } else if TOTAL_POSITIONS == 16 {
        (n << 12 & 0b_0001_0000_0000_0000)
            | (n << 7 & 0b_0000_0001_0000_0000)
            | (n << 2 & 0b_0000_0000_0001_0000)
            | (n >> 3 & 0b_0000_0000_0000_0001)
            | (n << 9 & 0b_0010_0000_0000_0000)
            | (n << 4 & 0b_0000_0010_0000_0000)
            | (n >> 1 & 0b_0000_0000_0010_0000)
            | (n >> 6 & 0b_0000_0000_0000_0010)
            | (n << 6 & 0b_0100_0000_0000_0000)
            | (n << 1 & 0b_0000_0100_0000_0000)
            | (n >> 4 & 0b_0000_0000_0100_0000)
            | (n >> 9 & 0b_0000_0000_0000_0100)
            | (n << 3 & 0b_1000_0000_0000_0000)
            | (n >> 2 & 0b_0000_1000_0000_0000)
            | (n >> 7 & 0b_0000_0000_1000_0000)
            | (n >> 12 & 0b_0000_0000_0000_1000)
    } else {
        unimplemented!();
    }
}

fn mirror_vertical(n: u32) -> u32 {
    if TOTAL_POSITIONS == 9 {
        (n << 2 & 0b_100_100_100) | (n & 0b_010_010_010) | (n >> 2 & 0b_001_001_001)
    } else if TOTAL_POSITIONS == 16 {
        (n << 3 & 0b_1000_1000_1000_1000)
            | (n << 1 & 0b_0100_0100_0100_0100)
            | (n >> 1 & 0b_0010_0010_0010_0010)
            | (n >> 3 & 0b_0001_0001_0001_0001)
    } else {
        unimplemented!();
    }
}

fn mirror_horizontal(n: u32) -> u32 {
    if TOTAL_POSITIONS == 9 {
        (n << 6 & 0b_111_000_000) | (n & 0b_000_111_000) | (n >> 6 & 0b_000_000_111)
    } else if TOTAL_POSITIONS == 16 {
        (n << 12 & 0b_1111_0000_0000_0000)
            | (n << 4 & 0b_0000_1111_0000_0000)
            | (n >> 4 & 0b_0000_0000_1111_0000)
            | (n >> 12 & 0b_0000_0000_0000_1111)
    } else {
        unimplemented!();
    }
}

fn variations(n: u32) -> [u32; 8] {
    let n_r1 = rotate_right(n);
    let n_r2 = rotate_right(n_r1);
    let n_r3 = rotate_right(n_r2);
    let mirrored_vertical = mirror_vertical(n);
    let mirrored_horizontal = mirror_horizontal(n);
    let mirrored_diagonal_down = mirror_horizontal(n_r1);
    let mirrored_diagonal_up = mirror_vertical(n_r1);
    [
        n,
        n_r1,
        n_r2,
        n_r3,
        mirrored_vertical,
        mirrored_horizontal,
        mirrored_diagonal_down,
        mirrored_diagonal_up,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    #[test]
    fn rotation_right() {
        for i in 0..=0b_111_111_111 {
            assert_eq!(rotate_right(rotate_right(rotate_right(rotate_right(i)))), i);
        }
    }
    #[test]
    fn mirror_vert() {
        for i in 0..=0b_111_111_111 {
            assert_eq!(mirror_vertical(mirror_vertical(i)), i);
        }
    }
    #[test]
    fn mirror_hori() {
        for i in 0..=0b_111_111_111 {
            assert_eq!(mirror_horizontal(mirror_horizontal(i)), i);
        }
    }

    #[bench]
    fn snakes(b: &mut Bencher) {
        b.iter(|| possible_snakes(1, 3));
    }
    #[bench]
    fn rot(b: &mut Bencher) {
        b.iter(|| {
            for i in 0..(1 << TOTAL_POSITIONS) {
                let a = rotate_right(i);
                test::black_box(a);
            }
        });
    }
}
