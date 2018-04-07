extern crate fnv;
extern crate indicatif;
extern crate rayon;

use fnv::FnvHashSet;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
const MAP_WIDTH: usize = 3;

struct PositionFinder<'a, 'b, 'c> {
    main_positions: &'c [u8],
    output: Vec<u8>,
    previous_choises: &'b [u8],
    tail_length: usize,
    snakes_calculated: &'a HashMap<(u8, usize), FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>>,
    done: bool,
}

enum Moves {
    Up,
    Right,
    Down,
    Left,
}

impl<'a, 'b, 'c> PositionFinder<'a, 'b, 'c> {
    fn new(
        main_positions: &'c [u8],
        previous_choises: &'b [u8],
        tail_length: usize,
        snakes_calculated: &'a HashMap<(u8, usize), FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>>,
    ) -> PositionFinder<'a, 'b, 'c> {
        PositionFinder {
            main_positions,
            output: { vec![0] },
            previous_choises,
            tail_length,
            snakes_calculated: snakes_calculated,
            done: false,
        }
    }
}

impl<'a, 'b, 'c> Iterator for PositionFinder<'a, 'b, 'c> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        // We don't need to do anything if we're done
        if self.done {
            return None;
        }
        loop {
            if self.output.len() == 1 && !self.main_positions.contains(&self.output[0]) {
                self.output[0] += 1;
                if self.output[0] == 9 {
                    self.done = true;
                    return None;
                }
            } else if *self.output.last().unwrap() >= (MAP_WIDTH * MAP_WIDTH) as u8 {
                // When we've iterated up to MAP_WIDTH*MAP_WIDTH then we've gone through all values
                // needed for that positions and can increment the previous position
                self.output.pop();
                *self.output.last_mut().unwrap() += 1;
                if self.output[0] == (MAP_WIDTH * MAP_WIDTH) as u8 {
                    self.done = true;
                    return None;
                }
            } else if (self.previous_choises.len() == 1
                // If the last element of our output exists elsewhere in our array,
                // it's an invalid value and we need to get a new one
                && self.previous_choises[0] == *self.output.last().unwrap())
                || {
                    let (last, rest) = self.output.split_last().unwrap();
                    rest.contains(last)
                } {
                *self.output.last_mut().unwrap() += 1;
            } else if self.output.len() < (self.tail_length + 2)
                && need_backup(
                    &self.previous_choises,
                    *self.output.last().unwrap(),
                    self.tail_length,
                )
                && could_block_all(
                    &self.previous_choises,
                    &self.output,
                    &self.snakes_calculated,
                    self.tail_length,
                ) {
                // Add another backup value if we need it
                for backup in 0..((MAP_WIDTH * MAP_WIDTH) as u8) {
                    if !self.output.contains(&backup) {
                        self.output.push(backup);
                        break;
                    }
                }
            } else {
                // Our output is valid
                let out = self.output.clone();
                *self.output.last_mut().unwrap() += 1;
                if self.output[0] == (MAP_WIDTH * MAP_WIDTH) as u8 {
                    self.done = true;
                }
                return Some(out);
            }
        }
    }
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

fn get_valid_snakes(tail_length: usize, head: u8) -> FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]> {
    let mut move_container: Vec<Vec<u8>> = Vec::with_capacity(tail_length + 1);
    let mut positions_taken: Vec<u8> = Vec::with_capacity(tail_length + 1);
    let head_x = head % MAP_WIDTH as u8;
    let head_y = head / MAP_WIDTH as u8;
    let mut moves: [u8; MAP_WIDTH * MAP_WIDTH - 1] = [0; (MAP_WIDTH * MAP_WIDTH) - 1];
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
    let bars_orig = Arc::new(MultiProgress::new());
    let pb = bars_orig.add(ProgressBar::new(3));
    let bars = bars_orig.clone();
    let mut final_value = 0;
    let _ = thread::spawn(move || {
        pb.tick();
        let corners = count_down_tree(1, &[0], &snakes_calculated, &bars);
        pb.inc(1);
        println!("{}", corners);
        let side = count_down_tree(1, &[1], &snakes_calculated, &bars);
        pb.inc(1);
        println!("{}", side);
        let middle = count_down_tree(1, &[4], &snakes_calculated, &bars);
        pb.inc(1);
        println!("{}", middle);
        final_value = 4 * corners + 4 * side + middle;
        println!("{}", final_value);
        pb.finish_with_message("done");
    });
    bars_orig.join_and_clear().unwrap();
    println!("{}", final_value);
}

