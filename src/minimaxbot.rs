use crate::bot::Bot;
use crate::moves::Move;
use crate::states::{GameState, GameStatus, Player};
use std::collections::HashMap;
use std::cell::RefCell;

// =============================================================================
// TRANSPOSITION TABLE
// =============================================================================

/// Type of bound stored in a transposition table entry.
///
/// During alpha-beta search, we don't always get exact scores:
/// - Exact: The score is the true minimax value
/// - LowerBound: Score >= stored value (beta cutoff, failed high)
/// - UpperBound: Score <= stored value (alpha cutoff, failed low)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryType {
    /// The stored score is the exact minimax value for this position
    Exact,
    /// The stored score is a lower bound (actual score >= this value)
    LowerBound,
    /// The stored score is an upper bound (actual score <= this value)
    UpperBound,
}

/// Entry in the transposition table storing evaluation results.
///
/// Stores the result of evaluating a position at a certain depth,
/// allowing us to skip re-evaluation if we encounter the same position again.
#[derive(Debug, Clone)]
struct TranspositionEntry {
    /// Search depth at which this position was evaluated
    depth: usize,
    /// Evaluated score for this position
    score: f64,
    /// Type of bound (exact, lower, or upper)
    flag: EntryType,
}

// =============================================================================
// MINIMAX BOT IMPLEMENTATION
// =============================================================================

/// Difficulty levels for the MinimaxBot.
///
/// Each difficulty level uses different search depths and evaluation parameters:
/// - Easy: Depth 2, limited evaluation
/// - Medium: Depth 3, standard evaluation
/// - Hard: Depth 4, comprehensive evaluation
/// - VeryHard: Depth 5, deep search
/// - Expert: Depth 6, very deep search
/// - Master: Depth 7, maximum search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
    Expert,
    Master,
}

impl Difficulty {
    /// Returns the search depth for this difficulty level.
    pub fn depth(&self) -> usize {
        match self {
            Difficulty::Easy => 2,
            Difficulty::Medium => 3,
            Difficulty::Hard => 4,
            Difficulty::VeryHard => 5,
            Difficulty::Expert => 6,
            Difficulty::Master => 7,
        }
    }

    /// Returns the evaluation weight multiplier for this difficulty.
    /// Higher difficulties use more sophisticated evaluation.
    pub fn evaluation_weight(&self) -> f64 {
        match self {
            Difficulty::Easy => 0.7,      // Simplified evaluation
            Difficulty::Medium => 1.0,    // Standard evaluation
            Difficulty::Hard => 1.3,      // Enhanced evaluation
            Difficulty::VeryHard => 1.5,  // Deep search evaluation
            Difficulty::Expert => 1.5,    // Very deep search evaluation
            Difficulty::Master => 1.5,    // Maximum search evaluation
        }
    }
}

/// Advanced bot that uses minimax algorithm with alpha-beta pruning.
///
/// This bot evaluates game positions by looking ahead several moves and
/// choosing the move that leads to the best outcome, assuming optimal play
/// from both players.
///
/// Features:
/// - Minimax algorithm with alpha-beta pruning for efficient search
/// - Configurable search depth (2-4 moves ahead)
/// - Comprehensive position evaluation function
/// - Multiple difficulty levels
/// - Transposition table to cache evaluated positions
/// - Quiescence search to avoid horizon effect
#[derive(Debug)]
pub struct MinimaxBot {
    player: Player,
    difficulty: Difficulty,
    /// Transposition table for caching position evaluations
    /// Maps position hash -> evaluation entry
    /// Uses RefCell for interior mutability during search
    transposition_table: RefCell<HashMap<u64, TranspositionEntry>>,
    /// Maximum number of entries in transposition table (for memory limit)
    max_table_size: usize,
    /// Maximum depth for quiescence search to prevent explosion
    max_quiescence_depth: usize,
    /// Killer moves for move ordering heuristic
    /// Maps depth -> [best_killer, second_best_killer]
    /// Stores moves that caused beta cutoffs at each depth
    killer_moves: RefCell<HashMap<usize, [Option<Move>; 2]>>,
}

impl MinimaxBot {
    /// Creates a new MinimaxBot for the specified player and difficulty.
    ///
    /// # Arguments
    /// * `player` - The player this bot will play as (Light or Dark)
    /// * `difficulty` - The difficulty level (Easy, Medium, Hard)
    ///
    /// # Example
    /// ```
    /// use eclipse::minimaxbot::{MinimaxBot, Difficulty};
    /// use eclipse::states::Player;
    ///
    /// let bot = MinimaxBot::new(Player::Dark, Difficulty::Hard);
    /// ```
    pub fn new(player: Player, difficulty: Difficulty) -> Self {
        MinimaxBot {
            player,
            difficulty,
            transposition_table: RefCell::new(HashMap::new()),
            max_table_size: 100_000, // ~2-3MB memory limit
            max_quiescence_depth: 1, // Limit quiescence to 1 additional ply
            killer_moves: RefCell::new(HashMap::new()),
        }
    }

    /// Returns the player this bot represents.
    pub fn player(&self) -> Player {
        self.player
    }

