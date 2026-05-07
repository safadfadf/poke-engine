use super::{
    mcts_worker_count, sigmoid, MctsResult, MctsSideResult, MCTS_DAMAGE_BRANCH_DEPTH,
    MCTS_MAX_ITERATIONS_PER_TREE,
};
use crate::engine::evaluate::evaluate;
use crate::engine::generate_instructions::generate_instructions_from_move_pair;
use crate::engine::state::MoveChoice;
use crate::instruction::StateInstructions;
use crate::state::State;
use rand::prelude::*;
use rand::rng;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

const SCORE_SCALE: f32 = 1_000_000.0;
const VIRTUAL_LOSS_VISITS: u32 = 3;
const SHARED_MCTS_JOB_ITERATIONS: u32 = 128;

struct SharedMoveNode {
    move_choice: MoveChoice,
    total_score: AtomicI64,
    visits: AtomicU32,
    virtual_visits: AtomicU32,
}

impl SharedMoveNode {
    fn new(move_choice: MoveChoice) -> Self {
        Self {
            move_choice,
            total_score: AtomicI64::new(0),
            visits: AtomicU32::new(0),
            virtual_visits: AtomicU32::new(0),
        }
    }

    #[inline]
    fn add_virtual_loss(&self) {
        self.virtual_visits
            .fetch_add(VIRTUAL_LOSS_VISITS, Ordering::AcqRel);
    }

    #[inline]
    fn remove_virtual_loss(&self) {
        self.virtual_visits
            .fetch_sub(VIRTUAL_LOSS_VISITS, Ordering::AcqRel);
    }

    #[inline]
    fn add_result(&self, score: f32) {
        self.total_score
            .fetch_add((score * SCORE_SCALE).round() as i64, Ordering::AcqRel);
        self.visits.fetch_add(1, Ordering::AcqRel);
    }

    #[inline]
    fn total_score_f32(&self) -> f32 {
        self.total_score.load(Ordering::Acquire) as f32 / SCORE_SCALE
    }

    #[inline]
    fn ucb1_with_parent_exploration(&self, parent_exploration_numerator: f32) -> f32 {
        let visits = self.visits.load(Ordering::Acquire);
        let virtual_visits = self.virtual_visits.load(Ordering::Acquire);
        if visits == 0 && virtual_visits == 0 {
            return f32::INFINITY;
        }

        let effective_visits = visits.saturating_add(virtual_visits).max(1);
        let total_score = self.total_score_f32();
        let average_score = total_score / effective_visits as f32;
        average_score + (parent_exploration_numerator / effective_visits as f32).sqrt()
    }
}

struct SharedNodeOptions {
    s1: Vec<SharedMoveNode>,
    s2: Vec<SharedMoveNode>,
}

impl SharedNodeOptions {
    fn new(s1_options: Vec<MoveChoice>, s2_options: Vec<MoveChoice>) -> Self {
        Self {
            s1: s1_options.into_iter().map(SharedMoveNode::new).collect(),
            s2: s2_options.into_iter().map(SharedMoveNode::new).collect(),
        }
    }
}

struct SharedNodeChildren {
    s2_len: usize,
    entries: Vec<OnceLock<SharedBranch>>,
}

impl SharedNodeChildren {
    fn new(s1_len: usize, s2_len: usize) -> Self {
        let mut entries = Vec::with_capacity(s1_len.saturating_mul(s2_len));
        entries.resize_with(s1_len.saturating_mul(s2_len), OnceLock::new);
        Self { s2_len, entries }
    }

    #[inline]
    fn index(&self, s1_index: usize, s2_index: usize) -> usize {
        s1_index * self.s2_len + s2_index
    }

    #[inline]
    fn get(&self, s1_index: usize, s2_index: usize) -> Option<&SharedBranch> {
        self.entries
            .get(self.index(s1_index, s2_index))
            .and_then(|entry| entry.get())
    }

