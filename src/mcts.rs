use crate::engine::evaluate::evaluate;
use crate::engine::generate_instructions::generate_instructions_from_move_pair;
use crate::engine::state::MoveChoice;
use crate::instruction::StateInstructions;
use crate::state::State;
use rand::prelude::*;
use rand::rng;
use std::thread;
use std::time::{Duration, Instant};

const MCTS_MAX_ITERATIONS_PER_TREE: u32 = 10_000_000;
const MCTS_DEADLINE_CHECK_INTERVAL: u32 = 128;
const MCTS_THREADS_ENV: &str = "POKE_ENGINE_MCTS_THREADS";
const MCTS_SYNC_TREE_DROP_ENV: &str = "POKE_ENGINE_MCTS_SYNC_TREE_DROP";
const MCTS_DAMAGE_BRANCH_DEPTH: u8 = 3;

fn sigmoid(x: f32) -> f32 {
    // Tuned so that ~200 points is very close to 1.0
    1.0 / (1.0 + (-0.0125 * x).exp())
}

#[derive(Debug)]
pub struct Node {
    pub root: bool,
    pub parent: *mut Node,
    pub children: Option<Box<NodeChildren>>,
    pub times_visited: u32,

    // represents the instructions & s1/s2 moves that led to this node from the parent
    pub instructions: StateInstructions,
    pub s1_choice: u8,
    pub s2_choice: u8,
    pub depth: u8,

    // represents the total score and number of visits for this node
    // de-coupled for s1 and s2
    pub s1_options: Option<Vec<MoveNode>>,
    pub s2_options: Option<Vec<MoveNode>>,
}

#[derive(Debug)]
pub struct NodeChildren {
    s2_len: usize,
    entries: Vec<Option<NodeBranch>>,
}

#[derive(Debug)]
pub struct NodeBranch {
    nodes: Vec<Node>,
    total_weight: f32,
}

impl NodeChildren {
    fn new(s1_len: usize, s2_len: usize) -> NodeChildren {
        let mut entries = Vec::with_capacity(s1_len.saturating_mul(s2_len));
        entries.resize_with(s1_len.saturating_mul(s2_len), || None);
        NodeChildren { s2_len, entries }
    }

    fn index(&self, s1_index: usize, s2_index: usize) -> usize {
        s1_index * self.s2_len + s2_index
    }

    #[inline]
    fn get_mut(&mut self, s1_index: usize, s2_index: usize) -> Option<&mut NodeBranch> {
        let index = self.index(s1_index, s2_index);
        self.entries.get_mut(index).and_then(|entry| entry.as_mut())
    }

    #[inline]
    fn insert(&mut self, s1_index: usize, s2_index: usize, branch: NodeBranch) {
        let index = self.index(s1_index, s2_index);
        self.entries[index] = Some(branch);
    }
}

impl Node {
    fn new() -> Node {
        Node {
            root: false,
            parent: std::ptr::null_mut(),
            instructions: StateInstructions::default(),
            times_visited: 0,
            children: None,
            s1_choice: 0,
            s2_choice: 0,
            depth: 0,
            s1_options: None,
            s2_options: None,
        }
    }
    unsafe fn populate(&mut self, s1_options: Vec<MoveChoice>, s2_options: Vec<MoveChoice>) {
        let s1_options_vec: Vec<MoveNode> = s1_options
            .iter()
            .map(|x| MoveNode {
                move_choice: x.clone(),
                total_score: 0.0,
                visits: 0,
            })
            .collect();
        let s2_options_vec: Vec<MoveNode> = s2_options
            .iter()
            .map(|x| MoveNode {
                move_choice: x.clone(),
                total_score: 0.0,
                visits: 0,
            })
            .collect();

        self.s1_options = Some(s1_options_vec);
        self.s2_options = Some(s2_options_vec);
    }