    /// Returns the difficulty level of this bot.
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    /// Evaluates the current game state from the perspective of this bot.
    ///
    /// The evaluation function considers multiple factors:
    /// - Mobility: Number of legal moves available for each player
    /// - Comet safety: Distance from opponent's comet and threats
    /// - Chain control: Number of active vs blocked chains
    /// - Comet positioning: Penalize comet being cornered
    ///
    /// # Returns
    /// A score value where:
    /// - Positive values favor this bot's player
    /// - Negative values favor the opponent
    /// - Higher absolute values indicate stronger positions
    fn evaluate_position(&self, game: &GameState) -> f64 {
        // Check for terminal positions (win/loss)
        match game.status {
            GameStatus::Won(winner) => {
                if winner == self.player {
                    return 10000.0; // We won
                } else {
                    return -10000.0; // We lost
                }
            }
            GameStatus::InProgress => {}
        }

        let mut score = 0.0;
        let weight = self.difficulty.evaluation_weight();

        // FACTOR 1: Mobility (legal moves count)
        // More moves = better position
        score += self.evaluate_mobility(game) * 50.0 * weight;

        // FACTOR 2: Comet safety
        // Distance from opponent's comet and threats
        score += self.evaluate_comet_safety(game) * 30.0 * weight;

        // FACTOR 3: Chain control
        // Active chains vs blocked chains
        score += self.evaluate_chain_control(game) * 40.0 * weight;

        // FACTOR 4: Comet position quality
        // Penalize being cornered or trapped
        score += self.evaluate_comet_position_quality(game) * 25.0 * weight;

        score
    }

    /// Evaluates mobility: difference in legal move counts between players.
    ///
    /// # Returns
    /// Positive if we have more moves, negative if opponent has more moves
    fn evaluate_mobility(&self, game: &GameState) -> f64 {
        // Count our moves
        let mut temp_game = game.clone();
        temp_game.current_turn = self.player;
        let our_moves = temp_game.get_legal_moves().len() as f64;

        // Count opponent's moves
        temp_game.current_turn = self.player.opponent();
        let opponent_moves = temp_game.get_legal_moves().len() as f64;

        // Return the difference (normalized)
        (our_moves - opponent_moves) / (our_moves + opponent_moves + 1.0)
    }

    /// Evaluates comet safety based on distance from threats.
    ///
    /// # Returns
    /// Positive if our comet is safer, negative if opponent's is safer
    fn evaluate_comet_safety(&self, game: &GameState) -> f64 {
        let our_comet = if self.player == Player::Light {
            game.comet_light
        } else {
            game.comet_dark
        };

        let opponent_comet = if self.player == Player::Light {
            game.comet_dark
        } else {
            game.comet_light
        };

        // Distance between comets (further is safer)
        let comet_distance = our_comet.distance(&opponent_comet) as f64;

        // Count adjacent empty spaces (more is safer)
        let our_escape_routes = our_comet.neighbors()
            .iter()
            .filter(|hex| hex.is_on_board() && !game.occupied.contains_key(hex))
            .count() as f64;

        let opponent_escape_routes = opponent_comet.neighbors()
            .iter()
            .filter(|hex| hex.is_on_board() && !game.occupied.contains_key(hex))
            .count() as f64;

        // Combine factors
        let safety_score = (our_escape_routes - opponent_escape_routes) / 6.0;
        let distance_score = comet_distance / 10.0;

        safety_score + distance_score
    }

    /// Evaluates chain control: active vs blocked chains.
    ///
    /// # Returns
    /// Positive if we have more active chains, negative otherwise
    fn evaluate_chain_control(&self, game: &GameState) -> f64 {
        let mut our_active = 0;
        let mut our_blocked = 0;
        let mut opponent_active = 0;
        let mut opponent_blocked = 0;

        for (chain_id, chain) in &game.chains {
            let is_blocked = game.is_chain_immobilized_external(*chain_id);

            if chain.owner == self.player {
                if is_blocked {
                    our_blocked += 1;
                } else {
                    our_active += 1;
                }
            } else {
                if is_blocked {
                    opponent_blocked += 1;
                } else {
                    opponent_active += 1;
                }
            }
        }

        // Active chains are good, blocked opponent chains are good
        let our_control = our_active as f64 - our_blocked as f64;
        let opponent_control = opponent_active as f64 - opponent_blocked as f64;

        // Blocking opponent chains is valuable
        let blocking_bonus = opponent_blocked as f64 * 0.5;

        our_control - opponent_control + blocking_bonus
    }

    /// Evaluates comet position quality (penalize being cornered).
    ///
    /// # Returns
    /// Positive if our comet is well-positioned, negative otherwise
    fn evaluate_comet_position_quality(&self, game: &GameState) -> f64 {
        let our_comet = if self.player == Player::Light {
            game.comet_light
        } else {
            game.comet_dark
        };

        let opponent_comet = if self.player == Player::Light {
            game.comet_dark
        } else {
            game.comet_light
        };

        // Penalize being near the edge of the board
        let distance_from_center = our_comet.distance(&crate::board::Hex::new(0, 0)) as f64;
        let edge_penalty = -distance_from_center / 4.0;

        // Penalize being surrounded
        let blocked_neighbors = our_comet.neighbors()
            .iter()
            .filter(|hex| !hex.is_on_board() || game.occupied.contains_key(hex))
            .count() as f64;
        let surrounded_penalty = -blocked_neighbors / 2.0;

        // Reward being protected by own chains (between us and opponent)
        let protection_bonus = self.evaluate_chain_protection(game, our_comet, opponent_comet);

        edge_penalty + surrounded_penalty + protection_bonus
    }

