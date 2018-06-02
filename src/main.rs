extern crate fnv;

use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::collections::HashMap;

const MAP_WIDTH: usize = 3;
const TOTAL_POSITIONS: usize = MAP_WIDTH * MAP_WIDTH;
const SEARCH_LENGTH: usize = TOTAL_POSITIONS - 2;

fn main() {
    // Prepare Hashmap
    let calculated_snakes = prepare_hashmap();
    let mut hashed_branches = FnvHashMap::default();
    let test = count_down_tree(0, 0, &calculated_snakes, &mut hashed_branches);
    println!("{}", test);
}

// Calculates the branch-sum for `previous_layer` by recursively traversing the
// tree. This function will be called multiple times as such:
//                           |
//                    count_down_tree
//                           |
//              generate_sums_of_branches - ...
//              /            |            \
//  count_down_tree  count_down_tree   count_down_tree
//         /                 |                     \
//       ...      generate_sums_of_branches - ...   \
//                    /      |      \               ...
//                  ...     ...     ...
fn count_down_tree(
    tail_length: usize,
    previous_layer: u16,
    calculated_snakes: &HashMap<(usize, usize), FnvHashSet<u16>>,
    hashed_branches: &mut FnvHashMap<(u16, usize), u128>,
) -> u128 {
    if let Some(hashed_sum) = hashed_branches.get(&(previous_layer, tail_length)) {
        return *hashed_sum;
    }
    let symmetricity = symmetricity(previous_layer);

    // These groups represent position-branches which will need their sums
    // calculated. Not all branches need their sums calculated as symmetry in the
    // previous layer causes some position-branches to have the same values.
    let mut groups: &[u16] = match symmetricity {
        Symmetry::Horizontal => &[0b_0_0000_0111, 0b_0_0011_1000],
        Symmetry::Vertical => &[0b_0_0100_1001, 0b_0_1001_0010],
        Symmetry::Full => &[0b_0_0000_0001, 0b_0_0000_0010, 0b_0_0001_0000],
        Symmetry::Plus => &[
            0b_0_0000_0001,
            0b_0_0000_0010,
            0b_0_0000_1000,
            0b_0_0001_0000,
        ],
        Symmetry::DiagonalCrossing => &[
            0b_0_0000_0001,
            0b_0_0000_0100,
            0b_0_0000_1000,
            0b_0_0001_0000,
        ],
        Symmetry::DiagonalDown => &[0b_0_0010_0110, 0b_1_0001_0001],
        Symmetry::DiagonalUp => &[0b_0_0000_1011, 0b_0_0101_0100],
        Symmetry::None => &[0b_1_1111_1111],
    };
    if TOTAL_POSITIONS == 4 {
        groups = &[0b_1111]
    }
    let sums = generate_sums_of_branches(
        groups,
        tail_length,
        calculated_snakes,
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
    hashed_branches.insert((previous_layer, tail_length), total_sum);
    total_sum
}

fn generate_sums_of_branches(
    groups: &[u16],
    tail_length: usize,
    calculated_snakes: &HashMap<(usize, usize), FnvHashSet<u16>>,
    hashed_branches: &mut FnvHashMap<(u16, usize), u128>,
    previous_layer: u16,
) -> Vec<u128> {
    let mut group_sums = Vec::new();
    for group in groups {
        let branches: Vec<u16> =
            branches_below(*group, previous_layer, tail_length, calculated_snakes);
        let mut sum = 0;
        if tail_length == SEARCH_LENGTH {
            sum = branches.len() as u128;
        } else {
            for layer in branches {
                sum += count_down_tree(tail_length + 1, layer, calculated_snakes, hashed_branches);
            }
        }
        group_sums.push(sum)
    }
    group_sums
}

fn branches_below(
    main_positions: u16,
    previous_choises: u16,
    tail_length: usize,
    calculated_snakes: &HashMap<(usize, usize), FnvHashSet<u16>>,
) -> Vec<u16> {
    let mut output: [u16; TOTAL_POSITIONS] = [1; TOTAL_POSITIONS];
    let mut output_n = 0;
    let mut output_vec = Vec::new();
    loop {
        assert!(output_n <= TOTAL_POSITIONS - 1);
        if output_n == 0 && main_positions & output[0] == 0 {
            output[0] <<= 1;
            if output[0] >= (1 << TOTAL_POSITIONS) {
                break;
            }
        } else if output[output_n] >= (1 << TOTAL_POSITIONS) {
            // When we've iterated up to 9 then we've gone through all values
            // needed for that position and can increment the previous position
            output[output_n] = 1;
            output_n -= 1;
            output[output_n] <<= 1;
            if output[0] == (1 << TOTAL_POSITIONS) {
                break;
            }
        } else if previous_choises == output[output_n]
            || output[0..output_n].contains(&output[output_n])
        {
            // We can't choose the same value as what we know was the value the last time
            // and if the last element of our output exists elsewhere in our array,
            // it's an invalid value and we need to get a new one
            output[output_n] <<= 1;
        } else if output_n + 1 < (tail_length + 2)
            && need_backup(previous_choises, output[output_n], tail_length)
            && could_block_all(
                previous_choises,
                combine_positions(&output[0..=output_n]),
                &calculated_snakes,
                tail_length,
            ) {
            // Add another backup value if we need it
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

fn combine_positions(positions: &[u16]) -> u16 {
    positions.iter().fold(0, |acc, n| acc | n)
}

// The simplest chech for if `check_pos` may need a backup. Calculates
// if `check_pos` is `tail_length` away from any of `prev_pos_choises`
fn need_backup(prev_pos_choises: u16, check_pos: u16, tail_length: usize) -> bool {
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
    head_positions: u16,
    chosen_positions: u16,
    calculated_snakes: &HashMap<(usize, usize), FnvHashSet<u16>>,
    tail_length: usize,
) -> bool {
    for head in 0..TOTAL_POSITIONS {
        if (1 << head) & head_positions != 0 {
            let possible_snakes = match calculated_snakes.get(&(head, tail_length)) {
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

fn prepare_hashmap() -> HashMap<(usize, usize), FnvHashSet<u16>> {
    let mut calculated_snakes = HashMap::new();
    for snake_head_position in 0..TOTAL_POSITIONS {
        for tail_length in 1..(TOTAL_POSITIONS - 1) {
            calculated_snakes
                .entry((snake_head_position, tail_length))
                .or_insert_with(|| get_snake_blocks(tail_length, 1 << snake_head_position));
        }
    }
    calculated_snakes
}

enum Moves {
    Up,
    Right,
    Down,
    Left,
}

fn get_snake_blocks(tail_length: usize, head: u16) -> FnvHashSet<u16> {
    let mut valid_snakes: Vec<u16> = Vec::with_capacity(tail_length + 1);
    let all_moves = generate_moves(tail_length);

    // This will filter through all permutations of moves and keep the ones that a
    // snake could have taken without dying.
    'outer: for moves in all_moves {
        let mut current_pos = head;
        let mut positions_taken = head;

        let mut first_move = true;

        // rustc believes this must be initialized here.
        let mut absolute_direction = Moves::Up;

        for relative_direction in moves.iter().take(tail_length) {
            absolute_direction = if first_move {
                first_move = false;
                // Handle the first move differently as it can move in four directions
                match relative_direction {
                    0 => Moves::Up,
                    1 => Moves::Right,
                    2 => Moves::Down,
                    3 => Moves::Left,
                    _ => unreachable!(),
                }
            } else {
                // We will enter here on every iteration of the for loop except the first one.
                match (absolute_direction, relative_direction) {
                    (Moves::Up, 1) | (Moves::Right, 0) | (Moves::Left, 2) => Moves::Up,
                    (Moves::Up, 2) | (Moves::Right, 1) | (Moves::Down, 0) => Moves::Right,
                    (Moves::Right, 2) | (Moves::Down, 1) | (Moves::Left, 0) => Moves::Down,
                    (Moves::Up, 0) | (Moves::Down, 2) | (Moves::Left, 1) => Moves::Left,
                    _ => unreachable!(),
                }
            };
            match absolute_direction {
                Moves::Up => {
                    if current_pos < 1 << MAP_WIDTH {
                        continue 'outer;
                    }
                    current_pos >>= MAP_WIDTH;
                }
                Moves::Right => {
                    if current_pos.trailing_zeros() as usize % MAP_WIDTH == MAP_WIDTH - 1 {
                        continue 'outer;
                    }
                    current_pos <<= 1;
                }
                Moves::Down => {
                    if current_pos >= 1 << (MAP_WIDTH * (MAP_WIDTH - 1)) {
                        continue 'outer;
                    }
                    current_pos <<= MAP_WIDTH;
                }
                Moves::Left => {
                    if current_pos.trailing_zeros() as usize % MAP_WIDTH == 0 {
                        continue 'outer;
                    }
                    current_pos >>= 1;
                }
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
        possible_blocks.insert(snake);
        for perm in 0..=0b_1_1111_1111 {
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

// The symmetricity of a chosen set of positions allows some optimizations in
// the calculation of the total tree-sum. If the previous layer has a kind of
// symmetry, then some of the following position-branches can be treated as
// equal. To be more specific, if we imagine the grid as:
//
// 0 1 2
// 3 4 5
// 6 7 8
//
// and the grid had a Symmetry::Full (meaning flipping/mirroring/rotating it
// wouldn't matter) then the position-branches at 0, 2, 6 and 8 would have an
// identical branch-sum. Each Symmetry variant has a description of which
// branch-positions can be treated as equal in the following layer. If
// positions have the same letter, their branch-sums are the same.
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

fn symmetricity(points_n: u16) -> Symmetry {
    let horizontal: bool;
    let vertical: bool;
    let diagonal_down: bool;
    let diagonal_up: bool;
    if TOTAL_POSITIONS == 9 {
        let shifted_six = points_n >> 6 & points_n;
        let shifted_two = points_n >> 2 & points_n;
        let shifted_four = points_n >> 4 & points_n;
        let shifted_eight = points_n >> 8 & points_n;
        horizontal = (shifted_six & 0b0000_0111) == 0b0000_0111;
        vertical = (shifted_two & 0b0100_1001) == 0b0100_1001;
        diagonal_down = (shifted_two & 0b0010_0010) == 0b0010_0010
            && (shifted_four & 0b0000_0100) == 0b0000_0100;
        diagonal_up = (shifted_eight & 0b0000_0001) == 0b0000_0001
            && (shifted_four & 0b0000_0010) == 0b0000_0010
            && (shifted_four & 0b0000_1000) == 0b0000_1000;
    } else {
        return Symmetry::None;
    }
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
