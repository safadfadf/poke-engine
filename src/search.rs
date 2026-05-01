use crate::engine::evaluate::evaluate;
use crate::engine::generate_instructions::generate_instructions_from_move_pair;
use crate::engine::state::MoveChoice;
use crate::state::State;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

enum IterativeDeependingThreadMessage {
    Stop((Vec<MoveChoice>, Vec<MoveChoice>, Vec<f32>, i8)),
}

pub fn expectiminimax_search(
    state: &mut State,
    depth: i8,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    ab_prune: bool,
    mtx: &Arc<Mutex<bool>>,
) -> Vec<f32> {
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();
    let num_move_pairs = num_s1_moves * num_s2_moves;
    let worker_count = available_search_workers().min(num_s1_moves);

    if worker_count <= 1 || depth <= 1 || num_move_pairs < worker_count * 2 {
        return expectiminimax_search_sequential(
            state,
            depth,
            side_one_options,
            side_two_options,
            ab_prune,
            mtx,
        );
    }

    expectiminimax_search_parallel_root(
        state,
        depth,
        side_one_options,
        side_two_options,
        ab_prune,
        mtx,
        worker_count,
    )
}

fn available_search_workers() -> usize {
    thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
}

fn score_move_pair(
    state: &mut State,
    depth: i8,
    side_one_move: &MoveChoice,
    side_two_move: &MoveChoice,
    mtx: &Arc<Mutex<bool>>,
) -> f32 {
    let mut score = 0.0;
    let instructions =
        generate_instructions_from_move_pair(state, side_one_move, side_two_move, false);
    if depth == 0 {
        for instruction in instructions.iter() {
            state.apply_instructions(&instruction.instruction_list);
            score += instruction.percentage * evaluate(state) / 100.0;
            state.reverse_instructions(&instruction.instruction_list);
        }
    } else {
        for instruction in instructions.iter() {
            state.apply_instructions(&instruction.instruction_list);
            let (next_turn_side_one_options, next_turn_side_two_options) = state.get_all_options();

            let next_turn_side_one_options_len = next_turn_side_one_options.len();
            let next_turn_side_two_options_len = next_turn_side_two_options.len();
            let (_, safest) = pick_safest(
                &expectiminimax_search_sequential(
                    state,
                    depth,
                    next_turn_side_one_options,
                    next_turn_side_two_options,
                    true, // until there is something better than `pick_safest` for evaluating a sub-game, there is no point in this being anything other than `true`
                    mtx,
                ),
                next_turn_side_one_options_len,
                next_turn_side_two_options_len,
            );
            score += instruction.percentage * safest / 100.0;

            state.reverse_instructions(&instruction.instruction_list);
        }
    }

    score
}

fn expectiminimax_search_sequential(
    state: &mut State,
    mut depth: i8,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    ab_prune: bool,
    mtx: &Arc<Mutex<bool>>,
) -> Vec<f32> {
    depth -= 1;
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();
    let mut score_lookup: Vec<f32> = Vec::with_capacity(num_s1_moves * num_s2_moves);

    if *mtx.lock().unwrap() == false {
        for _ in 0..(num_s1_moves * num_s2_moves) {
            score_lookup.push(0.0);
        }
        return score_lookup;
    }

    let battle_is_over = state.battle_is_over();
    if battle_is_over != 0.0 {
        for _ in 0..(num_s1_moves * num_s2_moves) {
            score_lookup.push(((100.0 * depth as f32) * battle_is_over) + evaluate(state));
        }
        return score_lookup;
    }

    let mut skip;
    let mut alpha = f32::MIN;
    for side_one_move in side_one_options.iter().as_ref() {
        let mut beta = f32::MAX;
        skip = false;

        for side_two_move in side_two_options.iter().as_ref() {
            if skip {
                score_lookup.push(f32::NAN);
                continue;
            }

            let score = score_move_pair(state, depth, side_one_move, side_two_move, mtx);
            score_lookup.push(score);

            if ab_prune {
                if score < beta {
                    beta = score;
                }
                if score <= alpha {
                    skip = true;
                }
            }
        }
        if beta > alpha {
            alpha = beta;
        }
    }
    score_lookup
}