    /// Evaluates how well chains protect our comet.
    fn evaluate_chain_protection(&self, game: &GameState, our_comet: crate::board::Hex, opponent_comet: crate::board::Hex) -> f64 {
        let mut protecting_chains = 0;

        for chain in game.chains.values() {
            if chain.owner == self.player {
                // Check if this chain is positioned between the two comets
                let chain_mid_q = (chain.head.q + chain.tail.q) as f64 / 2.0;
                let our_q = our_comet.q as f64;
                let opp_q = opponent_comet.q as f64;

                // Check if chain is between the two comets (in q-axis)
                if (our_q < opp_q && chain_mid_q > our_q && chain_mid_q < opp_q) ||
                   (our_q > opp_q && chain_mid_q < our_q && chain_mid_q > opp_q) {
                    protecting_chains += 1;
                }
            }
        }

        protecting_chains as f64 * 0.3
    }

    /// Orders moves to improve alpha-beta pruning efficiency.
    ///
    /// Better moves should be searched first to maximize pruning. This function
    /// orders moves by:
    /// 1. Comet moves first (highest impact - can win/lose the game)
    /// 2. Satellite moves second
    /// 3. Within each category, can add static evaluation if needed
    ///
    /// # Arguments
    /// * `moves` - Mutable reference to the move list to be sorted in-place
    /// * `depth` - Current search depth (for killer move lookup)
    fn order_moves(&self, moves: &mut Vec<Move>, depth: usize) {
        // Get killer moves for this depth
        let killers = self.killer_moves.borrow();
        let killer_moves = killers.get(&depth);

        moves.sort_by_key(|mv| {
            // Check if this move is a killer move
            if let Some(killers) = killer_moves {
                if killers[0].as_ref() == Some(mv) {
                    return -2; // First killer move (highest priority after best move hint)
                }
                if killers[1].as_ref() == Some(mv) {
                    return -1; // Second killer move
                }
            }

            // Default move ordering by type
            match mv {
                Move::MoveComet(_) => 0,  // Try comet moves first (game-changing)
                Move::MoveSatellite { .. } => 1,  // Then satellite moves
            }
        });
    }

    /// Generates tactical moves for quiescence search.
    ///
    /// Tactical moves in Eclipse are those that could create immediate threats:
    /// - Satellite moves that create new chain crossings with opponent chains
    ///
    /// This is a subset of all legal moves, focusing on non-quiet positions.
    /// Note: We exclude comet moves to keep quiescence search focused and fast.
    ///
    /// # Arguments
    /// * `game` - Current game state
    ///
    /// # Returns
    /// Vector of tactical moves
    fn get_tactical_moves(&self, game: &GameState) -> Vec<Move> {
        let all_moves = game.get_legal_moves();
        let mut tactical_moves = Vec::new();

        for mv in all_moves {
            match mv {
                Move::MoveComet(_) => {
                    // Skip comet moves in quiescence for performance
                    // (they're already considered in main search)
                    continue;
                }
                Move::MoveSatellite { chain_id, old_pos, new_pos } => {
                    // Check if this move would create a new crossing
                    let chain = &game.chains[&chain_id];

                    // Determine which end is moving and create temporary new chain position
                    let (temp_head, temp_tail) = if chain.head == old_pos {
                        (new_pos, chain.tail)
                    } else {
                        (chain.head, new_pos)
                    };

                    // Check if new position would cross any opponent chains
                    let opponent = self.player.opponent();
                    for other_chain in game.chains.values() {
                        if other_chain.owner == opponent {
                            // Check if moving this chain would create a crossing
                            let would_cross = {
                                use crate::board::segments_intersect;
                                let (a1, a2) = (temp_head.to_pixel(), temp_tail.to_pixel());
                                let (b1, b2) = (other_chain.head.to_pixel(), other_chain.tail.to_pixel());
                                segments_intersect(a1, a2, b1, b2)
                            };

                            if would_cross {
                                tactical_moves.push(mv);
                                break; // This move is tactical, no need to check more chains
                            }
                        }
                    }
                }
            }
        }

        tactical_moves
    }