    #[inline]
    pub fn maximize_ucb_for_side(&self, side_map: &[MoveNode]) -> usize {
        let mut choice = 0;
        let mut best_ucb1 = f32::MIN;
        let parent_exploration_numerator = 2.0 * (self.times_visited as f32).ln();
        for (index, node) in side_map.iter().enumerate() {
            let this_ucb1 = node.ucb1_with_parent_exploration(parent_exploration_numerator);
            if this_ucb1 > best_ucb1 {
                best_ucb1 = this_ucb1;
                choice = index;
            }
        }
        choice
    }

    pub unsafe fn selection(&mut self, state: &mut State) -> (*mut Node, usize, usize) {
        let return_node = self as *mut Node;
        if self.s1_options.is_none() {
            let (s1_options, s2_options) = state.get_all_options();
            self.populate(s1_options, s2_options);
        }

        let s1_mc_index = self.maximize_ucb_for_side(&self.s1_options.as_ref().unwrap());
        let s2_mc_index = self.maximize_ucb_for_side(&self.s2_options.as_ref().unwrap());
        let child_vector = self
            .children
            .as_mut()
            .and_then(|children| children.get_mut(s1_mc_index, s2_mc_index));
        match child_vector {
            Some(child_vector) => {
                let child_branch_ptr = child_vector as *mut NodeBranch;
                let chosen_child = self.sample_node(child_branch_ptr);
                state.apply_instructions(&(*chosen_child).instructions.instruction_list);
                (*chosen_child).selection(state)
            }
            None => (return_node, s1_mc_index, s2_mc_index),
        }
    }

    #[inline]
    unsafe fn sample_node(&self, branch: *mut NodeBranch) -> *mut Node {
        let mut rng = rng();
        let branch = &mut *branch;
        let move_slice = &mut branch.nodes;
        let mut threshold = rng.random_range(0.0..branch.total_weight);
        let mut chosen_index = move_slice.len().saturating_sub(1);
        for (index, node) in move_slice.iter().enumerate() {
            threshold -= node.instructions.percentage.max(0.0);
            if threshold <= 0.0 {
                chosen_index = index;
                break;
            }
        }
        let chosen_node = &mut move_slice[chosen_index];
        let chosen_node_ptr = chosen_node as *mut Node;
        chosen_node_ptr
    }

    pub unsafe fn expand(
        &mut self,
        state: &mut State,
        s1_move_index: usize,
        s2_move_index: usize,
    ) -> *mut Node {
        let s1_move = &self.s1_options.as_ref().unwrap()[s1_move_index].move_choice;
        let s2_move = &self.s2_options.as_ref().unwrap()[s2_move_index].move_choice;
        // if the battle is over or both moves are none there is no need to expand
        if (state.battle_is_over() != 0.0 && !self.root)
            || (s1_move == &MoveChoice::None && s2_move == &MoveChoice::None)
        {
            return self as *mut Node;
        }
        let should_branch_on_damage = self.depth < MCTS_DAMAGE_BRANCH_DEPTH;
        let mut new_instructions =
            generate_instructions_from_move_pair(state, s1_move, s2_move, should_branch_on_damage);
        let mut this_pair_vec = Vec::with_capacity(new_instructions.len());
        for state_instructions in new_instructions.drain(..) {
            let mut new_node = Node::new();
            new_node.parent = self;
            new_node.instructions = state_instructions;
            new_node.s1_choice = s1_move_index as u8;
            new_node.s2_choice = s2_move_index as u8;
            new_node.depth = self.depth.saturating_add(1);

            this_pair_vec.push(new_node);
        }

        // sample a node from the new instruction list.
        // this is the node that the rollout will be done on
        let total_weight = this_pair_vec
            .iter()
            .map(|node| node.instructions.percentage.max(0.0))
            .sum();
        let mut branch = NodeBranch {
            nodes: this_pair_vec,
            total_weight,
        };
        let new_node_ptr = self.sample_node(&mut branch);
        state.apply_instructions(&(*new_node_ptr).instructions.instruction_list);
        let s1_options_len = self.s1_options.as_ref().unwrap().len();
        let s2_options_len = self.s2_options.as_ref().unwrap().len();
        self.children
            .get_or_insert_with(|| Box::new(NodeChildren::new(s1_options_len, s2_options_len)))
            .insert(s1_move_index, s2_move_index, branch);
        new_node_ptr
    }