fn expectiminimax_search_parallel_root(
    state: &mut State,
    mut depth: i8,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    ab_prune: bool,
    mtx: &Arc<Mutex<bool>>,
    worker_count: usize,
) -> Vec<f32> {
    depth -= 1;
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();

    if *mtx.lock().unwrap() == false {
        return vec![0.0; num_s1_moves * num_s2_moves];
    }

    let battle_is_over = state.battle_is_over();
    if battle_is_over != 0.0 {
        return vec![
            ((100.0 * depth as f32) * battle_is_over) + evaluate(state);
            num_s1_moves * num_s2_moves
        ];
    }

    if ab_prune {
        parallel_root_with_pruning_shape(
            state,
            depth,
            side_one_options,
            side_two_options,
            mtx,
            worker_count,
        )
    } else {
        parallel_root_without_pruning(
            state,
            depth,
            side_one_options,
            side_two_options,
            mtx,
            worker_count,
        )
    }
}

fn parallel_root_without_pruning(
    state: &State,
    depth: i8,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    mtx: &Arc<Mutex<bool>>,
    worker_count: usize,
) -> Vec<f32> {
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();
    let base_state = state.clone();
    let chunk_size = (num_s1_moves + worker_count - 1) / worker_count;
    let mut rows = Vec::with_capacity(num_s1_moves);

    thread::scope(|scope| {
        let mut handles = Vec::new();
        for start in (0..num_s1_moves).step_by(chunk_size) {
            let end = (start + chunk_size).min(num_s1_moves);
            let base_state = base_state.clone();
            let side_one_options = &side_one_options;
            let side_two_options = &side_two_options;
            let mtx = Arc::clone(mtx);

            handles.push(scope.spawn(move || {
                let mut chunk_rows = Vec::with_capacity(end - start);
                for row_index in start..end {
                    let mut row_state = base_state.clone();
                    let mut row_scores = Vec::with_capacity(num_s2_moves);
                    for side_two_move in side_two_options {
                        row_scores.push(score_move_pair(
                            &mut row_state,
                            depth,
                            &side_one_options[row_index],
                            side_two_move,
                            &mtx,
                        ));
                    }
                    chunk_rows.push((row_index, row_scores));
                }
                chunk_rows
            }));
        }

        for handle in handles {
            rows.extend(handle.join().unwrap());
        }
    });

    rows.sort_by_key(|(row_index, _)| *row_index);
    let mut score_lookup = Vec::with_capacity(num_s1_moves * num_s2_moves);
    for (_, row_scores) in rows {
        score_lookup.extend(row_scores);
    }
    score_lookup
}

fn parallel_root_with_pruning_shape(
    state: &mut State,
    depth: i8,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    mtx: &Arc<Mutex<bool>>,
    worker_count: usize,
) -> Vec<f32> {
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();
    let mut row_scores = vec![vec![None; num_s2_moves]; num_s1_moves];
    let mut alpha = f32::MIN;

    if num_s1_moves == 0 {
        return Vec::new();
    }

    let mut beta = f32::MAX;
    let mut skip = false;
    for side_two_index in 0..num_s2_moves {
        if skip {
            continue;
        }

        let score = score_move_pair(
            state,
            depth,
            &side_one_options[0],
            &side_two_options[side_two_index],
            mtx,
        );
        row_scores[0][side_two_index] = Some(score);

        if score < beta {
            beta = score;
        }
        if score <= alpha {
            skip = true;
        }
    }
    if beta > alpha {
        alpha = beta;
    }

    if num_s1_moves > 1 {
        let base_state = state.clone();
        let start_row = 1;
        let remaining_rows = num_s1_moves - start_row;
        let worker_count = worker_count.min(remaining_rows);
        let chunk_size = (remaining_rows + worker_count - 1) / worker_count;
        let pruning_alpha = alpha;
        let mut rows = Vec::with_capacity(remaining_rows);

        thread::scope(|scope| {
            let mut handles = Vec::new();
            for chunk_start in (start_row..num_s1_moves).step_by(chunk_size) {
                let chunk_end = (chunk_start + chunk_size).min(num_s1_moves);
                let base_state = base_state.clone();
                let side_one_options = &side_one_options;
                let side_two_options = &side_two_options;
                let mtx = Arc::clone(mtx);

                handles.push(scope.spawn(move || {
                    let mut chunk_rows = Vec::with_capacity(chunk_end - chunk_start);
                    for row_index in chunk_start..chunk_end {
                        let mut row_state = base_state.clone();
                        let mut row = vec![None; num_s2_moves];
                        let mut beta = f32::MAX;
                        let mut skip = false;

                        for side_two_index in 0..num_s2_moves {
                            if skip {
                                continue;
                            }

                            let score = score_move_pair(
                                &mut row_state,
                                depth,
                                &side_one_options[row_index],
                                &side_two_options[side_two_index],
                                &mtx,
                            );
                            row[side_two_index] = Some(score);

                            if score < beta {
                                beta = score;
                            }
                            if score <= pruning_alpha {
                                skip = true;
                            }
                        }
                        chunk_rows.push((row_index, row));
                    }
                    chunk_rows
                }));
            }

            for handle in handles {
                rows.extend(handle.join().unwrap());
            }
        });

        for (row_index, row) in rows {
            row_scores[row_index] = row;
        }
    }

    replay_pruned_root_scores(&row_scores, num_s1_moves, num_s2_moves)
}