    /// Quiescence search to avoid horizon effect.
    ///
    /// Continues searching tactical positions beyond the normal depth limit
    /// to avoid evaluating positions that look good but have immediate tactical refutations.
    ///
    /// Uses "stand-pat" approach: compares static evaluation with searching tactical moves,
    /// and can choose to stand pat (not search further) if the position is already good enough.
    ///
    /// # Arguments
    /// * `game` - Current game state
    /// * `alpha` - Alpha value for pruning
    /// * `beta` - Beta value for pruning
    /// * `qs_depth` - Current quiescence depth (for limiting search explosion)
    /// * `maximizing` - Whether this is a maximizing or minimizing node
    ///
    /// # Returns
    /// The quiescence score for this position
    fn quiescence_search(
        &self,
        game: &mut GameState,
        mut alpha: f64,
        mut beta: f64,
        qs_depth: usize,
        maximizing: bool,
    ) -> f64 {
        // Stand-pat: evaluate the current position without searching further
        let stand_pat = self.evaluate_position(game);

        // Stop if we've reached max quiescence depth
        if qs_depth >= self.max_quiescence_depth {
            return stand_pat;
        }

        // Generate only tactical moves
        let tactical_moves = self.get_tactical_moves(game);

        // If no tactical moves, position is quiet - return stand-pat
        if tactical_moves.is_empty() {
            return stand_pat;
        }

        if maximizing {
            // Beta cutoff: if standing pat is already too good, prune
            if stand_pat >= beta {
                return stand_pat;
            }

            // Update alpha with stand-pat if it's better
            let mut max_score = stand_pat;
            if stand_pat > alpha {
                alpha = stand_pat;
            }

            // Search tactical moves
            for mv in tactical_moves {
                if let Ok(undo_info) = game.apply_move_for_search(mv) {
                    let score = self.quiescence_search(game, alpha, beta, qs_depth + 1, false);

                    game.undo_move(undo_info);

                    if score > max_score {
                        max_score = score;
                    }

                    if score > alpha {
                        alpha = score;
                    }

                    // Beta cutoff
                    if alpha >= beta {
                        break;
                    }
                }
            }

            max_score
        } else {
            // Alpha cutoff: if standing pat is already too bad, prune
            if stand_pat <= alpha {
                return stand_pat;
            }

            // Update beta with stand-pat if it's better
            let mut min_score = stand_pat;
            if stand_pat < beta {
                beta = stand_pat;
            }

            // Search tactical moves
            for mv in tactical_moves {
                if let Ok(undo_info) = game.apply_move_for_search(mv) {
                    let score = self.quiescence_search(game, alpha, beta, qs_depth + 1, true);

                    game.undo_move(undo_info);

                    if score < min_score {
                        min_score = score;
                    }

                    if score < beta {
                        beta = score;
                    }

                    // Alpha cutoff
                    if alpha >= beta {
                        break;
                    }
                }
            }

            min_score
        }
    }

    /// Stores an evaluation result in the transposition table.
    ///
    /// # Arguments
    /// * `hash` - Position hash
    /// * `depth` - Search depth
    /// * `score` - Evaluated score
    /// * `original_alpha` - Alpha value at start of search
    /// * `beta` - Beta value at end of search
    fn store_transposition(&self, hash: u64, depth: usize, score: f64, original_alpha: f64, beta: f64) {
        // Determine entry type based on how the search concluded
        let flag = if score <= original_alpha {
            // Failed low - score is an upper bound
            EntryType::UpperBound
        } else if score >= beta {
            // Failed high - score is a lower bound
            EntryType::LowerBound
        } else {
            // Score is exact (within alpha-beta window)
            EntryType::Exact
        };

        let entry = TranspositionEntry { depth, score, flag };

        let mut table = self.transposition_table.borrow_mut();

        // Check if we should replace an existing entry
        let should_store = if let Some(existing) = table.get(&hash) {
            // Prefer deeper searches (depth-preferred replacement)
            depth >= existing.depth
        } else {
            true
        };

        if should_store {
            // Check table size limit
            if table.len() >= self.max_table_size && !table.contains_key(&hash) {
                // Table is full and this is a new entry - need to evict
                // Simple eviction: clear a portion of the table
                if table.len() >= self.max_table_size {
                    // Clear 25% of entries randomly when full
                    let keys_to_remove: Vec<u64> = table.keys()
                        .take(self.max_table_size / 4)
                        .copied()
                        .collect();
                    for key in keys_to_remove {
                        table.remove(&key);
                    }
                }
            }

            table.insert(hash, entry);
        }
    }

    /// Stores a killer move that caused a beta cutoff at the given depth.
    /// Killer moves are tried first in move ordering to increase cutoff rates.
    ///
    /// # Arguments
    /// * `depth` - The depth at which the cutoff occurred
    /// * `mv` - The move that caused the beta cutoff
    fn store_killer_move(&self, depth: usize, mv: Move) {
        let mut killers = self.killer_moves.borrow_mut();
        let entry = killers.entry(depth).or_insert([None, None]);

        // Don't store if it's already the first killer
        if entry[0].as_ref() == Some(&mv) {
            return;
        }

        // Shift killers: second becomes first, new move becomes second
        entry[1] = entry[0].clone();
        entry[0] = Some(mv);
    }