fn count_down_tree(
    tail_length: usize,
    previous_layer: &[u8],
    snakes_calculated: &HashMap<(u8, usize), FnvHashSet<[bool; MAP_WIDTH * MAP_WIDTH]>>,
    bars: &MultiProgress,
) -> u128 {
    match symmetricality(simplify(&previous_layer)) {
        Some(Symmetry::Horizontal) => {
            let mut possible_tops: Vec<Vec<u8>> = Vec::new();
            let mut possible_middles: Vec<Vec<u8>> = Vec::new();
            rayon::join(
                || {
                    possible_tops = PositionFinder::new(
                        &[0, 1, 2],
                        &previous_layer,
                        tail_length,
                        &snakes_calculated,
                    ).collect();
                },
                || {
                    possible_middles = PositionFinder::new(
                        &[3, 4, 5],
                        &previous_layer,
                        tail_length,
                        &snakes_calculated,
                    ).collect();
                },
            );
            let mut top_sum = 0;
            let mut middle_sum = 0;
            if tail_length == 6 {
                top_sum = possible_tops.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                rayon::join(
                    || {
                        let bar = bars.add(ProgressBar::new(possible_tops.len() as u64));
                        bar.set_style(ProgressStyle::default_bar().template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                        ));
                        bar.set_message(&format!("Horizontal:Top : {}", tail_length));
                        for layer in possible_tops {
                            bar.inc(1);
                            top_sum +=
                                count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                            bar.finish_and_clear();
                        }
                    },
                    || {
                        let bar = bars.add(ProgressBar::new(possible_middles.len() as u64));
                        bar.set_style(ProgressStyle::default_bar().template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                        ));
                        bar.set_message(&format!("Horizontal:Middle : {}", tail_length));
                        for layer in possible_middles {
                            bar.inc(1);
                            middle_sum +=
                                count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                        }
                        bar.finish_and_clear();
                    },
                );
            }

            2 * top_sum + middle_sum
        }
        Some(Symmetry::Vertical) => {
            let mut possible_sides: Vec<Vec<u8>> = Vec::new();
            let mut possible_middles: Vec<Vec<u8>> = Vec::new();
            rayon::join(
                || {
                    possible_sides = PositionFinder::new(
                        &[0, 3, 6],
                        &previous_layer,
                        tail_length,
                        &snakes_calculated,
                    ).collect();
                },
                || {
                    possible_middles = PositionFinder::new(
                        &[1, 4, 7],
                        &previous_layer,
                        tail_length,
                        &snakes_calculated,
                    ).collect();
                },
            );
            let mut side_sum = 0;
            let mut middle_sum = 0;
            if tail_length == 6 {
                side_sum = possible_sides.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                rayon::join(
                    || {
                        let bar = bars.add(ProgressBar::new(possible_sides.len() as u64));
                        bar.set_style(ProgressStyle::default_bar().template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                        ));
                        bar.set_message(&format!("Vertical:Side : {}", tail_length));
                        for layer in possible_sides {
                            bar.inc(1);
                            side_sum +=
                                count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                        }
                        bar.finish_and_clear();
                    },
                    || {
                        let bar = bars.add(ProgressBar::new(possible_middles.len() as u64));
                        bar.set_style(ProgressStyle::default_bar().template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                        ));
                        bar.set_message(&format!("Vertical:Middle : {}", tail_length));
                        for layer in possible_middles {
                            bar.inc(1);
                            middle_sum +=
                                count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                        }
                        bar.finish_and_clear();
                    },
                );
            }

            2 * side_sum + middle_sum
        }
        Some(Symmetry::Full) => {
            let mut possible_corners: Vec<Vec<u8>> = Vec::new();
            let mut possible_sides: Vec<Vec<u8>> = Vec::new();
            rayon::join(
                || {
                    possible_corners =
                        PositionFinder::new(&[0], &previous_layer, tail_length, &snakes_calculated)
                            .collect();
                },
                || {
                    possible_sides =
                        PositionFinder::new(&[1], &previous_layer, tail_length, &snakes_calculated)
                            .collect();
                },
            );
            let possible_middles: Vec<Vec<u8>> =
                PositionFinder::new(&[4], &previous_layer, tail_length, &snakes_calculated)
                    .collect();
            let mut corner_sum = 0;
            let mut side_sum = 0;
            let mut middle_sum = 0;
            if tail_length == 6 {
                corner_sum = possible_corners.len() as u128;
                side_sum = possible_sides.len() as u128;
                middle_sum = possible_middles.len() as u128;
            } else {
                rayon::join(
                    || {
                        let bar = bars.add(ProgressBar::new(possible_corners.len() as u64));
                        bar.set_style(ProgressStyle::default_bar().template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                        ));
                        bar.set_message(&format!("Full:Corner : {}", tail_length));
                        for layer in possible_corners {
                            bar.inc(1);
                            corner_sum +=
                                count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                        }
                    },
                    || {
                        let bar = bars.add(ProgressBar::new(possible_sides.len() as u64));
                        bar.set_style(ProgressStyle::default_bar().template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                        ));
                        bar.set_message(&format!("Full:Side : {}", tail_length));
                        for layer in possible_sides {
                            bar.inc(1);
                            side_sum +=
                                count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                        }
                    },
                );
                let bar = bars.add(ProgressBar::new(possible_middles.len() as u64));
                bar.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"),
                );
                bar.set_message(&format!("Full:Middle : {}", tail_length));
                for layer in possible_middles {
                    bar.inc(1);
                    middle_sum +=
                        count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                }
                bar.finish_and_clear();
            }
            4 * corner_sum + 4 * side_sum + middle_sum
        }
        None => {
            let mut first_half: Vec<Vec<u8>> = Vec::new();
            let mut second_half: Vec<Vec<u8>> = Vec::new();
            rayon::join(
                || {
                    first_half = PositionFinder::new(
                        &[4, 5, 6, 7, 8],
                        &previous_layer,
                        tail_length,
                        &snakes_calculated,
                    ).collect();
                },
                || {
                    second_half = PositionFinder::new(
                        &[0, 1, 2, 3],
                        &previous_layer,
                        tail_length,
                        &snakes_calculated,
                    ).collect();
                },
            );
            first_half.append(&mut second_half);
            let possible_positions = first_half;
            let mut sum = 0;
            if tail_length == 6 {
                sum = possible_positions.len() as u128
            } else {
                let bar = bars.add(ProgressBar::new(possible_positions.len() as u64));
                bar.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"),
                );
                bar.set_message(&format!("None : {}", tail_length));
                for layer in possible_positions {
                    bar.inc(1);
                    sum += count_down_tree(tail_length + 1, &layer, snakes_calculated, &bars);
                }
                bar.finish_and_clear();
            }
            sum
        }
    }
}

enum Symmetry {
    Horizontal,
    Vertical,
    Full,
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
    if horizontal && vertical {
        Some(Symmetry::Full)
    } else if vertical {
        Some(Symmetry::Vertical)
    } else if horizontal {
        Some(Symmetry::Horizontal)
    } else {
        None
    }
}