fn replay_pruned_root_scores(
    row_scores: &[Vec<Option<f32>>],
    num_s1_moves: usize,
    num_s2_moves: usize,
) -> Vec<f32> {
    let mut score_lookup = Vec::with_capacity(num_s1_moves * num_s2_moves);
    let mut alpha = f32::MIN;

    for side_one_index in 0..num_s1_moves {
        let mut beta = f32::MAX;
        let mut skip = false;

        for side_two_index in 0..num_s2_moves {
            if skip {
                score_lookup.push(f32::NAN);
                continue;
            }

            let score = row_scores[side_one_index][side_two_index]
                .expect("parallel root pruning skipped a score needed for replay");
            score_lookup.push(score);

            if score < beta {
                beta = score;
            }
            if score <= alpha {
                skip = true;
            }
        }

        if beta > alpha {
            alpha = beta;
        }
    }

    score_lookup
}

pub fn pick_safest(
    score_lookup: &Vec<f32>,
    num_s1_moves: usize,
    num_s2_moves: usize,
) -> (usize, f32) {
    let mut best_worst_case = f32::MIN;
    let mut best_worst_case_s1_index = 0;
    let mut vec_index = 0;

    for s1_index in 0..num_s1_moves {
        let mut worst_case_this_row = f32::MAX;
        for _ in 0..num_s2_moves {
            let score = score_lookup[vec_index];
            vec_index += 1;
            if score < worst_case_this_row {
                worst_case_this_row = score;
            }
        }
        if worst_case_this_row > best_worst_case {
            best_worst_case_s1_index = s1_index;
            best_worst_case = worst_case_this_row;
        }
    }

    (best_worst_case_s1_index, best_worst_case)
}

fn re_order_moves_for_iterative_deepening(
    last_search_result: &Vec<f32>,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
) -> (Vec<MoveChoice>, Vec<MoveChoice>) {
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();
    let mut worst_case_s1_scores: Vec<(MoveChoice, f32)> = vec![];
    let mut vec_index = 0;

    for s1_index in 0..num_s1_moves {
        let mut worst_case_this_row = f32::MAX;
        for _ in 0..num_s2_moves {
            let score = last_search_result[vec_index];
            vec_index += 1;
            if score < worst_case_this_row {
                worst_case_this_row = score;
            }
        }
        worst_case_s1_scores.push((side_one_options[s1_index].clone(), worst_case_this_row));
    }

    worst_case_s1_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let new_s1_vec = worst_case_s1_scores.iter().map(|x| x.0.clone()).collect();

    (new_s1_vec, side_two_options)
}