    /// Checks if a move is tactical (creates immediate threats).
    ///
    /// A tactical move is one that creates new chain crossings with opponent chains.
    /// Comet moves are not considered tactical for LMR purposes (they're always important).
    ///
    /// # Arguments
    /// * `game` - Current game state
    /// * `mv` - Move to check
    ///
    /// # Returns
    /// true if the move is tactical, false otherwise
    fn is_tactical_move(&self, game: &GameState, mv: &Move) -> bool {
        match mv {
            Move::MoveComet(_) => {
                // Comet moves are always important, treat as tactical
                true
            }
            Move::MoveSatellite { chain_id, old_pos, new_pos } => {
                // Check if this move would create a new crossing
                let chain = &game.chains[chain_id];

                // Determine which end is moving and create temporary new chain position
                let (temp_head, temp_tail) = if chain.head == *old_pos {
                    (*new_pos, chain.tail)
                } else {
                    (chain.head, *new_pos)
                };

                // Check if new position would cross any opponent chains
                let opponent = self.player.opponent();
                for other_chain in game.chains.values() {
                    if other_chain.owner == opponent {
                        // Check if moving this chain would create a crossing
                        let would_cross = {
                            use crate::board::segments_intersect;
                            let (a1, a2) = (temp_head.to_pixel(), temp_tail.to_pixel());
                            let (b1, b2) = (other_chain.head.to_pixel(), other_chain.tail.to_pixel());
                            segments_intersect(a1, a2, b1, b2)
                        };

                        if would_cross {
                            return true;
                        }
                    }
                }

                false
            }
        }
    }

    /// Implements the minimax algorithm with alpha-beta pruning.
    ///
    /// # Arguments
    /// * `game` - Current game state
    /// * `depth` - Remaining search depth
    /// * `alpha` - Alpha value for pruning
    /// * `beta` - Beta value for pruning
    /// * `maximizing` - Whether this is a maximizing or minimizing node
    /// * `null_move_allowed` - Whether null move pruning is allowed (prevents double null moves)
    ///
    /// # Returns
    /// The best score achievable from this position
    fn minimax(
        &self,
        game: &mut GameState,
        depth: usize,
        mut alpha: f64,
        mut beta: f64,
        maximizing: bool,
        null_move_allowed: bool,
    ) -> f64 {
        // Terminal condition: game over
        if game.status != GameStatus::InProgress {
            return self.evaluate_position(game);
        }

        // Terminal condition: depth limit reached
        if depth == 0 {
            // Use quiescence search only for Hard difficulty to balance speed and quality
            if self.difficulty == Difficulty::Hard {
                return self.quiescence_search(game, alpha, beta, 0, maximizing);
            } else {
                return self.evaluate_position(game);
            }
        }

        // Only use transposition table for depth >= 2 to avoid overhead for shallow searches
        let use_tt = depth >= 2;
        let position_hash = if use_tt { game.hash_position() } else { 0 };
        let original_alpha = alpha;

        // Check transposition table for cached evaluation
        if use_tt {
            let table = self.transposition_table.borrow();
            if let Some(entry) = table.get(&position_hash) {
                // Only use cached result if it was evaluated at >= current depth
                if entry.depth >= depth {
                    match entry.flag {
                        EntryType::Exact => {
                            // Exact score - use directly
                            return entry.score;
                        }
                        EntryType::LowerBound => {
                            // Score is at least this much
                            alpha = alpha.max(entry.score);
                        }
                        EntryType::UpperBound => {
                            // Score is at most this much
                            beta = beta.min(entry.score);
                        }
                    }
                    // Check if we can prune based on cached bounds
                    if alpha >= beta {
                        return entry.score;
                    }
                }
            }
        }

        // Null Move Pruning: Try giving opponent a free move at reduced depth
        // If even with a free move they can't improve enough to cause beta cutoff, prune
        // R value of 2 means we search 2 plies shallower
        const NULL_MOVE_R: usize = 2;

        if null_move_allowed
            && !maximizing  // Only try null move when minimizing (opponent's turn)
            && depth >= 3    // Don't use in shallow searches
            && beta < 9999.0 // Don't use when looking for checkmate
        {
            // Save current turn
            let saved_turn = game.current_turn;

            // Give opponent a "free move" by switching turns
            game.current_turn = game.current_turn.opponent();

            // Search at reduced depth with null move disallowed
            let null_score = self.minimax(
                game,
                depth.saturating_sub(NULL_MOVE_R + 1),
                alpha,
                beta,
                true,  // Next level is maximizing
                false, // Disallow null move to prevent double null moves
            );

            // Restore turn
            game.current_turn = saved_turn;

            // If null move causes beta cutoff, prune this branch
            if null_score >= beta {
                return beta;
            }
        }

        let mut legal_moves = game.get_legal_moves();

        // If no legal moves, evaluate terminal position
        if legal_moves.is_empty() {
            return self.evaluate_position(game);
        }

        // Order moves to improve alpha-beta pruning efficiency
        self.order_moves(&mut legal_moves, depth);

        if maximizing {
            let mut max_eval = f64::NEG_INFINITY;

            for (move_index, mv) in legal_moves.iter().enumerate() {
                // Clone the move for killer move storage (needed because apply_move_for_search takes ownership)
                let mv_clone = mv.clone();

                // Apply move with undo capability (avoids expensive clone)
                if let Ok(undo_info) = game.apply_move_for_search(mv_clone.clone()) {
                    let eval;

                    // Late Move Reduction (LMR)
                    // Search later moves at reduced depth first, then re-search if promising
                    if move_index >= 4 && depth >= 3 && !self.is_tactical_move(game, &mv) {
                        // Calculate reduction: more reduction for later moves
                        // Formula: 1 + (move_index / 8).min(2)
                        // This gives: moves 4-11 -> reduce by 1, moves 12-19 -> reduce by 2, moves 20+ -> reduce by 3
                        let reduction = 1 + (move_index / 8).min(2);
                        let reduced_depth = depth.saturating_sub(reduction + 1);

                        // Search at reduced depth first
                        let reduced_score = self.minimax(game, reduced_depth, alpha, beta, false, true);

                        // If the move looks promising (score > alpha), re-search at full depth
                        // Otherwise, use the reduced score
                        if reduced_score > alpha {
                            eval = self.minimax(game, depth - 1, alpha, beta, false, true);
                        } else {
                            eval = reduced_score;
                        }
                    } else {
                        // Always full search for first few moves and tactical moves
                        eval = self.minimax(game, depth - 1, alpha, beta, false, true);
                    }

                    max_eval = max_eval.max(eval);
                    alpha = alpha.max(eval);

                    // Undo move (efficient reversal)
                    game.undo_move(undo_info);

                    // Alpha-beta pruning
                    if beta <= alpha {
                        // Store killer move - this move caused a cutoff
                        self.store_killer_move(depth, mv_clone);
                        break;
                    }
                }
            }

            // Store result in transposition table (if enabled for this depth)
            if use_tt {
                self.store_transposition(position_hash, depth, max_eval, original_alpha, beta);
            }

            max_eval
        } else {
            let mut min_eval = f64::INFINITY;

            for (move_index, mv) in legal_moves.iter().enumerate() {
                // Clone the move for killer move storage (needed because apply_move_for_search takes ownership)
                let mv_clone = mv.clone();

                // Apply move with undo capability (avoids expensive clone)
                if let Ok(undo_info) = game.apply_move_for_search(mv_clone.clone()) {
                    let eval;

                    // Late Move Reduction (LMR)
                    // Search later moves at reduced depth first, then re-search if promising
                    if move_index >= 4 && depth >= 3 && !self.is_tactical_move(game, &mv) {
                        // Calculate reduction: more reduction for later moves
                        // Formula: 1 + (move_index / 8).min(2)
                        // This gives: moves 4-11 -> reduce by 1, moves 12-19 -> reduce by 2, moves 20+ -> reduce by 3
                        let reduction = 1 + (move_index / 8).min(2);
                        let reduced_depth = depth.saturating_sub(reduction + 1);

                        // Search at reduced depth first
                        let reduced_score = self.minimax(game, reduced_depth, alpha, beta, true, true);

                        // If the move looks promising (score < beta for minimizer), re-search at full depth
                        // Otherwise, use the reduced score
                        if reduced_score < beta {
                            eval = self.minimax(game, depth - 1, alpha, beta, true, true);
                        } else {
                            eval = reduced_score;
                        }
                    } else {
                        // Always full search for first few moves and tactical moves
                        eval = self.minimax(game, depth - 1, alpha, beta, true, true);
                    }

                    min_eval = min_eval.min(eval);
                    beta = beta.min(eval);

                    // Undo move (efficient reversal)
                    game.undo_move(undo_info);

                    // Alpha-beta pruning
                    if beta <= alpha {
                        // Store killer move - this move caused a cutoff
                        self.store_killer_move(depth, mv_clone);
                        break;
                    }
                }
            }

            // Store result in transposition table (if enabled for this depth)
            if use_tt {
                self.store_transposition(position_hash, depth, min_eval, original_alpha, beta);
            }

            min_eval
        }
    }