    #[inline]
    fn entry(&self, s1_index: usize, s2_index: usize) -> &OnceLock<SharedBranch> {
        let index = self.index(s1_index, s2_index);
        &self.entries[index]
    }
}

struct SharedBranch {
    nodes: Vec<Arc<SharedNode>>,
    total_weight: f32,
}

impl SharedBranch {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Arc<SharedNode> {
        if self.nodes.len() <= 1 || self.total_weight <= 0.0 {
            return self.nodes[0].clone();
        }

        let mut threshold = rng.random_range(0.0..self.total_weight);
        for node in &self.nodes {
            threshold -= node.instructions.percentage.max(0.0);
            if threshold <= 0.0 {
                return node.clone();
            }
        }
        self.nodes[self.nodes.len() - 1].clone()
    }
}

struct SharedNode {
    root: bool,
    instructions: StateInstructions,
    depth: u8,
    times_visited: AtomicU32,
    virtual_losses: AtomicI32,
    eval: OnceLock<f32>,
    options: OnceLock<SharedNodeOptions>,
    children: OnceLock<SharedNodeChildren>,
}

impl SharedNode {
    fn new_root(side_one_options: Vec<MoveChoice>, side_two_options: Vec<MoveChoice>) -> Arc<Self> {
        let node = Arc::new(Self {
            root: true,
            instructions: StateInstructions::default(),
            depth: 0,
            times_visited: AtomicU32::new(0),
            virtual_losses: AtomicI32::new(0),
            eval: OnceLock::new(),
            options: OnceLock::new(),
            children: OnceLock::new(),
        });
        let _ = node
            .options
            .set(SharedNodeOptions::new(side_one_options, side_two_options));
        node
    }

    fn new_child(
        instructions: StateInstructions,
        _s1_choice: usize,
        _s2_choice: usize,
        depth: u8,
    ) -> Arc<Self> {
        Arc::new(Self {
            root: false,
            instructions,
            depth,
            times_visited: AtomicU32::new(0),
            virtual_losses: AtomicI32::new(0),
            eval: OnceLock::new(),
            options: OnceLock::new(),
            children: OnceLock::new(),
        })
    }

    fn ensure_options(&self, state: &State) -> &SharedNodeOptions {
        self.options.get_or_init(|| {
            let (s1_options, s2_options) = state.get_all_options();
            SharedNodeOptions::new(s1_options, s2_options)
        })
    }

    fn maximize_ucb_for_side(&self, side_options: &[SharedMoveNode]) -> usize {
        let mut choice = 0;
        let mut best_ucb1 = f32::MIN;
        let parent_visits = self
            .times_visited
            .load(Ordering::Acquire)
            .saturating_add(self.virtual_losses.load(Ordering::Acquire).max(0) as u32)
            .max(1);
        let parent_exploration_numerator = 2.0 * (parent_visits as f32).ln().max(0.0);

        for (index, node) in side_options.iter().enumerate() {
            let this_ucb1 = node.ucb1_with_parent_exploration(parent_exploration_numerator);
            if this_ucb1 > best_ucb1 {
                best_ucb1 = this_ucb1;
                choice = index;
            }
        }
        choice
    }

    fn select_move_pair(&self, state: &State) -> Option<(usize, usize)> {
        let options = self.ensure_options(state);
        if options.s1.is_empty() || options.s2.is_empty() {
            return None;
        }
        Some((
            self.maximize_ucb_for_side(&options.s1),
            self.maximize_ucb_for_side(&options.s2),
        ))
    }

    fn children(&self) -> &SharedNodeChildren {
        let options = self
            .options
            .get()
            .expect("node options must be initialized");
        self.children
            .get_or_init(|| SharedNodeChildren::new(options.s1.len(), options.s2.len()))
    }

    fn sample_existing_child<R: Rng + ?Sized>(
        &self,
        s1_index: usize,
        s2_index: usize,
        rng: &mut R,
    ) -> Option<Arc<SharedNode>> {
        let children = self.children.get()?;
        children
            .get(s1_index, s2_index)
            .map(|branch| branch.sample(rng))
    }