pub fn iterative_deepen_expectiminimax(
    state: &mut State,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    max_time: Duration,
) -> (Vec<MoveChoice>, Vec<MoveChoice>, Vec<f32>, i8) {
    let mut state_clone = state.clone();

    let mut result = expectiminimax_search(
        state,
        1,
        side_one_options.clone(),
        side_two_options.clone(),
        true,
        &Arc::new(Mutex::new(true)),
    );
    let (mut re_ordered_s1_options, mut re_ordered_s2_options) =
        re_order_moves_for_iterative_deepening(&result, side_one_options, side_two_options);
    let mut i = 1;
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    let (sender, receiver): (
        Sender<IterativeDeependingThreadMessage>,
        Receiver<IterativeDeependingThreadMessage>,
    ) = channel();

    let handle = thread::spawn(move || {
        let mut previous_turn_s1_options = re_ordered_s1_options.clone();
        let mut previous_turn_s2_options = re_ordered_s2_options.clone();
        loop {
            let previous_result = result;
            i += 1;
            result = expectiminimax_search(
                &mut state_clone,
                i,
                re_ordered_s1_options.clone(),
                re_ordered_s2_options.clone(),
                true,
                &running_clone,
            );

            // when we are told to stop, return the *previous* result.
            // the current result will be invalid
            if *running_clone.lock().unwrap() == false {
                sender
                    .send(IterativeDeependingThreadMessage::Stop((
                        previous_turn_s1_options,
                        previous_turn_s2_options,
                        previous_result,
                        i - 1,
                    )))
                    .unwrap();
                break;
            }
            previous_turn_s1_options = re_ordered_s1_options.clone();
            previous_turn_s2_options = re_ordered_s2_options.clone();
            (re_ordered_s1_options, re_ordered_s2_options) = re_order_moves_for_iterative_deepening(
                &result,
                re_ordered_s1_options,
                re_ordered_s2_options,
            );
        }
    });

    thread::sleep(max_time);
    *running.lock().unwrap() = false;
    match receiver.recv() {
        Ok(IterativeDeependingThreadMessage::Stop(result)) => {
            handle.join().unwrap();
            result
        }
        _ => panic!("Failed to receive stop message"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choices::Choices;
    use crate::state::PokemonMoveIndex;

    fn assert_scores_match(expected: &[f32], actual: &[f32]) {
        assert_eq!(expected.len(), actual.len());
        for (expected_score, actual_score) in expected.iter().zip(actual.iter()) {
            if expected_score.is_nan() {
                assert!(actual_score.is_nan());
            } else {
                assert!(
                    (expected_score - actual_score).abs() < 0.0001,
                    "expected {}, got {}",
                    expected_score,
                    actual_score
                );
            }
        }
    }

    fn search_state() -> State {
        let mut state = State::default();
        state
            .side_one
            .get_active()
            .replace_move(PokemonMoveIndex::M0, Choices::TACKLE);
        state
            .side_one
            .get_active()
            .replace_move(PokemonMoveIndex::M1, Choices::WATERGUN);
        state
            .side_one
            .get_active()
            .replace_move(PokemonMoveIndex::M2, Choices::GROWL);
        state
            .side_one
            .get_active()
            .replace_move(PokemonMoveIndex::M3, Choices::SPLASH);
        state
            .side_two
            .get_active()
            .replace_move(PokemonMoveIndex::M0, Choices::TACKLE);
        state
            .side_two
            .get_active()
            .replace_move(PokemonMoveIndex::M1, Choices::WATERGUN);
        state
            .side_two
            .get_active()
            .replace_move(PokemonMoveIndex::M2, Choices::GROWL);
        state
            .side_two
            .get_active()
            .replace_move(PokemonMoveIndex::M3, Choices::SPLASH);
        state
    }

    fn move_options() -> Vec<MoveChoice> {
        vec![
            MoveChoice::Move(PokemonMoveIndex::M0),
            MoveChoice::Move(PokemonMoveIndex::M1),
            MoveChoice::Move(PokemonMoveIndex::M2),
            MoveChoice::Move(PokemonMoveIndex::M3),
        ]
    }

    #[test]
    fn parallel_root_matches_sequential_expectiminimax_without_pruning() {
        let mtx = Arc::new(Mutex::new(true));
        let mut sequential_state = search_state();
        let mut parallel_state = sequential_state.clone();
        let side_one_options = move_options();
        let side_two_options = move_options();

        let expected = expectiminimax_search_sequential(
            &mut sequential_state,
            2,
            side_one_options.clone(),
            side_two_options.clone(),
            false,
            &mtx,
        );
        let actual = expectiminimax_search_parallel_root(
            &mut parallel_state,
            2,
            side_one_options,
            side_two_options,
            false,
            &mtx,
            4,
        );

        assert_scores_match(&expected, &actual);
    }

    #[test]
    fn parallel_root_matches_sequential_expectiminimax_with_pruning() {
        let mtx = Arc::new(Mutex::new(true));
        let mut sequential_state = search_state();
        let mut parallel_state = sequential_state.clone();
        let side_one_options = move_options();
        let side_two_options = move_options();

        let expected = expectiminimax_search_sequential(
            &mut sequential_state,
            2,
            side_one_options.clone(),
            side_two_options.clone(),
            true,
            &mtx,
        );
        let actual = expectiminimax_search_parallel_root(
            &mut parallel_state,
            2,
            side_one_options,
            side_two_options,
            true,
            &mtx,
            4,
        );

        assert_scores_match(&expected, &actual);
    }
}