    /// Finds the best move at a specific search depth.
    ///
    /// # Arguments
    /// * `game` - Current game state
    /// * `depth` - Search depth to use
    /// * `best_move_hint` - Optional hint from previous iteration to prioritize
    ///
    /// # Returns
    /// Tuple of (best move, best score) or None if no legal moves
    fn find_best_move_at_depth(
        &self,
        game: &GameState,
        depth: usize,
        best_move_hint: Option<Move>,
    ) -> Option<(Move, f64)> {
        let mut legal_moves = game.get_legal_moves();

        if legal_moves.is_empty() {
            return None;
        }

        // Order moves to improve alpha-beta pruning efficiency
        self.order_moves(&mut legal_moves, depth);

        // If we have a best move from a previous iteration, prioritize it
        // by moving it to the front of the list (best move ordering)
        if let Some(hint) = best_move_hint {
            if let Some(pos) = legal_moves.iter().position(|m| *m == hint) {
                legal_moves.swap(0, pos);
            }
        }

        let mut best_move: Option<Move> = None;
        let mut best_score = f64::NEG_INFINITY;

        for mv in legal_moves {
            let mut temp_game = game.clone();

            if temp_game.apply_move(mv.clone()).is_ok() {
                // We're maximizing, so next level is minimizing
                let score = self.minimax(
                    &mut temp_game,
                    depth - 1,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                    false,
                    true, // Allow null move pruning
                );

                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);

                    // Early termination: if we found a winning move, stop searching
                    // A win score is >= 9999.0 (terminal positions return ±10000.0)
                    if best_score >= 9999.0 {
                        break;
                    }
                }
            }
        }

        best_move.map(|mv| (mv, best_score))
    }

    /// Finds the best move using iterative deepening with minimax.
    ///
    /// For shallow depths (<4), searches directly at target depth to avoid overhead.
    /// For deeper searches (>=4), uses iterative deepening: searches at progressively
    /// deeper depths, using results from previous iterations to improve move ordering.
    ///
    /// # Arguments
    /// * `game` - Current game state
    ///
    /// # Returns
    /// The best move found, or None if no legal moves
    fn find_best_move(&self, game: &GameState) -> Option<Move> {
        let max_depth = self.difficulty.depth();

        // Clear killer moves from previous searches
        self.killer_moves.borrow_mut().clear();

        // Only use iterative deepening for deep searches (depth >= 4)
        // For shallow searches, the overhead outweighs the benefit
        if max_depth < 4 {
            // Direct search at target depth
            return self.find_best_move_at_depth(game, max_depth, None).map(|(mv, _)| mv);
        }

        // Iterative deepening for depth >= 4
        let mut best_move: Option<Move> = None;

        // Start from depth 2 to save time (depth 1 doesn't provide useful ordering)
        for depth in 2..=max_depth {
            // Use previous iteration's best move as a hint for move ordering
            if let Some((mv, _score)) = self.find_best_move_at_depth(game, depth, best_move.clone()) {
                best_move = Some(mv);
            }
        }

        best_move
    }
}

