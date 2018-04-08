extern crate fnv;

use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::collections::HashMap;

const MAP_WIDTH: usize = 3;

const SEARCH_LENGTH: usize = 6;

enum Moves {
    Up,
    Right,
    Down,
    Left,
}

fn branches_below(
    main_positions: &[u8],
    previous_choises: &[u8],
    tail_length: usize,
    snakes_calculated: &HashMap<(u8, usize), FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>>,
) -> Vec<Vec<u8>> {
    let mut output = [0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut output_n = 0;
    let mut output_vec = Vec::new();
    // We don't need to do anything if we're done
    loop {
        if output_n == 0 && !main_positions.contains(&output[0]) {
            output[0] += 1;
            if output[0] >= 9 {
                break;
            }
        } else if output[output_n] >= (MAP_WIDTH * MAP_WIDTH) as u8 {
            // When we've iterated up to MAP_WIDTH*MAP_WIDTH then we've gone through all values
            // needed for that positions and can increment the previous position
            output[output_n] = 0;
            output_n -= 1;
            output[output_n] += 1;
            if output[0] == (MAP_WIDTH * MAP_WIDTH) as u8 {
                break;
            }
        } else if (
                // We can't choose the same value as what we know was the value the last time
                previous_choises.len() == 1 &&
                previous_choises[0] == output[output_n]
            ) ||
                // If the last element of our output exists elsewhere in our array,
                // it's an invalid value and we need to get a new one
                 {
                    let (last, rest) = output[0..=output_n].split_last().unwrap();
                    rest.contains(last)
                } {
            output[output_n] += 1;
        } else if output_n + 1 < (tail_length + 2)
            && need_backup(&previous_choises, output[output_n], tail_length)
            && could_block_all(
                &previous_choises,
                &output[0..=output_n],
                &snakes_calculated,
                tail_length,
            ) {
            // Add another backup value if we need it
            for backup in 0..((MAP_WIDTH * MAP_WIDTH) as u8) {
                if !output[0..=output_n].contains(&backup) {
                    output_n += 1;
                    output[output_n] = backup;
                    break;
                }
            }
        } else {
            // Our output is valid
            let mut out = output[0..=output_n].to_vec();
            out.sort();
            output_vec.push(out);

            output[output_n] += 1;
            if output[0] == (MAP_WIDTH * MAP_WIDTH) as u8 {
                break;
            }
        }
    }
    output_vec
}

// The simplest chech for if `check_pos` may need a backup. Calculates
// if `check_pos` is `tail_length` away from any of `prev_pos_choises`
#[inline]
fn need_backup(prev_pos_choises: &[u8], check_pos: u8, tail_length: usize) -> bool {
    let check_pos_x = check_pos % MAP_WIDTH as u8;
    let check_pos_y = check_pos / MAP_WIDTH as u8;
    prev_pos_choises.iter().any(|choise| {
        let prev_pos_x = choise % MAP_WIDTH as u8;
        let prev_pos_y = choise / MAP_WIDTH as u8;
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
    snakes_calculated: &HashMap<(u8, usize), FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>>,
    tail_length: usize,
) -> bool {
    'outer_for_loop: for head in head_positions {
        let possible_snakes = match snakes_calculated.get(&(*head, tail_length)) {
            Some(x) => x,
            None => panic!("WRONG"),
        };
        let simple = simplify(chosen_positions);
        if possible_snakes.contains(&simple) {
            return true;
        }
    }
    false
}

// Not neccessirarily valid moves
fn generate_moves(max: usize) -> Vec<[u8; MAP_WIDTH * MAP_WIDTH - 1]> {
    let mut all_moves: Vec<[u8; MAP_WIDTH * MAP_WIDTH - 1]> = Vec::new();
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

fn get_valid_snakes(tail_length: usize, head: u8) -> FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]> {
    let mut move_container: Vec<Vec<u8>> = Vec::with_capacity(tail_length + 1);
    let mut positions_taken: Vec<u8> = Vec::with_capacity(tail_length + 1);
    let head_x = head % MAP_WIDTH as u8;
    let head_y = head / MAP_WIDTH as u8;
    let all_moves = generate_moves(tail_length);
    'outer: for moves in all_moves {
        let mut current_x = head_x;
        let mut current_y = head_y;
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
                if current_x == MAP_WIDTH as u8 - 1 {
                    continue 'outer;
                }
                current_x += 1;
                last_move = 1;
            }
            2 => {
                // DOWN
                if current_y == MAP_WIDTH as u8 - 1 {
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
        positions_taken.push(current_y * MAP_WIDTH as u8 + current_x);

        for direction in moves.iter().take(tail_length).skip(1) {
            let chosen_move = match (last_move, *direction) {
                (0, 1) | (1, 0) | (3, 2) => Moves::Up,
                (0, 2) | (1, 1) | (2, 0) => Moves::Right,
                (1, 2) | (2, 1) | (3, 0) => Moves::Down,
                (0, 0) | (2, 2) | (3, 1) => Moves::Left,
                _ => unreachable!(),
            };
            match chosen_move {
                Moves::Up => {
                    if current_y == 0 {
                        continue 'outer;
                    }
                    current_y -= 1;
                    last_move = 0;
                }
                Moves::Right => {
                    if current_x == MAP_WIDTH as u8 - 1 {
                        continue 'outer;
                    }
                    current_x += 1;
                    last_move = 1;
                }
                Moves::Down => {
                    if current_y == MAP_WIDTH as u8 - 1 {
                        continue 'outer;
                    }
                    current_y += 1;
                    last_move = 2;
                }
                Moves::Left => {
                    if current_x == 0 {
                        continue 'outer;
                    }
                    current_x -= 1;
                    last_move = 3;
                }
            }
            let n = current_y * MAP_WIDTH as u8 + current_x;
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
        let mut out = [false; MAP_WIDTH * MAP_WIDTH];
        for square in snake {
            out[square as usize] = true;
        }
        insert_permutations(out, &mut possible_blocks);
    }
    possible_blocks
}

// Creates permutations of `list` and inserts them into `possible blocks`
// Example:
// list = [false, true,  true,  false];
//
// What will be inserted:
// [false, true,  true,  false]
// [false, true,  false, false]
// [false, false, true,  false]
// [false, false, false, false]
//
fn insert_permutations(
    mut list: [bool; MAP_WIDTH * MAP_WIDTH],
    possible_blocks: &mut FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>,
) {
    let original_list = list;
    possible_blocks.insert(list);
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
        possible_blocks.insert(list);
    }
}

fn main() {
    // Prepare Hashmap
    let mut snakes_calculated: HashMap<(u8, usize), FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>> =
        HashMap::new();
    for o in 0..9 {
        for p in 1..8 {
            snakes_calculated
                .entry((o, p))
                .or_insert_with(|| get_valid_snakes(p, o));
        }
    }
    println!("Done generating");
    let mut hashed_branches = FnvHashMap::default();
    let corners = count_down_tree(1, &[0], &snakes_calculated, &mut hashed_branches);
    println!("{}", corners);
    let side = count_down_tree(1, &[1], &snakes_calculated, &mut hashed_branches);
    println!("{}", side);
    let middle = count_down_tree(1, &[4], &snakes_calculated, &mut hashed_branches);
    println!("{}", middle);
    let final_value = 4 * corners + 4 * side + middle;
    println!("{}", final_value);
}

fn count_down_tree(
    tail_length: usize,
    previous_layer: &[u8],
    snakes_calculated: &HashMap<(u8, usize), FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>>,
    hashed_branches: &mut FnvHashMap<([bool; 9], usize), u128>,
) -> u128 {
    let previous_layer_simple = simplify(&previous_layer);
    match hashed_branches.get(&(previous_layer_simple, tail_length)) {
        Some(value) => return *value,
        None => {}
    }
    match symmetricality(previous_layer_simple) {
        Some(Symmetry::Horizontal) => {
            let possible_tops =
                branches_below(&[0, 1, 2], &previous_layer, tail_length, &snakes_calculated);
            let possible_middles =
                branches_below(&[3, 4, 5], &previous_layer, tail_length, &snakes_calculated);
            let mut top_sum = 0;
            let mut middle_sum = 0;
            if tail_length == SEARCH_LENGTH {
                top_sum = possible_tops.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                for layer in possible_tops {
                    top_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_middles {
                    middle_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }

            let total = 2 * top_sum + middle_sum;
            hashed_branches.insert((previous_layer_simple, tail_length), total);
            total
        }
        Some(Symmetry::Vertical) => {
            let possible_sides =
                branches_below(&[0, 3, 6], &previous_layer, tail_length, &snakes_calculated);
            let possible_middles =
                branches_below(&[1, 4, 7], &previous_layer, tail_length, &snakes_calculated);
            let mut side_sum = 0;
            let mut middle_sum = 0;
            if tail_length == SEARCH_LENGTH {
                side_sum = possible_sides.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                for layer in possible_sides {
                    side_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_middles {
                    middle_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }

            let total = 2 * side_sum + middle_sum;
            hashed_branches.insert((previous_layer_simple, tail_length), total);
            total
        }
        Some(Symmetry::Full) => {
            let possible_corners =
                branches_below(&[0], &previous_layer, tail_length, &snakes_calculated);
            let possible_sides =
                branches_below(&[1], &previous_layer, tail_length, &snakes_calculated);
            let possible_middles: Vec<Vec<u8>> =
                branches_below(&[4], &previous_layer, tail_length, &snakes_calculated);
            let mut corner_sum = 0;
            let mut side_sum = 0;
            let mut middle_sum = 0;
            if tail_length == SEARCH_LENGTH {
                corner_sum = possible_corners.len() as u128;
                side_sum = possible_sides.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                for layer in possible_corners {
                    corner_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_sides {
                    side_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_middles {
                    middle_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }

            let total = 4 * corner_sum + 4 * side_sum + middle_sum;
            hashed_branches.insert((previous_layer_simple, tail_length), total);
            total
        }
        Some(Symmetry::Plus) => {
            let possible_corners =
                branches_below(&[0], &previous_layer, tail_length, &snakes_calculated);
            let possible_sides_horizontal =
                branches_below(&[1], &previous_layer, tail_length, &snakes_calculated);
            let possible_sides_vertical =
                branches_below(&[3], &previous_layer, tail_length, &snakes_calculated);
            let possible_middles =
                branches_below(&[4], &previous_layer, tail_length, &snakes_calculated);
            let mut corner_sum = 0;
            let mut side_sum_horizontal = 0;
            let mut side_sum_vertical = 0;
            let mut middle_sum = 0;
            if tail_length == SEARCH_LENGTH {
                corner_sum = possible_corners.len() as u128;
                side_sum_horizontal = possible_sides_horizontal.len() as u128;
                side_sum_vertical = possible_sides_vertical.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                for layer in possible_corners {
                    corner_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_sides_vertical {
                    side_sum_vertical += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_sides_horizontal {
                    side_sum_horizontal += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_middles {
                    middle_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }

            let total =
                4 * corner_sum + 2 * side_sum_vertical + 2 * side_sum_horizontal + middle_sum;
            hashed_branches.insert((previous_layer_simple, tail_length), total);
            total
        }
        Some(Symmetry::X) => {
            let possible_corners_one =
                branches_below(&[0], &previous_layer, tail_length, &snakes_calculated);
            let possible_corners_two =
                branches_below(&[2], &previous_layer, tail_length, &snakes_calculated);
            let possible_sides =
                branches_below(&[3], &previous_layer, tail_length, &snakes_calculated);
            let possible_middles =
                branches_below(&[4], &previous_layer, tail_length, &snakes_calculated);
            let mut corner_one_sum = 0;
            let mut corner_two_sum = 0;
            let mut side_sum = 0;
            let mut middle_sum = 0;
            if tail_length == SEARCH_LENGTH {
                corner_one_sum = possible_corners_one.len() as u128;
                corner_two_sum = possible_corners_two.len() as u128;
                side_sum = possible_sides.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                for layer in possible_corners_one {
                    corner_one_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_corners_two {
                    corner_two_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_sides {
                    side_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_middles {
                    middle_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }

            let total = 4 * side_sum + 2 * corner_one_sum + 2 * corner_two_sum + middle_sum;
            hashed_branches.insert((previous_layer_simple, tail_length), total);
            total
        }
        Some(Symmetry::DiagonalDown) => {
            let possible_sides =
                branches_below(&[1, 2, 5], &previous_layer, tail_length, &snakes_calculated);
            let possible_middles =
                branches_below(&[0, 4, 8], &previous_layer, tail_length, &snakes_calculated);
            let mut side_sum = 0;
            let mut middle_sum = 0;
            if tail_length == SEARCH_LENGTH {
                side_sum = possible_sides.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                for layer in possible_sides {
                    side_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_middles {
                    middle_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }

            let total = 2 * side_sum + middle_sum;
            hashed_branches.insert((previous_layer_simple, tail_length), total);
            total
        }
        Some(Symmetry::DiagonalUp) => {
            let possible_sides =
                branches_below(&[0, 1, 3], &previous_layer, tail_length, &snakes_calculated);
            let possible_middles =
                branches_below(&[2, 4, 6], &previous_layer, tail_length, &snakes_calculated);
            let mut side_sum = 0;
            let mut middle_sum = 0;
            if tail_length == SEARCH_LENGTH {
                side_sum = possible_sides.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                for layer in possible_sides {
                    side_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
                for layer in possible_middles {
                    middle_sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }

            let total = 2 * side_sum + middle_sum;
            hashed_branches.insert((previous_layer_simple, tail_length), total);
            total
        }
        None => {
            let possible_positions = branches_below(
                &[0, 1, 2, 3, 4, 5, 6, 7, 8],
                &previous_layer,
                tail_length,
                &snakes_calculated,
            );
            let mut sum = 0;
            if tail_length == SEARCH_LENGTH {
                sum = possible_positions.len() as u128
            } else {
                for layer in possible_positions {
                    sum += count_down_tree(
                        tail_length + 1,
                        &layer,
                        snakes_calculated,
                        hashed_branches,
                    );
                }
            }
            hashed_branches.insert((previous_layer_simple, tail_length), sum);
            sum
        }
    }
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
    X,
}

#[inline]
fn simplify(points: &[u8]) -> [bool; 9] {
    let mut simple_format = [false; 9];
    for pos in points {
        simple_format[*pos as usize] = true;
    }
    simple_format
}

fn symmetricality(points: [bool; 9]) -> Option<Symmetry> {
    let horizontal = points[0] == points[6] && points[1] == points[7] && points[2] == points[8];
    let vertical = points[0] == points[2] && points[3] == points[5] && points[6] == points[8];
    let diagonal_down = points[1] == points[3] && points[2] == points[6] && points[5] == points[7];
    let diagonal_up = points[0] == points[8] && points[1] == points[5] && points[3] == points[7];
    if horizontal && vertical && diagonal_down && diagonal_up {
        Some(Symmetry::Full)
    } else if horizontal && vertical {
        Some(Symmetry::Plus)
    } else if vertical {
        Some(Symmetry::Vertical)
    } else if horizontal {
        Some(Symmetry::Horizontal)
    } else if diagonal_down && diagonal_up {
        Some(Symmetry::X)
    } else if diagonal_up {
        Some(Symmetry::DiagonalUp)
    } else if diagonal_down {
        Some(Symmetry::DiagonalDown)
    } else {
        None
    }
}
