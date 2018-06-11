extern crate fnv;

use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::collections::HashMap;

const MAP_WIDTH: usize = 3;
const TOTAL_POSITIONS: usize = MAP_WIDTH * MAP_WIDTH;
const SEARCH_LENGTH: usize = TOTAL_POSITIONS - 2;

fn main() {
    // Prepare Hashmap
    let mut calculated_snakes = HashMap::new();
    let mut hashed_branches = FnvHashMap::default();
    let test = count_down_tree(0, 0, &mut calculated_snakes, &mut hashed_branches);
    println!("{}", test);
}

fn count_down_tree(
    tail_length: usize,
    previous_layer: u32,
    calculated_snakes: &mut HashMap<(usize, usize), FnvHashSet<u32>>,
    hashed_branches: &mut FnvHashMap<(u32, usize), u128>,
) -> u128 {
    for check in &variations(previous_layer) {
        if let Some(hashed_sum) = hashed_branches.get(&(*check, tail_length)) {
            return *hashed_sum;
        }
    }
    let branches: Vec<u32> = branches_below(previous_layer, tail_length, calculated_snakes);
    let sum = if tail_length == SEARCH_LENGTH {
        branches.len() as u128
    } else {
        branches
            .iter()
            .map(|layer| {
                count_down_tree(tail_length + 1, *layer, calculated_snakes, hashed_branches)
            })
            .sum()
    };
    hashed_branches.insert((previous_layer, tail_length), sum);
    sum
}

fn branches_below(
    previous_choises: u32,
    tail_length: usize,
    calculated_snakes: &mut HashMap<(usize, usize), FnvHashSet<u32>>,
) -> Vec<u32> {
    let mut output: [u32; TOTAL_POSITIONS] = [1; TOTAL_POSITIONS];
    let mut output_n = 0;
    let mut output_vec = Vec::new();
    loop {
        if output[output_n] == (1 << TOTAL_POSITIONS) {
            // When we've iterated up to TOTAL_POSITIONS then we've gone through all values
            // possible for that position and can increment the previous position
            if output_n == 0 {
                break;
            }
            output[output_n] = 1;
            output_n -= 1;
            output[output_n] <<= 1;
        } else if previous_choises == output[output_n]
            || output[0..output_n].contains(&output[output_n])
        {
            // We can't choose the same value as what we know was the value the last time
            // and if the last element of our output exists elsewhere in our array,
            // it's an invalid value and we need to get a new one
            output[output_n] <<= 1;
        } else if output_n <= tail_length
            && need_backup(previous_choises, output[output_n], tail_length)
            && could_block_all(
                previous_choises,
                combine_positions(&output[0..=output_n]),
                calculated_snakes,
                tail_length,
            ) {
            // Add another backup value
            let mut new_value = 1;
            while new_value != (1 << TOTAL_POSITIONS) {
                if !output[0..=output_n].contains(&new_value) {
                    output_n += 1;
                    output[output_n] = new_value;
                    break;
                }
                new_value <<= 1;
            }
        } else {
            // Our output is valid
            let out = combine_positions(&output[0..=output_n]);
            output_vec.push(out);

            output[output_n] <<= 1;
        }
    }
    output_vec
}

fn combine_positions(positions: &[u32]) -> u32 {
    positions.iter().fold(0, |acc, n| acc | n)
}

// The simplest chech for if `check_pos` may need a backup. Calculates
// if `check_pos` is `tail_length` away from any of `prev_pos_choises`
fn need_backup(prev_pos_choises: u32, check_pos: u32, tail_length: usize) -> bool {
    let check_pos = check_pos.trailing_zeros() as usize;
    let check_pos_x = check_pos % MAP_WIDTH;
    let check_pos_y = check_pos / MAP_WIDTH;
    for i in 0..TOTAL_POSITIONS {
        if (1 << i) & prev_pos_choises != 0 {
            let prev_pos_x = i % MAP_WIDTH;
            let prev_pos_y = i / MAP_WIDTH;
            if (prev_pos_x as i8 - check_pos_x as i8).abs()
                + (prev_pos_y as i8 - check_pos_y as i8).abs() <= tail_length as i8
            {
                return true;
            }
        }
    }
    false
}

fn could_block_all(
    head_positions: u32,
    chosen_positions: u32,
    calculated_snakes: &mut HashMap<(usize, usize), FnvHashSet<u32>>,
    tail_length: usize,
) -> bool {
    for head in 0..TOTAL_POSITIONS {
        if (1 << head) & head_positions != 0 {
            let possible_snakes = calculated_snakes
                .entry((head, tail_length))
                .or_insert_with(|| possible_snakes(tail_length, 1 << head as u32));
            if possible_snakes.contains(&chosen_positions) {
                return true;
            }
        }
    }
    false
}