impl Bot for MinimaxBot {
    fn choose_move(&self, game: &GameState) -> Option<Move> {
        self.find_best_move(game)
    }

    fn name(&self) -> &str {
        match self.difficulty {
            Difficulty::Easy => "MinimaxBot (Easy)",
            Difficulty::Medium => "MinimaxBot (Medium)",
            Difficulty::Hard => "MinimaxBot (Hard)",
            Difficulty::VeryHard => "MinimaxBot (Very Hard)",
            Difficulty::Expert => "MinimaxBot (Expert)",
            Difficulty::Master => "MinimaxBot (Master)",
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::GameState;

    #[test]
    fn test_minimax_bot_creation() {
        let bot = MinimaxBot::new(Player::Light, Difficulty::Easy);
        assert_eq!(bot.player(), Player::Light);
        assert_eq!(bot.difficulty(), Difficulty::Easy);
        assert_eq!(bot.name(), "MinimaxBot (Easy)");

        let bot = MinimaxBot::new(Player::Dark, Difficulty::Medium);
        assert_eq!(bot.player(), Player::Dark);
        assert_eq!(bot.difficulty(), Difficulty::Medium);
        assert_eq!(bot.name(), "MinimaxBot (Medium)");

        let bot = MinimaxBot::new(Player::Light, Difficulty::Hard);
        assert_eq!(bot.difficulty(), Difficulty::Hard);
        assert_eq!(bot.name(), "MinimaxBot (Hard)");
    }

    #[test]
    fn test_difficulty_levels() {
        assert_eq!(Difficulty::Easy.depth(), 2);
        assert_eq!(Difficulty::Medium.depth(), 3);
        assert_eq!(Difficulty::Hard.depth(), 4);

        assert_eq!(Difficulty::Easy.evaluation_weight(), 0.7);
        assert_eq!(Difficulty::Medium.evaluation_weight(), 1.0);
        assert_eq!(Difficulty::Hard.evaluation_weight(), 1.3);
    }

    #[test]
    fn test_minimax_bot_chooses_legal_move() {
        let game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Easy);

        let chosen_move = bot.choose_move(&game);
        assert!(chosen_move.is_some(), "MinimaxBot should choose a move when legal moves exist");

        // Verify the chosen move is legal
        let legal_moves = game.get_legal_moves();
        let chosen = chosen_move.unwrap();

        let is_valid = legal_moves.iter().any(|m| {
            match (&chosen, m) {
                (Move::MoveComet(pos1), Move::MoveComet(pos2)) => pos1 == pos2,
                (
                    Move::MoveSatellite { chain_id: id1, old_pos: old1, new_pos: new1 },
                    Move::MoveSatellite { chain_id: id2, old_pos: old2, new_pos: new2 },
                ) => id1 == id2 && old1 == old2 && new1 == new2,
                _ => false,
            }
        });

        assert!(is_valid, "MinimaxBot should choose a valid legal move");
    }

    #[test]
    fn test_evaluate_position() {
        let game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Medium);

        // Evaluate starting position
        let score = bot.evaluate_position(&game);

        // Score should be finite and reasonable
        assert!(score.is_finite(), "Evaluation score should be finite");
        assert!(score.abs() < 10000.0, "Starting position should not have terminal score");
    }

    #[test]
    fn test_evaluate_winning_position() {
        let mut game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Medium);

        // Manually set a win condition
        game.status = GameStatus::Won(Player::Light);

