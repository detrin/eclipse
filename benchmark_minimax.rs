// Simple benchmarking script for the minimax engine
// Run with: cargo run --release --bin benchmark_minimax

use eclipse::minimaxbot::{MinimaxBot, Difficulty};
use eclipse::states::{GameState, Player};
use eclipse::bot::Bot;
use std::time::Instant;

fn main() {
    println!("=== Eclipse Minimax Engine Benchmarking ===\n");

    // Create a fresh game state
    let game = GameState::new();

    // Test each difficulty level
    for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        println!("Testing {:?} difficulty (depth {})...", difficulty, difficulty.depth());

        let bot = MinimaxBot::new(Player::Dark, difficulty);

        // Warm-up run
        let _ = bot.choose_move(&game);

        // Timed runs
        const RUNS: usize = 5;
        let mut times = Vec::new();

        for run in 1..=RUNS {
            let start = Instant::now();
            let _move = bot.choose_move(&game);
            let elapsed = start.elapsed();
            times.push(elapsed.as_secs_f64());
            println!("  Run {}: {:.3}s", run, elapsed.as_secs_f64());
        }

        let avg_time = times.iter().sum::<f64>() / times.len() as f64;
        let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_time = times.iter().fold(0.0f64, |a, &b| a.max(b));

        println!("  Average: {:.3}s", avg_time);
        println!("  Min: {:.3}s, Max: {:.3}s", min_time, max_time);

        // Estimate nodes per second (rough estimate based on branching factor)
        // Typical branching factor is ~10-20 moves per position
        let branching_factor = 15.0f64;
        let depth = difficulty.depth() as f64;
        let estimated_nodes = branching_factor.powf(depth);
        let nodes_per_sec = estimated_nodes / avg_time;

        println!("  Estimated ~{:.0} nodes searched", estimated_nodes);
        println!("  ~{:.0} nodes/second", nodes_per_sec);
        println!();
    }

    println!("\n=== Profiling Specific Operations ===\n");

    // Profile move generation
    println!("Move generation:");
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = game.get_legal_moves();
    }
    let elapsed = start.elapsed();
    println!("  10,000 calls: {:.3}s ({:.1}µs per call)",
             elapsed.as_secs_f64(),
             elapsed.as_micros() as f64 / 10000.0);

    // Profile position evaluation (need to create a bot instance)
    let bot = MinimaxBot::new(Player::Dark, Difficulty::Medium);
    println!("\nPosition evaluation:");
    println!("  (Note: evaluate_position is private - use choose_move for full benchmark)");

    // Profile game state cloning
    println!("\nGameState cloning:");
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = game.clone();
    }
    let elapsed = start.elapsed();
    println!("  10,000 clones: {:.3}s ({:.1}µs per clone)",
             elapsed.as_secs_f64(),
             elapsed.as_micros() as f64 / 10000.0);

    println!("\n=== Bottleneck Analysis ===");
    println!("If minimax is slow, typical causes:");
    println!("1. Move generation (get_legal_moves) - see timing above");
    println!("2. Position evaluation - complex evaluation function");
    println!("3. State cloning - see timing above (mitigated by undo/redo)");
    println!("4. Transposition table lookups - hash computation + HashMap ops");
    println!("5. Branching factor too high - more pruning needed");
    println!("\nUse 'cargo flamegraph' for detailed profiling:");
    println!("  cargo install flamegraph");
    println!("  cargo flamegraph --bin benchmark_minimax");
}