fn possible_snakes(tail_length: usize, head: u32) -> FnvHashSet<u32> {
    let mut valid_snakes: Vec<u32> = Vec::new();

    // This will filter through all permutations of moves and keep the ones that a
    // snake could have taken without dying.
    'outer: for moves in generate_moves(tail_length) {
        let mut current_pos = head;
        let mut positions_taken = head;

        let mut first_move = true;

        // rustc believes this must be initialized here.
        let mut current_direction = 0;

        for input_direction in moves.iter().take(tail_length) {
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
        valid_snakes.push(positions_taken);
    }

    // All possible positions that a snake can block are generated to allow faster
    // lookup.
    let mut possible_blocks = FnvHashSet::default();
    for snake in valid_snakes {
        for perm in 0..(1 << TOTAL_POSITIONS) {
            possible_blocks.insert(snake & perm);
        }
    }
    possible_blocks
}

// Not neccessarily valid moves
fn generate_moves(max: usize) -> Vec<[usize; TOTAL_POSITIONS - 1]> {
    let mut all_moves: Vec<[usize; TOTAL_POSITIONS - 1]> = Vec::new();
    let mut current_move = [0; TOTAL_POSITIONS - 1];
    let mut start = true;
    'outer: loop {
        let mut i = 0;
        if start {
            start = false
        } else {
            current_move[0] += 1;
        }

        while i != 0 && current_move[i] == 3 || current_move[0] == 4 {
            current_move[i] = 0;
            i += 1;
            if i == max {
                break 'outer;
            }
            current_move[i] += 1;
        }
        all_moves.push(current_move);
    }
    all_moves
}

fn rotate_right(n: u32) -> u32 {
    let shifted_left_2 = n << 2;
    let shifted_left_4 = n << 4;
    let shifted_left_6 = n << 6;
    let shifted_right_2 = n >> 2;
    let shifted_right_4 = n >> 4;
    let shifted_right_6 = n >> 6;
    (shifted_left_6 & 0b_001_000_000)
        | (shifted_left_2 & 0b_000_001_000)
        | (shifted_right_2 & 0b_000_000_001)
        | (shifted_left_4 & 0b_010_000_000)
        | (n & 0b_000_010_000)
        | (shifted_right_4 & 0b_000_000_010)
        | (shifted_left_2 & 0b_100_000_000)
        | (shifted_right_2 & 0b_000_100_000)
        | (shifted_right_6 & 0b_000_000_100)
}

fn mirror_vertical(n: u32) -> u32 {
    let shifted_left_2 = n << 2;
    let shifted_right_2 = n >> 2;
    (shifted_left_2 & 0b_100_100_100) | (n & 0b_010_010_010) | (shifted_right_2 & 0b_001_001_001)
}

fn mirror_horizontal(n: u32) -> u32 {
    let shifted_left_6 = n << 6;
    let shifted_right_6 = n >> 6;
    (shifted_left_6 & 0b_111_000_000) | (n & 0b_000_111_000) | (shifted_right_6 & 0b_000_000_111)
}

fn mirror_diagonal_down(n: u32) -> u32 {
    let shifted_left_2 = n << 2;
    let shifted_left_4 = n << 4;
    let shifted_right_2 = n >> 2;
    let shifted_right_4 = n >> 4;
    (shifted_left_2 & 0b_010_001_000)
        | (shifted_left_4 & 0b_001_000_000)
        | (shifted_right_4 & 0b_000_000_100)
        | (shifted_right_2 & 0b_000_100_010)
        | (n & 0b_100_010_001)
}

fn mirror_diagonal_up(n: u32) -> u32 {
    let shifted_left_8 = n << 8;
    let shifted_left_4 = n << 4;
    let shifted_right_8 = n >> 8;
    let shifted_right_4 = n >> 4;
    (shifted_left_8 & 0b_100_000_000)
        | (shifted_left_4 & 0b_010_100_000)
        | (shifted_right_4 & 0b_000_001_010)
        | (shifted_right_8 & 0b_000_000_001)
        | (n & 0b_001_010_100)
}

fn variations(n: u32) -> [u32; 8] {
    let n_r1 = rotate_right(n);
    let n_r2 = rotate_right(n_r1);
    let n_r3 = rotate_right(n_r2);
    let mirrored_vertical = mirror_vertical(n);
    let mirrored_horizontal = mirror_horizontal(n);
    let mirrored_diagonal_down = mirror_diagonal_down(n);
    let mirrored_diagonal_up = mirror_diagonal_up(n);
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
    #[test]
    fn mirror_diag_down() {
        for i in 0..=0b_111_111_111 {
            assert_eq!(mirror_diagonal_down(mirror_diagonal_down(i)), i);
        }
    }
    #[test]
    fn mirror_diag_up() {
        for i in 0..=0b_111_111_111 {
            assert_eq!(mirror_diagonal_up(mirror_diagonal_up(i)), i);
        }
    }

    #[test]
    fn gen() {
        let moves = generate_moves(3);
        assert_eq!(moves.len(), 36);
    }
}