    pub unsafe fn backpropagate(&mut self, score: f32, state: &mut State) {
        self.times_visited += 1;
        if self.root {
            return;
        }

        let parent_s1_movenode =
            &mut (*self.parent).s1_options.as_mut().unwrap()[self.s1_choice as usize];
        parent_s1_movenode.total_score += score;
        parent_s1_movenode.visits += 1;

        let parent_s2_movenode =
            &mut (*self.parent).s2_options.as_mut().unwrap()[self.s2_choice as usize];
        parent_s2_movenode.total_score += 1.0 - score;
        parent_s2_movenode.visits += 1;

        state.reverse_instructions(&self.instructions.instruction_list);
        (*self.parent).backpropagate(score, state);
    }

    pub fn rollout(&mut self, state: &mut State, root_eval: &f32) -> f32 {
        let battle_is_over = state.battle_is_over();
        if battle_is_over == 0.0 {
            let eval = evaluate(state);
            sigmoid(eval - root_eval)
        } else {
            if battle_is_over == -1.0 {
                0.0
            } else {
                battle_is_over
            }
        }
    }
}

#[derive(Debug)]
pub struct MoveNode {
    pub move_choice: MoveChoice,
    pub total_score: f32,
    pub visits: u32,
}

impl MoveNode {
    #[inline]
    pub fn ucb1(&self, parent_visits: u32) -> f32 {
        self.ucb1_with_parent_exploration(2.0 * (parent_visits as f32).ln())
    }

    #[inline]
    pub fn ucb1_with_parent_exploration(&self, parent_exploration_numerator: f32) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let score = (self.total_score / self.visits as f32)
            + (parent_exploration_numerator / self.visits as f32).sqrt();
        score
    }
    pub fn average_score(&self) -> f32 {
        let score = self.total_score / self.visits as f32;
        score
    }
}

#[derive(Clone)]
pub struct MctsSideResult {
    pub move_choice: MoveChoice,
    pub total_score: f32,
    pub visits: u32,
}

impl MctsSideResult {
    pub fn average_score(&self) -> f32 {
        if self.visits == 0 {
            return 0.0;
        }
        let score = self.total_score / self.visits as f32;
        score
    }
}

pub struct MctsResult {
    pub s1: Vec<MctsSideResult>,
    pub s2: Vec<MctsSideResult>,
    pub iteration_count: u32,
}

struct FinishedMctsTree(Node);

// A finished tree is only moved to another thread for destruction after all
// statistics have been copied out. The raw parent pointers are never read while
// dropping, so moving this wrapper is safe even though active Nodes are not Send.
unsafe impl Send for FinishedMctsTree {}

fn drop_finished_mcts_tree(tree: FinishedMctsTree) {
    let FinishedMctsTree(root_node) = tree;
    drop(root_node);
}