    fn expand_and_sample_child<R: Rng + ?Sized>(
        &self,
        state: &mut State,
        s1_index: usize,
        s2_index: usize,
        rng: &mut R,
    ) -> Option<Arc<SharedNode>> {
        let children = self.children();
        if let Some(branch) = children.get(s1_index, s2_index) {
            return Some(branch.sample(rng));
        }

        let options = self
            .options
            .get()
            .expect("node options must be initialized");
        let s1_move = &options.s1[s1_index].move_choice;
        let s2_move = &options.s2[s2_index].move_choice;

        if (state.battle_is_over() != 0.0 && !self.root)
            || (s1_move == &MoveChoice::None && s2_move == &MoveChoice::None)
        {
            return None;
        }

        let should_branch_on_damage = self.depth < MCTS_DAMAGE_BRANCH_DEPTH;
        let instructions =
            generate_instructions_from_move_pair(state, s1_move, s2_move, should_branch_on_damage);
        if instructions.is_empty() {
            return None;
        }

        let nodes = instructions
            .into_iter()
            .map(|state_instructions| {
                SharedNode::new_child(
                    state_instructions,
                    s1_index,
                    s2_index,
                    self.depth.saturating_add(1),
                )
            })
            .collect::<Vec<_>>();
        let total_weight = nodes
            .iter()
            .map(|node| node.instructions.percentage.max(0.0))
            .sum();
        let branch = SharedBranch {
            nodes,
            total_weight,
        };
        let sampled = branch.sample(rng);
        let entry = children.entry(s1_index, s2_index);
        if entry.set(branch).is_err() {
            return entry
                .get()
                .map(|existing_branch| existing_branch.sample(rng));
        }
        Some(sampled)
    }
}

struct PathStep {
    parent: Arc<SharedNode>,
    child: Arc<SharedNode>,
    s1_index: usize,
    s2_index: usize,
}

fn rollout(node: &SharedNode, state: &State, root_eval: f32) -> f32 {
    let battle_is_over = state.battle_is_over();
    if battle_is_over == 0.0 {
        let eval = *node.eval.get_or_init(|| evaluate(state));
        sigmoid(eval - root_eval)
    } else if battle_is_over == -1.0 {
        0.0
    } else {
        battle_is_over
    }
}

fn reverse_path(state: &mut State, path: &[PathStep]) {
    for step in path.iter().rev() {
        state.reverse_instructions(&step.child.instructions.instruction_list);
    }
}

fn remove_virtual_losses(path: &[PathStep]) {
    for step in path {
        if let Some(options) = step.parent.options.get() {
            options.s1[step.s1_index].remove_virtual_loss();
            options.s2[step.s2_index].remove_virtual_loss();
        }
        step.child.virtual_losses.fetch_sub(1, Ordering::AcqRel);
    }
}

fn backpropagate(path: &[PathStep], leaf: &Arc<SharedNode>, score: f32) {
    leaf.times_visited.fetch_add(1, Ordering::AcqRel);

    for step in path.iter().rev() {
        let options = step.parent.options.get().expect("path parent has options");
        options.s1[step.s1_index].add_result(score);
        options.s2[step.s2_index].add_result(1.0 - score);
        step.parent.times_visited.fetch_add(1, Ordering::AcqRel);
    }
}

