extern crate fnv;

use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::collections::HashMap;

const MAP_WIDTH: usize = 3;
const SEARCH_LENGTH: usize = 6;

#[derive(Clone, Copy)]
enum Moves {
    Up,
    Right,
    Down,
    Left,
}

fn branches_below(
    main_positions: u16,
    previous_choises: u16,
    tail_length: usize,
    snakes_calculated: &HashMap<(usize, usize), FnvHashSet<u16>>,
) -> Vec<u16> {
    let mut output: [u16; 9] = [1, 1, 1, 1, 1, 1, 1, 1, 1];
    let mut output_n = 0;
    let mut output_vec = Vec::new();
    loop {
        if output_n == 0 && main_positions & output[0] == 0 {
            output[0] <<= 1;
            if output[0] >= (1 << 9) {
                break;
            }
        } else if output[output_n] >= (1 << 9) {
            // When we've iterated up to MAP_WIDTH*MAP_WIDTH then we've gone through all values
            // needed for that positions and can increment the previous position
            output[output_n] = 1;
            output_n -= 1;
            output[output_n] <<= 1;
            if output[0] == (1 << 9) {
                break;
            }
        } else if (
                // We can't choose the same value as what we know was the value the last time
                previous_choises == output[output_n]
            ) ||
                // If the last element of our output exists elsewhere in our array,
                // it's an invalid value and we need to get a new one
                 {
                    let (last, rest) = output[0..=output_n].split_last().unwrap();
                    let taken_positions = combine_positions(rest);
                    taken_positions & last != 0
                } {
            output[output_n] <<= 1;
        } else if output_n + 1 < (tail_length + 2)
            && need_backup(previous_choises, output[output_n], tail_length)
            && could_block_all(
                previous_choises,
                combine_positions(&output[0..=output_n]),
                &snakes_calculated,
                tail_length,
            ) {
            // Add another backup value if we need it
            let mut new_value = 1;
            while new_value != (1 << 9) {
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
// The simplest chech for if `check_pos` may need a backup. Calculates
// if `check_pos` is `tail_length` away from any of `prev_pos_choises`
fn need_backup(prev_pos_choises: u16, check_pos: u16, tail_length: usize) -> bool {
    let check_pos = check_pos.trailing_zeros() as usize;
    let check_pos_x = check_pos % MAP_WIDTH;
    let check_pos_y = check_pos / MAP_WIDTH;
    for i in 0..9 {
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
    head_positions: u16,
    chosen_positions: u16,
    snakes_calculated: &HashMap<(usize, usize), FnvHashSet<u16>>,
    tail_length: usize,
) -> bool {
    for head in 0..9 {
        if (1 << head) & head_positions != 0 {
            let possible_snakes = match snakes_calculated.get(&(head, tail_length)) {
                Some(x) => x,
                None => panic!("WRONG"),
            };
            if possible_snakes.contains(&chosen_positions) {
                return true;
            }
        }
    }
    false
}

// Not neccessirarily valid moves
fn generate_moves(max: usize) -> Vec<[usize; MAP_WIDTH * MAP_WIDTH - 1]> {
    let mut all_moves: Vec<[usize; MAP_WIDTH * MAP_WIDTH - 1]> = Vec::new();
    let mut current_move = [0; (MAP_WIDTH * MAP_WIDTH) - 1];
    let mut start = true;
    'outer: loop {
        let mut i = 0;
        if start {
            start = false
        } else {
            current_move[i] += 1;
        }

        while (i == 0 && current_move[0] == 4) || (i != 0 && current_move[i] == 3) {
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

fn get_valid_snakes(tail_length: usize, head: usize) -> FnvHashSet<u16> {
    let mut move_container: Vec<Vec<usize>> = Vec::with_capacity(tail_length + 1);
    let mut positions_taken: Vec<usize> = Vec::with_capacity(tail_length + 1);
    let head_x = head % MAP_WIDTH;
    let head_y = head / MAP_WIDTH;
    let all_moves = generate_moves(tail_length);
    'outer: for moves in all_moves {
        let mut current_x = head_x;
        let mut current_y = head_y;
        positions_taken.clear();
        positions_taken.push(head);

        let mut non_relative_moves = Vec::with_capacity(tail_length);

        // Handle the first move differently as it can move in four directions
        let first_move = match moves[0] {
            0 => Moves::Up,
            1 => Moves::Right,
            2 => Moves::Down,
            3 => Moves::Left,
            _ => unreachable!(),
        };
        let mut last_move = first_move;
        non_relative_moves.push(first_move);

        for direction in moves.iter().take(tail_length).skip(1) {
            let chosen_move = match (last_move, *direction) {
                (Moves::Up, 1) | (Moves::Right, 0) | (Moves::Left, 2) => Moves::Up,
                (Moves::Up, 2) | (Moves::Right, 1) | (Moves::Down, 0) => Moves::Right,
                (Moves::Right, 2) | (Moves::Down, 1) | (Moves::Left, 0) => Moves::Down,
                (Moves::Up, 0) | (Moves::Down, 2) | (Moves::Left, 1) => Moves::Left,
                _ => unreachable!(),
            };
            last_move = chosen_move;
            non_relative_moves.push(chosen_move);
        }

        for direction in non_relative_moves {
            match direction {
                Moves::Up => {
                    if current_y == 0 {
                        continue 'outer;
                    }
                    current_y -= 1;
                }
                Moves::Right => {
                    if current_x == MAP_WIDTH - 1 {
                        continue 'outer;
                    }
                    current_x += 1;
                }
                Moves::Down => {
                    if current_y == MAP_WIDTH - 1 {
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
            let n = current_y * MAP_WIDTH + current_x;
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
        let out = positions_to_u16(&snake);
        insert_permutations_for_u16(out, &mut possible_blocks);
    }
    possible_blocks
}

fn insert_permutations_for_u16(list: u16, possible_blocks: &mut FnvHashSet<u16>) {
    possible_blocks.insert(list);
    for perm in 0..=0b_1_1111_1111 {
        possible_blocks.insert(list & perm);
    }
}

fn prepare_hashmap() -> HashMap<(usize, usize), FnvHashSet<u16>> {
    let mut snakes_calculated = HashMap::new();
    for o in 0..9 {
        for p in 1..8 {
            snakes_calculated
                .entry((o, p))
                .or_insert_with(|| get_valid_snakes(p, o));
        }
    }
    snakes_calculated
}
fn main() {
    // Prepare Hashmap
    let snakes_calculated = prepare_hashmap();
    don(snakes_calculated);
}

fn don(snakes_calculated: HashMap<(usize, usize), FnvHashSet<u16>>) {
    println!("Done generating");
    let mut hashed_branches = FnvHashMap::default();
    let corners = count_down_tree(1, 1 << 0, &snakes_calculated, &mut hashed_branches);
    println!("{}", corners);
    let side = count_down_tree(1, 1 << 1, &snakes_calculated, &mut hashed_branches);
    println!("{}", side);
    let middle = count_down_tree(1, 1 << 4, &snakes_calculated, &mut hashed_branches);
    println!("{}", middle);
    let final_value = 4 * corners + 4 * side + middle;
    println!("{}", final_value);
}

fn count_down_tree(
    tail_length: usize,
    previous_layer: u16,
    snakes_calculated: &HashMap<(usize, usize), FnvHashSet<u16>>,
    hashed_branches: &mut FnvHashMap<(u16, usize), u128>,
) -> u128 {
    let previous_layer_u16 = previous_layer;
    match hashed_branches.get(&(previous_layer_u16, tail_length)) {
        Some(value) => return *value,
        None => {}
    }
    let symmetricity = alternative_symmetricity(previous_layer_u16);
    let groups: &[u16] = match symmetricity {
        Symmetry::Horizontal => &[0b_0_0000_0111, 0b_0_0011_1000],
        Symmetry::Vertical => &[0b_0_0100_1001, 0b_0_1001_0010],
        Symmetry::Full => &[0b_0_0000_0001, 0b_0_0000_0010, 0b_0_0001_0000],
        Symmetry::Plus => &[0b_0_0000_0001, 0b_0_0000_0010, 0b_0_0000_1000, 0b_0_0001_0000],
        Symmetry::DiagonalCrossing => &[0b_0_0000_0001, 0b_0_0000_0100, 0b_0_0000_1000, 0b_0_0001_0000],
        Symmetry::DiagonalDown => &[0b_0_0010_0110,  0b_1_0001_0001],
        Symmetry::DiagonalUp => &[0b_0_0000_1011, 0b_0_0101_0100],
        Symmetry::None => &[0b_1_1111_1111],
    };
    let sums = generate_sums_of_branches(
        groups,
        tail_length,
        snakes_calculated,
        hashed_branches,
        previous_layer,
    );
    let total_sum = match symmetricity {
        Symmetry::Horizontal => 2 * sums[0] + sums[1],
        Symmetry::Vertical => 2 * sums[0] + sums[1],
        Symmetry::Full => 4 * sums[0] + 4 * sums[1] + sums[2],
        Symmetry::Plus => 4 * sums[0] + 2 * (sums[1] + sums[2]) + sums[3],
        Symmetry::DiagonalCrossing => 2 * (sums[0] + sums[1]) + 4 * sums[2] + sums[3],
        Symmetry::DiagonalDown => 2 * sums[0] + sums[1],
        Symmetry::DiagonalUp => 2 * sums[0] + sums[1],
        Symmetry::None => sums[0],
    };
    hashed_branches.insert((previous_layer_u16, tail_length), total_sum);
    total_sum
}

fn generate_sums_of_branches(
    groups: &[u16],
    tail_length: usize,
    snakes_calculated: &HashMap<(usize, usize), FnvHashSet<u16>>,
    hashed_branches: &mut FnvHashMap<(u16, usize), u128>,
    previous_layer: u16,
) -> Vec<u128> {
    let mut group_sums = Vec::new();
    for group in groups {
        let branches: Vec<u16> =
            branches_below(*group, previous_layer, tail_length, snakes_calculated);
        let mut sum = 0;
        if tail_length == SEARCH_LENGTH {
            sum = branches.len() as u128;
        } else {
            for layer in branches {
                sum += count_down_tree(tail_length + 1, layer, snakes_calculated, hashed_branches);
            }
        }
        group_sums.push(sum)
    }
    group_sums
}

// 0 1 2
// 3 4 5
// 6 7 8
enum Symmetry {
    // a b a
    // b c b
    // a b a
    Full,

    // a b c
    // d e f
    // a b c
    Horizontal,

    // a d a
    // b e b
    // c f c
    Vertical,

    // a b a
    // c d c
    // a b a
    Plus,

    // d a b
    // a e c
    // b c f
    DiagonalDown,

    // a b d
    // c e c
    // f b a
    DiagonalUp,

    // a b c
    // b d b
    // c b a
    DiagonalCrossing,

    // a b c
    // d e f
    // g h i
    None,
}

fn positions_to_u16(positions: &[usize]) -> u16 {
    let mut out: u16 = 0;
    for i in positions {
        out += 2u16.pow(*i as u32);
    }
    out
}

fn combine_positions(positions: &[u16]) -> u16 {
    positions.iter().fold(0, |acc, n| acc | n)
}

fn alternative_symmetricity(points_n: u16) -> Symmetry {
    let shifted_six = points_n >> 6 & points_n;
    let horizontal = (shifted_six & 0b0000_0111) == 0b0000_0111;
    let shifted_two = points_n >> 2 & points_n;
    let vertical = (shifted_two & 0b0100_1001) == 0b0100_1001;
    let shifted_four = points_n >> 4 & points_n;
    let diagonal_down =
        (shifted_two & 0b0010_0010) == 0b0010_0010 && (shifted_four & 0b0000_0100) == 0b0000_0100;
    let shifted_eight = points_n >> 8 & points_n;
    let diagonal_up = (shifted_eight & 0b0000_0001) == 0b0000_0001
        && (shifted_four & 0b0000_0010) == 0b0000_0010
        && (shifted_four & 0b0000_1000) == 0b0000_1000;
    if horizontal && vertical && diagonal_down && diagonal_up {
        Symmetry::Full
    } else if horizontal && vertical {
        Symmetry::Plus
    } else if vertical {
        Symmetry::Vertical
    } else if horizontal {
        Symmetry::Horizontal
    } else if diagonal_down && diagonal_up {
        Symmetry::DiagonalCrossing
    } else if diagonal_up {
        Symmetry::DiagonalUp
    } else if diagonal_down {
        Symmetry::DiagonalDown
    } else {
        Symmetry::None
    }
}