fn should_drop_finished_tree_async() -> bool {
    !matches!(
        std::env::var(MCTS_SYNC_TREE_DROP_ENV).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

fn drop_mcts_tree(root_node: Node, async_drop: bool) {
    let finished_tree = FinishedMctsTree(root_node);
    if async_drop && should_drop_finished_tree_async() {
        thread::spawn(move || drop_finished_mcts_tree(finished_tree));
    } else {
        drop_finished_mcts_tree(finished_tree);
    }
}

fn do_mcts(root_node: &mut Node, state: &mut State, root_eval: &f32) {
    let (mut new_node, s1_move, s2_move) = unsafe { root_node.selection(state) };
    new_node = unsafe { (*new_node).expand(state, s1_move, s2_move) };
    let rollout_result = unsafe { (*new_node).rollout(state, root_eval) };
    unsafe { (*new_node).backpropagate(rollout_result, state) }
}

fn mcts_worker_count() -> usize {
    let available_parallelism = thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1);

    // Root-parallel MCTS builds independent trees and merges only root stats.
    // That is faster, but it is not equivalent to the legacy single-tree search,
    // so keep it opt-in for callers that prefer speed over identical policy.
    std::env::var(MCTS_THREADS_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .map(|value| value.min(available_parallelism))
        .unwrap_or(1)
        .max(1)
}

fn perform_mcts_single_thread(
    state: &mut State,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    deadline: Instant,
    root_eval: f32,
    max_iterations: u32,
    async_tree_drop: bool,
) -> MctsResult {
    let mut root_node = Node::new();
    unsafe {
        root_node.populate(side_one_options, side_two_options);
    }
    root_node.root = true;

    let mut iterations_until_deadline_check = 0;
    while root_node.times_visited < max_iterations {
        if iterations_until_deadline_check == 0 {
            if Instant::now() >= deadline {
                break;
            }
            iterations_until_deadline_check = MCTS_DEADLINE_CHECK_INTERVAL;
        }

        do_mcts(&mut root_node, state, &root_eval);
        iterations_until_deadline_check -= 1;

        /*
        Cut off after 10 million iterations

        Under normal circumstances the bot will only run for 2.5-3.5 million iterations
        however towards the end of a battle the bot may perform tens of millions of iterations

        Beyond about 30 million iterations some floating point nonsense happens where
        MoveNode.total_score stops updating because f32 does not have enough precision

        I can push the problem farther out by using f64 but if the bot is running for 10 million iterations
        then it almost certainly sees a forced win
        */
        if root_node.times_visited >= max_iterations {
            break;
        }
    }

    let result = MctsResult {
        s1: root_node
            .s1_options
            .as_ref()
            .unwrap()
            .iter()
            .map(|v| MctsSideResult {
                move_choice: v.move_choice.clone(),
                total_score: v.total_score,
                visits: v.visits,
            })
            .collect(),
        s2: root_node
            .s2_options
            .as_ref()
            .unwrap()
            .iter()
            .map(|v| MctsSideResult {
                move_choice: v.move_choice.clone(),
                total_score: v.total_score,
                visits: v.visits,
            })
            .collect(),
        iteration_count: root_node.times_visited,
    };

    drop_mcts_tree(root_node, async_tree_drop);

    result
}

fn merge_mcts_results(
    results: Vec<MctsResult>,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
) -> MctsResult {
    let mut s1 = side_one_options
        .into_iter()
        .map(|move_choice| MctsSideResult {
            move_choice,
            total_score: 0.0,
            visits: 0,
        })
        .collect::<Vec<_>>();
    let mut s2 = side_two_options
        .into_iter()
        .map(|move_choice| MctsSideResult {
            move_choice,
            total_score: 0.0,
            visits: 0,
        })
        .collect::<Vec<_>>();
    let mut iteration_count = 0;

    for mut result in results {
        debug_assert_eq!(s1.len(), result.s1.len());
        debug_assert_eq!(s2.len(), result.s2.len());
        iteration_count += result.iteration_count;

        for (combined, worker_result) in s1.iter_mut().zip(result.s1.drain(..)) {
            combined.total_score += worker_result.total_score;
            combined.visits += worker_result.visits;
        }

        for (combined, worker_result) in s2.iter_mut().zip(result.s2.drain(..)) {
            combined.total_score += worker_result.total_score;
            combined.visits += worker_result.visits;
        }
    }

    MctsResult {
        s1,
        s2,
        iteration_count,
    }
}

pub fn perform_mcts(
    state: &mut State,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    max_time: Duration,
) -> MctsResult {
    let worker_count = mcts_worker_count();
    let root_eval = evaluate(state);
    let deadline = Instant::now() + max_time;
    let max_iterations_per_worker = MCTS_MAX_ITERATIONS_PER_TREE.div_ceil(worker_count as u32);

    if worker_count == 1 {
        return perform_mcts_single_thread(
            state,
            side_one_options,
            side_two_options,
            deadline,
            root_eval,
            MCTS_MAX_ITERATIONS_PER_TREE,
            false,
        );
    }

    let mut worker_handles = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let mut worker_state = state.clone();
        let worker_side_one_options = side_one_options.clone();
        let worker_side_two_options = side_two_options.clone();

        worker_handles.push(thread::spawn(move || {
            perform_mcts_single_thread(
                &mut worker_state,
                worker_side_one_options,
                worker_side_two_options,
                deadline,
                root_eval,
                max_iterations_per_worker,
                true,
            )
        }));
    }

    let results = worker_handles
        .into_iter()
        .map(|handle| handle.join().expect("MCTS worker thread panicked"))
        .collect();

    merge_mcts_results(results, side_one_options, side_two_options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PokemonMoveIndex;

    #[test]
    fn merge_mcts_results_preserves_order_and_sums_worker_totals() {
        let side_one_options = vec![
            MoveChoice::Move(PokemonMoveIndex::M0),
            MoveChoice::Move(PokemonMoveIndex::M1),
        ];
        let side_two_options = vec![MoveChoice::Move(PokemonMoveIndex::M2), MoveChoice::None];
        let results = vec![
            MctsResult {
                s1: vec![
                    MctsSideResult {
                        move_choice: side_one_options[0],
                        total_score: 1.5,
                        visits: 2,
                    },
                    MctsSideResult {
                        move_choice: side_one_options[1],
                        total_score: 3.0,
                        visits: 4,
                    },
                ],
                s2: vec![
                    MctsSideResult {
                        move_choice: side_two_options[0],
                        total_score: 5.0,
                        visits: 6,
                    },
                    MctsSideResult {
                        move_choice: side_two_options[1],
                        total_score: 7.0,
                        visits: 8,
                    },
                ],
                iteration_count: 10,
            },
            MctsResult {
                s1: vec![
                    MctsSideResult {
                        move_choice: side_one_options[0],
                        total_score: 2.5,
                        visits: 3,
                    },
                    MctsSideResult {
                        move_choice: side_one_options[1],
                        total_score: 4.0,
                        visits: 5,
                    },
                ],
                s2: vec![
                    MctsSideResult {
                        move_choice: side_two_options[0],
                        total_score: 6.0,
                        visits: 7,
                    },
                    MctsSideResult {
                        move_choice: side_two_options[1],
                        total_score: 8.0,
                        visits: 9,
                    },
                ],
                iteration_count: 20,
            },
        ];

        let merged = merge_mcts_results(results, side_one_options, side_two_options);

        assert_eq!(merged.iteration_count, 30);
        assert_eq!(
            merged.s1[0].move_choice,
            MoveChoice::Move(PokemonMoveIndex::M0)
        );
        assert_eq!(
            merged.s1[1].move_choice,
            MoveChoice::Move(PokemonMoveIndex::M1)
        );
        assert_eq!(
            merged.s2[0].move_choice,
            MoveChoice::Move(PokemonMoveIndex::M2)
        );
        assert_eq!(merged.s2[1].move_choice, MoveChoice::None);
        assert_eq!(merged.s1[0].total_score, 4.0);
        assert_eq!(merged.s1[0].visits, 5);
        assert_eq!(merged.s1[1].total_score, 7.0);
        assert_eq!(merged.s1[1].visits, 9);
        assert_eq!(merged.s2[0].total_score, 11.0);
        assert_eq!(merged.s2[0].visits, 13);
        assert_eq!(merged.s2[1].total_score, 15.0);
        assert_eq!(merged.s2[1].visits, 17);
    }
}