fn do_shared_tree_playout<R: Rng + ?Sized>(
    root: &Arc<SharedNode>,
    state: &mut State,
    root_eval: f32,
    rng: &mut R,
) {
    let mut path = Vec::with_capacity(16);
    let mut current = root.clone();

    loop {
        if current.depth > 0 && state.battle_is_over() != 0.0 {
            break;
        }

        let Some((s1_index, s2_index)) = current.select_move_pair(state) else {
            break;
        };
        let options = current.options.get().expect("selected node has options");
        options.s1[s1_index].add_virtual_loss();
        options.s2[s2_index].add_virtual_loss();

        let child =
            if let Some(existing_child) = current.sample_existing_child(s1_index, s2_index, rng) {
                Some(existing_child)
            } else {
                current.expand_and_sample_child(state, s1_index, s2_index, rng)
            };

        let Some(child) = child else {
            options.s1[s1_index].remove_virtual_loss();
            options.s2[s2_index].remove_virtual_loss();
            break;
        };

        child.virtual_losses.fetch_add(1, Ordering::AcqRel);
        state.apply_instructions(&child.instructions.instruction_list);
        path.push(PathStep {
            parent: current.clone(),
            child: child.clone(),
            s1_index,
            s2_index,
        });

        let child_was_new_leaf = child.times_visited.load(Ordering::Acquire) == 0;
        current = child;
        if child_was_new_leaf {
            break;
        }
    }

    let score = rollout(&current, state, root_eval);
    backpropagate(&path, &current, score);
    remove_virtual_losses(&path);
    reverse_path(state, &path);
}

type SharedMctsJob = Box<dyn FnOnce() + Send + 'static>;

struct SharedMctsJobQueue {
    jobs: Mutex<VecDeque<SharedMctsJob>>,
    available: Condvar,
}

impl SharedMctsJobQueue {
    fn new() -> Self {
        Self {
            jobs: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
        }
    }

    fn submit(&self, job: SharedMctsJob) {
        let mut jobs = self.jobs.lock().expect("shared MCTS job queue poisoned");
        jobs.push_back(job);
        self.available.notify_one();
    }

    fn recv(&self) -> SharedMctsJob {
        let mut jobs = self.jobs.lock().expect("shared MCTS job queue poisoned");
        loop {
            if let Some(job) = jobs.pop_front() {
                return job;
            }
            jobs = self
                .available
                .wait(jobs)
                .expect("shared MCTS job queue poisoned");
        }
    }
}

struct SharedMctsWorkerPool {
    queue: Arc<SharedMctsJobQueue>,
    workers: usize,
}

impl SharedMctsWorkerPool {
    fn new() -> Self {
        Self {
            queue: Arc::new(SharedMctsJobQueue::new()),
            workers: 0,
        }
    }

    fn ensure_workers(&mut self, worker_count: usize) {
        while self.workers < worker_count {
            let queue = self.queue.clone();
            let worker_index = self.workers;
            thread::Builder::new()
                .name(format!("poke-engine-shared-mcts-{worker_index}"))
                .spawn(move || loop {
                    let job = queue.recv();
                    job();
                })
                .expect("failed to spawn shared MCTS worker");
            self.workers += 1;
        }
    }
}

fn shared_mcts_worker_pool() -> &'static Mutex<SharedMctsWorkerPool> {
    static POOL: OnceLock<Mutex<SharedMctsWorkerPool>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(SharedMctsWorkerPool::new()))
}

struct SharedSearchLane {
    root: Arc<SharedNode>,
    state: Mutex<State>,
    started_iterations: Arc<AtomicU32>,
    deadline: Instant,
    max_iterations: u32,
    root_eval: f32,
    job_queue: Arc<SharedMctsJobQueue>,
    done_sender: mpsc::Sender<bool>,
}

fn schedule_shared_search_lane(lane: Arc<SharedSearchLane>) {
    let job_queue = lane.job_queue.clone();
    job_queue.submit(Box::new(move || run_shared_search_lane(lane)));
}