        let score = bot.evaluate_position(&game);
        assert!(score > 5000.0, "Winning position should have very high score");
    }

    #[test]
    fn test_evaluate_losing_position() {
        let mut game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Medium);

        // Manually set a loss condition
        game.status = GameStatus::Won(Player::Dark);

        let score = bot.evaluate_position(&game);
        assert!(score < -5000.0, "Losing position should have very low score");
    }

    #[test]
    fn test_evaluate_mobility() {
        let game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Medium);

        let mobility = bot.evaluate_mobility(&game);

        // At game start, mobility should be relatively balanced
        assert!(mobility.is_finite(), "Mobility should be finite");
        assert!(mobility.abs() < 2.0, "Starting mobility should be relatively balanced");
    }

    #[test]
    fn test_evaluate_comet_safety() {
        let game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Medium);

        let safety = bot.evaluate_comet_safety(&game);

        // Safety score should be finite
        assert!(safety.is_finite(), "Safety score should be finite");
    }

    #[test]
    fn test_evaluate_chain_control() {
        let game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Medium);

        let control = bot.evaluate_chain_control(&game);

        // At game start, no chains should be blocked
        assert!(control.is_finite(), "Chain control should be finite");
    }

    #[test]
    fn test_minimax_bot_can_play_complete_game() {
        let mut game = GameState::new();
        let light_bot = MinimaxBot::new(Player::Light, Difficulty::Easy);
        let dark_bot = MinimaxBot::new(Player::Dark, Difficulty::Easy);

        let max_turns = 200; // Reduced for faster test
        let mut turn_count = 0;

        while game.status == GameStatus::InProgress && turn_count < max_turns {
            let bot: &dyn Bot = if game.current_turn == Player::Light {
                &light_bot
            } else {
                &dark_bot
            };

            let chosen_move = bot.choose_move(&game);

            match chosen_move {
                Some(mv) => {
                    let result = game.apply_move(mv);
                    assert!(result.is_ok(), "MinimaxBot should only choose valid moves");
                }
                None => break,
            }

            turn_count += 1;
        }

        println!("MinimaxBot game ran for {} turns", turn_count);
        println!("Final status: {:?}", game.status);

        assert!(turn_count > 0, "Bots should make at least one move");
    }

    #[test]
    fn test_minimax_bot_vs_simple_bot() {
        use crate::simplebot::SimpleBot;

        let mut game = GameState::new();
        let minimax_bot = MinimaxBot::new(Player::Light, Difficulty::Easy);
        let simple_bot = SimpleBot::new(Player::Dark);

        let max_turns = 200;
        let mut turn_count = 0;

        while game.status == GameStatus::InProgress && turn_count < max_turns {
            let bot: &dyn Bot = if game.current_turn == Player::Light {
                &minimax_bot
            } else {
                &simple_bot
            };

            let chosen_move = bot.choose_move(&game);

            match chosen_move {
                Some(mv) => {
                    let result = game.apply_move(mv);
                    assert!(result.is_ok(), "Bot should only choose valid moves");
                }
                None => break,
            }

            turn_count += 1;
        }

        println!("MinimaxBot vs SimpleBot game ran for {} turns", turn_count);
        println!("Final status: {:?}", game.status);

        assert!(turn_count > 0, "Game should progress");
    }

    #[test]
    fn test_minimax_returns_consistent_moves() {
        let game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Easy);

        // Call multiple times - should return same move (deterministic)
        let move1 = bot.choose_move(&game);
        let move2 = bot.choose_move(&game);

        assert_eq!(
            format!("{:?}", move1),
            format!("{:?}", move2),
            "MinimaxBot should be deterministic"
        );
    }

    #[test]
    fn test_minimax_bot_trait_object() {
        let game = GameState::new();
        let bot: Box<dyn Bot> = Box::new(MinimaxBot::new(Player::Light, Difficulty::Medium));

        let chosen_move = bot.choose_move(&game);
        assert!(chosen_move.is_some(), "MinimaxBot trait object should work");
        assert_eq!(bot.name(), "MinimaxBot (Medium)");
    }

    #[test]
    fn test_alpha_beta_pruning_efficiency() {
        // This test verifies that minimax completes in reasonable time
        // even with deeper search (alpha-beta pruning should make it efficient)
        let game = GameState::new();
        let bot = MinimaxBot::new(Player::Light, Difficulty::Medium);

        let start = std::time::Instant::now();
        let chosen_move = bot.choose_move(&game);
        let duration = start.elapsed();

        assert!(chosen_move.is_some(), "Should find a move");
        println!("Minimax (depth {}) took {:?}", bot.difficulty().depth(), duration);

        // Should complete in reasonable time (adjust as needed)
        assert!(duration.as_secs() < 30, "Should complete within 30 seconds");
    }

    #[test]
    fn test_different_difficulties_have_different_depths() {
        let easy = MinimaxBot::new(Player::Light, Difficulty::Easy);
        let medium = MinimaxBot::new(Player::Light, Difficulty::Medium);
        let hard = MinimaxBot::new(Player::Light, Difficulty::Hard);

        assert!(easy.difficulty().depth() < medium.difficulty().depth());
        assert!(medium.difficulty().depth() < hard.difficulty().depth());
    }
}