fn run_shared_search_lane(lane: Arc<SharedSearchLane>) {
    let mut finished = false;
    let mut reschedule = false;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if Instant::now() >= lane.deadline {
            finished = true;
            return;
        }

        {
            let mut state = lane
                .state
                .lock()
                .expect("shared MCTS lane state mutex poisoned");
            let mut rng = rng();
            let mut remaining = SHARED_MCTS_JOB_ITERATIONS;
            while remaining > 0 {
                let iteration = lane.started_iterations.fetch_add(1, Ordering::AcqRel);
                if iteration >= lane.max_iterations {
                    finished = true;
                    break;
                }

                do_shared_tree_playout(&lane.root, &mut state, lane.root_eval, &mut rng);
                remaining -= 1;
            }
        }

        if !finished {
            if Instant::now() >= lane.deadline {
                finished = true;
            } else {
                reschedule = true;
            }
        }
    }));

    if result.is_err() {
        let _ = lane.done_sender.send(false);
    } else if finished {
        let _ = lane.done_sender.send(true);
    } else if reschedule {
        schedule_shared_search_lane(lane);
    }
}

fn shared_tree_drop_sender() -> &'static mpsc::Sender<Arc<SharedNode>> {
    static DROP_SENDER: OnceLock<mpsc::Sender<Arc<SharedNode>>> = OnceLock::new();
    DROP_SENDER.get_or_init(|| {
        let (sender, receiver) = mpsc::channel::<Arc<SharedNode>>();
        thread::Builder::new()
            .name("poke-engine-shared-mcts-drop".to_string())
            .spawn(move || {
                while let Ok(root) = receiver.recv() {
                    drop(root);
                }
            })
            .expect("failed to spawn shared MCTS tree drop thread");
        sender
    })
}

fn drop_finished_shared_tree(root: Arc<SharedNode>) {
    shared_tree_drop_sender()
        .send(root)
        .expect("shared MCTS tree drop worker stopped unexpectedly");
}

fn should_use_shared_tree_mcts() -> bool {
    !matches!(
        std::env::var("POKE_ENGINE_MCTS_SHARED_TREE").as_deref(),
        Ok("0") | Ok("false") | Ok("FALSE") | Ok("no") | Ok("NO") | Ok("off") | Ok("OFF")
    )
}

pub(super) fn shared_tree_enabled() -> bool {
    should_use_shared_tree_mcts()
}

pub(super) fn perform_mcts_shared_tree(
    state: &mut State,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    max_time: Duration,
    root_eval: f32,
) -> MctsResult {
    let worker_count = mcts_worker_count();
    let deadline = Instant::now() + max_time;
    let root = SharedNode::new_root(side_one_options, side_two_options);
    let started_iterations = Arc::new(AtomicU32::new(0));
    let max_iterations = MCTS_MAX_ITERATIONS_PER_TREE;

    let queue = {
        let mut pool = shared_mcts_worker_pool()
            .lock()
            .expect("shared MCTS worker pool mutex poisoned");
        pool.ensure_workers(worker_count);
        pool.queue.clone()
    };
    let (done_sender, done_receiver) = mpsc::channel();
    for _ in 0..worker_count {
        schedule_shared_search_lane(Arc::new(SharedSearchLane {
            root: root.clone(),
            state: Mutex::new(state.clone()),
            started_iterations: started_iterations.clone(),
            deadline,
            max_iterations,
            root_eval,
            job_queue: queue.clone(),
            done_sender: done_sender.clone(),
        }));
    }
    drop(done_sender);
    for worker_result in done_receiver.iter().take(worker_count) {
        if !worker_result {
            panic!("shared MCTS worker thread panicked");
        }
    }

    let result = {
        let options = root.options.get().expect("root options initialized");
        MctsResult {
            s1: options
                .s1
                .iter()
                .map(|v| MctsSideResult {
                    move_choice: v.move_choice,
                    total_score: v.total_score_f32(),
                    visits: v.visits.load(Ordering::Acquire),
                })
                .collect(),
            s2: options
                .s2
                .iter()
                .map(|v| MctsSideResult {
                    move_choice: v.move_choice,
                    total_score: v.total_score_f32(),
                    visits: v.visits.load(Ordering::Acquire),
                })
                .collect(),
            iteration_count: root.times_visited.load(Ordering::Acquire),
        }
    };
    drop_finished_shared_tree(root);
    result
}
