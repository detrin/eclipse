use std::collections::HashMap;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::board::{Hex, segments_intersect};
use crate::moves::Move;
use serde::{Deserialize, Serialize};

// =============================================================================
// GAME ENTITIES
// =============================================================================

/// Represents a player in the game.
/// Light player starts at the bottom (positive r coordinates).
/// Dark player starts at the top (negative r coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Light,
    Dark,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Light => Player::Dark,
            Player::Dark => Player::Light,
        }
    }
}

/// Type of chain connecting two satellites.
/// Chains are FIXED-LENGTH - they cannot be compressed or extended.
/// The two satellites must always be exactly max_len() hexes apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    Short, // Fixed distance of 1 hex (adjacent satellites, touching)
    Long,  // Fixed distance of 2 hexes (1 empty hex between satellites)
}

impl ChainType {
    /// Returns the fixed length of this chain type.
    /// Satellites MUST be exactly this distance apart at all times.
    pub fn max_len(&self) -> i32 {
        match self {
            // Short chain: satellites are 1 hex apart (adjacent, touching)
            ChainType::Short => 1,
            // Long chain: satellites are 2 hexes apart (1 empty hex between)
            ChainType::Long => 2,
        }
    }

    /// Returns the letter code for this chain type ('s' for Short, 'l' for Long).
    pub fn to_letter(&self) -> char {
        match self {
            ChainType::Short => 's',
            ChainType::Long => 'l',
        }
    }

    /// Parses a letter code ('s' or 'l') into a ChainType.
    pub fn from_letter(letter: char) -> Option<ChainType> {
        match letter.to_lowercase().next()? {
            's' => Some(ChainType::Short),
            'l' => Some(ChainType::Long),
            _ => None,
        }
    }
}

/// Unique identifier for a chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(pub usize);

impl ChainId {
    /// Returns the letter label for this chain (a-j).
    pub fn to_letter(&self) -> char {
        match self.0 {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            8 => 'i',
            9 => 'j',
            _ => '?',
        }
    }

    /// Parses a letter label (a-j) into a ChainId.
    pub fn from_letter(letter: char) -> Option<ChainId> {
        match letter.to_lowercase().next()? {
            'a' => Some(ChainId(0)),
            'b' => Some(ChainId(1)),
            'c' => Some(ChainId(2)),
            'd' => Some(ChainId(3)),
            'e' => Some(ChainId(4)),
            'f' => Some(ChainId(5)),
            'g' => Some(ChainId(6)),
            'h' => Some(ChainId(7)),
            'i' => Some(ChainId(8)),
            'j' => Some(ChainId(9)),
            _ => None,
        }
    }
}

/// Represents a chain connecting two satellites.
/// Chains can cross each other, blocking the opponent's movement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub id: ChainId,
    pub owner: Player,
    pub ctype: ChainType,
    /// One end of the chain (satellite position)
    pub head: Hex,
    /// Other end of the chain (satellite position)
    pub tail: Hex,
    /// Move number when head was last moved (0 = initial position, never moved)
    pub head_last_moved: usize,
    /// Move number when tail was last moved (0 = initial position, never moved)
    pub tail_last_moved: usize,
}

/// Represents what occupies a hex on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Occupant {
    /// A comet belonging to a player
    Comet(Player),
    /// A satellite (part of a chain) belonging to a player
    Satellite(ChainId, Player),
}

/// Represents the current status of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    /// Game is still in progress
    InProgress,
    /// Game has ended with a winner
    Won(Player),
}

/// Information needed to undo a move.
///
/// This structure captures all state changes from applying a move,
/// allowing the move to be efficiently reversed without full game state cloning.
#[derive(Debug, Clone)]
pub struct UndoInfo {
    /// The move that was applied
    applied_move: Move,
    /// Previous move number before the move
    prev_move_number: usize,
    /// Previous player turn before switching
    prev_turn: Player,
    /// Previous game status
    prev_status: GameStatus,
    /// Previous immobilization cache validity flag
    prev_cache_valid: bool,
    /// Comet-specific undo data (if comet move)
    comet_undo: Option<CometUndoData>,
    /// Satellite-specific undo data (if satellite move)
    satellite_undo: Option<SatelliteUndoData>,
}

/// Undo data specific to comet moves
#[derive(Debug, Clone)]
struct CometUndoData {
    /// Old comet position before the move
    old_position: Hex,
    /// Previous last_moved value for this comet
    prev_last_moved: usize,
}

/// Undo data specific to satellite moves
#[derive(Debug, Clone)]
struct SatelliteUndoData {
    /// Chain that was moved
    chain_id: ChainId,
    /// Old position before the move
    old_position: Hex,
    /// Whether head was moved (true) or tail (false)
    moved_head: bool,
    /// Previous last_moved value for the moved end
    prev_last_moved: usize,
}

// =============================================================================
// GAME STATE
// =============================================================================

/// Main game state containing all pieces, positions, and game metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Map of occupied hexes to what occupies them (comets or satellites)
    #[serde(
        serialize_with = "serialize_occupied",
        deserialize_with = "deserialize_occupied"
    )]
    pub occupied: HashMap<Hex, Occupant>,

    /// Detailed store of all chains with their properties
    #[serde(
        serialize_with = "serialize_chains",
        deserialize_with = "deserialize_chains"
    )]
    pub chains: HashMap<ChainId, Chain>,

    /// Position of the Light player's comet
    pub comet_light: Hex,
    /// Position of the Dark player's comet
    pub comet_dark: Hex,

    /// Which player's turn it is
    pub current_turn: Player,

    /// Current game status (InProgress or Won)
    pub status: GameStatus,

    /// History of moves made in the game (most recent last)
    pub move_history: Vec<Move>,

    /// Status messages to display (cleared after each move)
    pub status_messages: Vec<String>,

    /// Maximum distance from center (0,0) that defines the board boundary
    board_radius: i32,

    /// Current move number (incremented each time a move is applied)
    /// Used to track which pieces moved most recently for rendering z-order
    pub move_number: usize,

    /// Move number when Light comet was last moved (0 = never moved)
    pub comet_light_last_moved: usize,

    /// Move number when Dark comet was last moved (0 = never moved)
    pub comet_dark_last_moved: usize,

    /// Cache of chain immobilization status (chain_id -> is_immobilized)
    /// This cache eliminates O(n²) redundant crossing checks when evaluating positions.
    /// The cache is invalidated whenever satellites move (chain geometry changes).
    /// Uses RefCell for interior mutability (allows updating cache with &self reference).
    #[serde(skip)]
    immobilization_cache: RefCell<HashMap<ChainId, bool>>,

    /// Flag indicating whether the immobilization cache is valid
    /// Set to false when satellites move, causing cache to be recomputed on next access
    /// Uses RefCell for interior mutability (allows updating cache with &self reference).
    #[serde(skip)]
    cache_valid: RefCell<bool>,
}

// =============================================================================
// GAME STATE IMPLEMENTATION
// =============================================================================

impl GameState {
    /// Creates a new game with initial setup.
    /// Dark player on left side (negative q), Light player on right side (positive q).
    pub fn new() -> Self {
        let mut state = GameState {
            occupied: HashMap::new(),
            chains: HashMap::new(),
            comet_light: Hex::new(3, 0),
            comet_dark: Hex::new(-3, 0),
            current_turn: Player::Light,
            status: GameStatus::InProgress,
            move_history: Vec::new(),
            status_messages: Vec::new(),
            board_radius: 4, // Not used for custom board, but kept for compatibility
            move_number: 0,
            comet_light_last_moved: 0,
            comet_dark_last_moved: 0,
            immobilization_cache: RefCell::new(HashMap::new()),
            cache_valid: RefCell::new(false), // Will be computed on first access
        };

        // Place comets in occupied map
        state.occupied.insert(state.comet_light, Occupant::Comet(Player::Light));
        state.occupied.insert(state.comet_dark, Occupant::Comet(Player::Dark));

        // --- Setup Light (Right side) ---
        // Starting position: Light comet at (3, 0) on the right
        // Formation: Defensive shield with satellites positioned between comet and center
        // All chains must have EXACTLY their fixed length (Short=1, Long=2)
        // Board constraints:
        //   r=-3: q in [-1, 4]  |  r=-2: q in [-2, 4]  |  r=-1: q in [-3, 4]
        //   r=0:  q in [-3, 3]  |  r=1:  q in [-4, 3]  |  r=2:  q in [-4, 2]
        //   r=3:  q in [-4, 1]

        // OUTER WINGS (Short Chains - distance 1, adjacent/touching)
        // Short 'a': Top right wing (4,-3) ←→ (4,-2)
        state.add_chain(0, Player::Light, ChainType::Short, Hex::new(4, -3), Hex::new(4, -2));
        // Short 'b': Bottom right wing (2,2) ←→ (1,3)
        state.add_chain(1, Player::Light, ChainType::Short, Hex::new(2, 2), Hex::new(1, 3));

        // INNER SHIELD (Long Chains - distance 2, NOT axis-aligned)
        // Long 'c': Diagonal (4,-1) ←→ (3,1)
        state.add_chain(2, Player::Light, ChainType::Long, Hex::new(4, -1), Hex::new(3, 1));
        // Long 'd': Diagonal (3,-1) ←→ (2,1)
        state.add_chain(3, Player::Light, ChainType::Long, Hex::new(3, -1), Hex::new(2, 1));
        // Long 'e': Diagonal (2,-1) ←→ (1,1)
        state.add_chain(4, Player::Light, ChainType::Long, Hex::new(2, -1), Hex::new(1, 1));

        // --- Setup Dark (Left side) ---
        // Dark comet at (-3, 0) on the left
        // Mirror formation (180° rotation from Light)

        // OUTER WINGS (Short Chains - distance 1, adjacent/touching)
        // Short 'f': Top left wing (-1,-3) ←→ (-2,-2)
        state.add_chain(5, Player::Dark, ChainType::Short, Hex::new(-1, -3), Hex::new(-2, -2));
        // Short 'g': Bottom left wing (-4,2) ←→ (-4,3)
        state.add_chain(6, Player::Dark, ChainType::Short, Hex::new(-4, 2), Hex::new(-4, 3));

        // INNER SHIELD (Long Chains - distance 2, NOT axis-aligned)
        // Long 'h': Diagonal (-3,-1) ←→ (-4,1)
        state.add_chain(7, Player::Dark, ChainType::Long, Hex::new(-3, -1), Hex::new(-4, 1));
        // Long 'i': Diagonal (-2,-1) ←→ (-3,1)
        state.add_chain(8, Player::Dark, ChainType::Long, Hex::new(-2, -1), Hex::new(-3, 1));
        // Long 'j': Diagonal (-1,-1) ←→ (-2,1)
        state.add_chain(9, Player::Dark, ChainType::Long, Hex::new(-1, -1), Hex::new(-2, 1));

        state
    }

    fn add_chain(&mut self, id_val: usize, owner: Player, ctype: ChainType, p1: Hex, p2: Hex) {
        let id = ChainId(id_val);
        let chain = Chain {
            id,
            owner,
            ctype,
            head: p1,
            tail: p2,
            head_last_moved: 0,
            tail_last_moved: 0,
        };
        self.chains.insert(id, chain);
        self.occupied.insert(p1, Occupant::Satellite(id, owner));
        self.occupied.insert(p2, Occupant::Satellite(id, owner));
    }

    /// Validates the initial game setup to ensure no overlapping pieces
    /// and correct chain lengths.
    ///
    /// # Returns
    /// `Ok(())` if setup is valid, `Err(String)` with error message otherwise
    pub fn validate_setup(&self) -> Result<(), String> {
        // 1. Check that comets are on the board
        if !self.comet_light.is_on_board() {
            return Err(format!("Light comet at {:?} is not on the board", self.comet_light));
        }
        if !self.comet_dark.is_on_board() {
            return Err(format!("Dark comet at {:?} is not on the board", self.comet_dark));
        }

        // 2. Check that comets are properly registered in occupied map
        match self.occupied.get(&self.comet_light) {
            Some(Occupant::Comet(Player::Light)) => {},
            _ => return Err(format!("Light comet at {:?} is not properly registered in occupied map", self.comet_light)),
        }
        match self.occupied.get(&self.comet_dark) {
            Some(Occupant::Comet(Player::Dark)) => {},
            _ => return Err(format!("Dark comet at {:?} is not properly registered in occupied map", self.comet_dark)),
        }

        // 3. Validate each chain
        for (chain_id, chain) in &self.chains {
            // Check that both ends are on the board
            if !chain.head.is_on_board() {
                return Err(format!("Chain {:?} head at {:?} is not on the board", chain_id, chain.head));
            }
            if !chain.tail.is_on_board() {
                return Err(format!("Chain {:?} tail at {:?} is not on the board", chain_id, chain.tail));
            }

            // Check that chain length is exactly the required length (chains are fixed-length)
            let distance = chain.head.distance(&chain.tail);
            if distance != chain.ctype.max_len() {
                return Err(format!(
                    "Chain {:?} has length {} but must be exactly {} for {:?}",
                    chain_id, distance, chain.ctype.max_len(), chain.ctype
                ));
            }

            // Long chains can be either axis-aligned or diagonal
            // Both configurations are valid as long as distance = 2

            // Check that both ends are properly registered in occupied map
            match self.occupied.get(&chain.head) {
                Some(Occupant::Satellite(id, owner)) if id == chain_id && owner == &chain.owner => {},
                _ => return Err(format!(
                    "Chain {:?} head at {:?} is not properly registered in occupied map",
                    chain_id, chain.head
                )),
            }
            match self.occupied.get(&chain.tail) {
                Some(Occupant::Satellite(id, owner)) if id == chain_id && owner == &chain.owner => {},
                _ => return Err(format!(
                    "Chain {:?} tail at {:?} is not properly registered in occupied map",
                    chain_id, chain.tail
                )),
            }
        }

        // 4. Check that the number of occupied positions matches expected
        // Expected: 2 comets + (2 satellites per chain * number of chains)
        let expected_occupied = 2 + (self.chains.len() * 2);
        if self.occupied.len() != expected_occupied {
            return Err(format!(
                "Occupied map has {} entries but expected {}",
                self.occupied.len(), expected_occupied
            ));
        }

        // 5. Verify no duplicate positions (this should be impossible due to HashMap, but check anyway)
        let mut positions = std::collections::HashSet::new();
        for pos in self.occupied.keys() {
            if !positions.insert(*pos) {
                return Err(format!("Duplicate position found at {:?}", pos));
            }
        }

        // 6. Validate piece counts per player
        let mut light_satellites = 0;
        let mut dark_satellites = 0;
        let mut light_comets = 0;
        let mut dark_comets = 0;

        for occupant in self.occupied.values() {
            match occupant {
                Occupant::Comet(Player::Light) => light_comets += 1,
                Occupant::Comet(Player::Dark) => dark_comets += 1,
                Occupant::Satellite(_, Player::Light) => light_satellites += 1,
                Occupant::Satellite(_, Player::Dark) => dark_satellites += 1,
            }
        }

        if light_comets != 1 {
            return Err(format!("Light player should have 1 comet, found {}", light_comets));
        }
        if dark_comets != 1 {
            return Err(format!("Dark player should have 1 comet, found {}", dark_comets));
        }
        if light_satellites != 10 {
            return Err(format!("Light player should have 10 satellites (5 chains * 2), found {}", light_satellites));
        }
        if dark_satellites != 10 {
            return Err(format!("Dark player should have 10 satellites (5 chains * 2), found {}", dark_satellites));
        }

        Ok(())
    }

    /// Resets the game to the initial starting position.
    ///
    /// This clears all current state and re-initializes the game
    /// to the standard starting configuration.
    pub fn reset(&mut self) {
        *self = GameState::new();
    }

    /// The Core Mechanic: Check if two chains intersect
    /// Returns true if chain A and chain B cross each other.
    fn chains_cross(&self, chain_a: &Chain, chain_b: &Chain) -> bool {
        let (a1, a2) = (chain_a.head.to_pixel(), chain_a.tail.to_pixel());
        let (b1, b2) = (chain_b.head.to_pixel(), chain_b.tail.to_pixel());

        segments_intersect(a1, a2, b1, b2)
    }

    /// Returns the most recent move number for a chain (max of head and tail)
    fn chain_last_moved(&self, chain: &Chain) -> usize {
        chain.head_last_moved.max(chain.tail_last_moved)
    }

    /// Computes the immobilization status for ALL chains and updates the cache.
    ///
    /// This function performs O(n²) crossing checks for all chain pairs,
    /// but is only called once when the cache is invalidated (when satellites move).
    /// Subsequent immobilization queries use the cached results in O(1) time.
    fn compute_immobilization_cache(&self) {
        let mut cache = self.immobilization_cache.borrow_mut();
        cache.clear();

        // For each chain, check if it's immobilized by any opponent chain
        let chain_ids: Vec<ChainId> = self.chains.keys().copied().collect();

        for &chain_id in &chain_ids {
            let my_chain = &self.chains[&chain_id];
            let opponent = my_chain.owner.opponent();
            let my_last_moved = self.chain_last_moved(my_chain);
            let mut is_immobilized = false;

            for other_chain in self.chains.values() {
                if other_chain.owner == opponent {
                    if self.chains_cross(my_chain, other_chain) {
                        let other_last_moved = self.chain_last_moved(other_chain);
                        // Only immobilized if the opponent chain moved MORE recently
                        if other_last_moved > my_last_moved {
                            is_immobilized = true;
                            break;
                        }
                    }
                }
            }

            cache.insert(chain_id, is_immobilized);
        }

        // Release the mutable borrow before borrowing cache_valid
        drop(cache);
        *self.cache_valid.borrow_mut() = true;
    }

    /// Check if a specific chain is blocked (immobilized) by ANY opponent chain
    ///
    /// A chain is immobilized if:
    /// 1. An opponent's chain crosses it, AND
    /// 2. The opponent's chain moved more recently (has higher move number)
    ///
    /// This ensures that the chain that moved most recently is "on top"
    /// and pins the chain underneath it.
    ///
    /// This function now uses a cache to eliminate O(n²) redundant crossing checks.
    /// The cache is recomputed when invalidated (on satellite moves).
    fn is_chain_immobilized(&self, chain_id: ChainId) -> bool {
        // If cache is invalid, recompute it for all chains
        if !*self.cache_valid.borrow() {
            self.compute_immobilization_cache();
        }

        // Return cached result (defaults to false if chain_id not found)
        *self.immobilization_cache.borrow()
            .get(&chain_id)
            .unwrap_or(&false)
    }

    /// Public method to check if a chain is immobilized.
    /// This is used by bots to evaluate moves strategically.
    ///
    /// # Arguments
    /// * `chain_id` - The ID of the chain to check
    ///
    /// # Returns
    /// `true` if the chain is crossed by a more recently moved opponent chain, `false` otherwise
    pub fn is_chain_immobilized_external(&self, chain_id: ChainId) -> bool {
        self.is_chain_immobilized(chain_id)
    }

    /// Get the last move number for a chain (public version for API use)
    ///
    /// Returns the maximum of head_last_moved and tail_last_moved for the chain.
    /// This can be used to determine rendering z-order for crossed chains.
    ///
    /// # Arguments
    /// * `chain_id` - The ID of the chain to query
    ///
    /// # Returns
    /// The move number when this chain was last moved (0 if never moved)
    pub fn get_chain_last_moved(&self, chain_id: ChainId) -> Option<usize> {
        self.chains.get(&chain_id).map(|chain| self.chain_last_moved(chain))
    }

    /// Computes a hash for the current game position.
    ///
    /// This hash is used for transposition tables in the minimax search.
    /// Two positions with the same hash are considered identical for evaluation purposes.
    ///
    /// The hash includes:
    /// - Comet positions for both players
    /// - Chain positions (head/tail) and last_moved times for all chains
    /// - Current turn (who moves next)
    /// - Game status
    ///
    /// # Returns
    /// A 64-bit hash value representing this position
    pub fn hash_position(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash comet positions
        self.comet_light.q.hash(&mut hasher);
        self.comet_light.r.hash(&mut hasher);
        self.comet_dark.q.hash(&mut hasher);
        self.comet_dark.r.hash(&mut hasher);

        // Hash current turn
        (self.current_turn as u8).hash(&mut hasher);

        // Hash game status
        match self.status {
            GameStatus::InProgress => 0u8.hash(&mut hasher),
            GameStatus::Won(Player::Light) => 1u8.hash(&mut hasher),
            GameStatus::Won(Player::Dark) => 2u8.hash(&mut hasher),
        }

        // Hash all chains in a deterministic order (sorted by chain_id)
        let mut chain_ids: Vec<ChainId> = self.chains.keys().copied().collect();
        chain_ids.sort_by_key(|id| id.0);

        for chain_id in chain_ids {
            let chain = &self.chains[&chain_id];
            chain_id.0.hash(&mut hasher);
            chain.head.q.hash(&mut hasher);
            chain.head.r.hash(&mut hasher);
            chain.tail.q.hash(&mut hasher);
            chain.tail.r.hash(&mut hasher);
            chain.head_last_moved.hash(&mut hasher);
            chain.tail_last_moved.hash(&mut hasher);
            (chain.owner as u8).hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Applies a move optimized for search (minimax) with undo capability.
    ///
    /// This is a lightweight version of apply_move that:
    /// - Skips status messages (not needed for search)
    /// - Skips move history tracking (not needed for search)
    /// - Returns undo information for efficient reversal
    /// - Still validates move legality and updates game state correctly
    ///
    /// # Arguments
    /// * `mv` - The move to apply
    ///
    /// # Returns
    /// * `Ok(UndoInfo)` - Undo information if move was valid
    /// * `Err(String)` - Error message if move was invalid
    pub fn apply_move_for_search(&mut self, mv: Move) -> Result<UndoInfo, String> {
        // Capture undo information before making changes
        let prev_move_number = self.move_number;
        let prev_turn = self.current_turn;
        let prev_status = self.status;
        let prev_cache_valid = *self.cache_valid.borrow();

        // Increment move number
        self.move_number += 1;

        let (comet_undo, satellite_undo) = match mv {
            Move::MoveComet(new_pos) => {
                // Get current comet position
                let old_pos = if self.current_turn == Player::Light {
                    self.comet_light
                } else {
                    self.comet_dark
                };

                // Validate the move is legal
                if !self.is_valid_comet_move(old_pos, new_pos, self.current_turn) {
                    self.move_number = prev_move_number; // Restore move number
                    return Err(format!(
                        "Invalid comet move from {:?} to {:?}",
                        old_pos, new_pos
                    ));
                }

                // Capture undo data
                let prev_last_moved = if self.current_turn == Player::Light {
                    self.comet_light_last_moved
                } else {
                    self.comet_dark_last_moved
                };

                // Update occupied map
                self.occupied.remove(&old_pos);
                self.occupied.insert(new_pos, Occupant::Comet(self.current_turn));

                // Update comet position and last_moved
                if self.current_turn == Player::Light {
                    self.comet_light = new_pos;
                    self.comet_light_last_moved = self.move_number;
                } else {
                    self.comet_dark = new_pos;
                    self.comet_dark_last_moved = self.move_number;
                }

                let comet_data = CometUndoData {
                    old_position: old_pos,
                    prev_last_moved,
                };

                (Some(comet_data), None)
            }

            Move::MoveSatellite { chain_id, old_pos, new_pos } => {
                // Get the chain
                let chain = self.chains.get(&chain_id)
                    .ok_or_else(|| {
                        self.move_number = prev_move_number;
                        format!("Chain {:?} not found", chain_id)
                    })?;

                // Verify the chain belongs to the current player
                if chain.owner != self.current_turn {
                    self.move_number = prev_move_number;
                    return Err(format!(
                        "Chain {:?} belongs to {:?}, but it's {:?}'s turn",
                        chain_id, chain.owner, self.current_turn
                    ));
                }

                // Determine which end is moving
                let is_moving_head = chain.head == old_pos;
                let is_moving_tail = chain.tail == old_pos;

                if !is_moving_head && !is_moving_tail {
                    self.move_number = prev_move_number;
                    return Err(format!(
                        "Position {:?} is not part of chain {:?}",
                        old_pos, chain_id
                    ));
                }

                let other_end = if is_moving_head { chain.tail } else { chain.head };

                // Validate the move is legal
                if !self.is_valid_satellite_move(chain, old_pos, new_pos, other_end) {
                    self.move_number = prev_move_number;
                    return Err(format!(
                        "Invalid satellite move for chain {:?} from {:?} to {:?}",
                        chain_id, old_pos, new_pos
                    ));
                }

                // Capture undo data
                let prev_last_moved = if is_moving_head {
                    chain.head_last_moved
                } else {
                    chain.tail_last_moved
                };

                // Update occupied map
                self.occupied.remove(&old_pos);
                self.occupied.insert(new_pos, Occupant::Satellite(chain_id, self.current_turn));

                // Update chain position
                let chain = self.chains.get_mut(&chain_id).unwrap();
                if is_moving_head {
                    chain.head = new_pos;
                    chain.head_last_moved = self.move_number;
                } else {
                    chain.tail = new_pos;
                    chain.tail_last_moved = self.move_number;
                }

                // Validate chain length (same as apply_move)
                let new_distance = chain.head.distance(&chain.tail);
                if new_distance == 0 || new_distance > chain.ctype.max_len() {
                    // This shouldn't happen if is_valid_satellite_move works correctly
                    // but include for safety
                    self.move_number = prev_move_number;
                    return Err(format!(
                        "Chain {:?} would have invalid length {}",
                        chain_id, new_distance
                    ));
                }

                // For Long chains at distance 2: must be diagonal
                if chain.ctype == ChainType::Long && new_distance == 2 {
                    if chain.head.shares_axis(&chain.tail) {
                        self.move_number = prev_move_number;
                        return Err(format!(
                            "Chain {:?} at distance 2 must be diagonal",
                            chain_id
                        ));
                    }
                }

                // Invalidate immobilization cache
                *self.cache_valid.borrow_mut() = false;

                let satellite_data = SatelliteUndoData {
                    chain_id,
                    old_position: old_pos,
                    moved_head: is_moving_head,
                    prev_last_moved,
                };

                (None, Some(satellite_data))
            }
        };

        // Switch turn
        self.current_turn = self.current_turn.opponent();

        // Check for win condition
        self.check_winner();

        // Create undo info
        let undo_info = UndoInfo {
            applied_move: mv,
            prev_move_number,
            prev_turn,
            prev_status,
            prev_cache_valid,
            comet_undo,
            satellite_undo,
        };

        Ok(undo_info)
    }

    /// Undoes a move that was applied using apply_move_for_search.
    ///
    /// This efficiently reverses all changes made by a move without
    /// requiring a full game state clone.
    ///
    /// # Arguments
    /// * `undo_info` - The undo information returned by apply_move_for_search
    pub fn undo_move(&mut self, undo_info: UndoInfo) {
        // Restore basic state
        self.move_number = undo_info.prev_move_number;
        self.current_turn = undo_info.prev_turn;
        self.status = undo_info.prev_status;
        *self.cache_valid.borrow_mut() = undo_info.prev_cache_valid;

        // Undo move-specific changes
        match undo_info.applied_move {
            Move::MoveComet(new_pos) => {
                let comet_data = undo_info.comet_undo.unwrap();
                let old_pos = comet_data.old_position;

                // Restore occupied map
                self.occupied.remove(&new_pos);
                self.occupied.insert(old_pos, Occupant::Comet(self.current_turn));

                // Restore comet position and last_moved
                if self.current_turn == Player::Light {
                    self.comet_light = old_pos;
                    self.comet_light_last_moved = comet_data.prev_last_moved;
                } else {
                    self.comet_dark = old_pos;
                    self.comet_dark_last_moved = comet_data.prev_last_moved;
                }
            }

            Move::MoveSatellite { old_pos: _, new_pos, .. } => {
                let satellite_data = undo_info.satellite_undo.unwrap();

                // Restore occupied map
                self.occupied.remove(&new_pos);
                self.occupied.insert(
                    satellite_data.old_position,
                    Occupant::Satellite(satellite_data.chain_id, self.current_turn)
                );

                // Restore chain position and last_moved
                let chain = self.chains.get_mut(&satellite_data.chain_id).unwrap();
                if satellite_data.moved_head {
                    chain.head = satellite_data.old_position;
                    chain.head_last_moved = satellite_data.prev_last_moved;
                } else {
                    chain.tail = satellite_data.old_position;
                    chain.tail_last_moved = satellite_data.prev_last_moved;
                }
            }
        }
    }

    /// Generate all legal moves for the current player
    pub fn get_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let player = self.current_turn;

        // 1. Comet Moves
        let comet_pos = if player == Player::Light { self.comet_light } else { self.comet_dark };
        for neighbor in comet_pos.neighbors() {
            if self.is_valid_comet_move(comet_pos, neighbor, player) {
                moves.push(Move::MoveComet(neighbor));
            }
        }

        // 2. Chain Moves
        for chain in self.chains.values() {
            if chain.owner != player { continue; }

            // If chain is crossed by an opponent chain, it is immobilized!
            // When a chain is crossed, BOTH satellites of that chain cannot move.
            // This is the core "blocking" mechanic of the game.
            if self.is_chain_immobilized(chain.id) {
                continue;
            }

            // Try moving Head (tail stays fixed)
            let potential_hexes = self.get_reachable_hexes(chain.tail, chain.ctype.max_len());
            for hex in potential_hexes {
                // Validate: new head position to tail must be <= max_len()
                if self.is_valid_satellite_move(chain, chain.head, hex, chain.tail) {
                    moves.push(Move::MoveSatellite { chain_id: chain.id, old_pos: chain.head, new_pos: hex });
                }
            }

            // Try moving Tail (head stays fixed)
            let potential_hexes = self.get_reachable_hexes(chain.head, chain.ctype.max_len());
            for hex in potential_hexes {
                // Validate: new tail position to head must be <= max_len()
                if self.is_valid_satellite_move(chain, chain.tail, hex, chain.head) {
                    moves.push(Move::MoveSatellite { chain_id: chain.id, old_pos: chain.tail, new_pos: hex });
                }
            }
        }

        moves
    }

    fn is_valid_comet_move(&self, from: Hex, to: Hex, player: Player) -> bool {
        // 1. Is it on board?
        if !to.is_on_board() { return false; }

        // 2. Is it empty?
        if self.occupied.contains_key(&to) { return false; }

        // 3. Does the path cross an OPPONENT chain?
        // (Comets can jump their own chains, but not opponent's)
        let opponent = player.opponent();
        for chain in self.chains.values() {
            if chain.owner == opponent {
                // Check if the segment (CometStart -> CometEnd) intersects the Chain
                let (c1, c2) = (from.to_pixel(), to.to_pixel());
                let (ch1, ch2) = (chain.head.to_pixel(), chain.tail.to_pixel());
                if segments_intersect(c1, c2, ch1, ch2) {
                    return false;
                }
            }
        }

        true
    }

    /// Validates if a satellite move is legal.
    ///
    /// # Arguments
    /// * `chain` - The chain being moved
    /// * `old_pos` - Current position of the satellite being moved
    /// * `new_pos` - Target position for the satellite
    /// * `other_end` - Position of the OTHER end of the chain (the one NOT moving)
    ///
    /// # Validation Checks
    /// 1. New position must be empty (not occupied)
    /// 2. New position must be on the board
    /// 3. Distance from new_pos to other_end must be within chain's max_len
    ///    - Short chains (max_len=1): distance must be exactly 1 (adjacent neighbors)
    ///    - Long chains (max_len=2): distance can be 1 or 2 (adjacent or one hex between)
    ///
    /// # Returns
    /// `true` if the move is valid, `false` otherwise
    fn is_valid_satellite_move(&self, chain: &Chain, _old_pos: Hex, new_pos: Hex, other_end: Hex) -> bool {
        // 1. Check if new position is empty
        if self.occupied.contains_key(&new_pos) {
            return false;
        }

        // 2. Check if new position is on the board
        if !new_pos.is_on_board() {
            return false;
        }

        // 3. Validate that the new chain length is within the maximum distance
        // Chains can be any length from 1 to max_len (not stretched beyond max)
        let new_chain_length = new_pos.distance(&other_end);
        if new_chain_length == 0 || new_chain_length > chain.ctype.max_len() {
            return false;
        }

        // 4. For Long chains at distance 2: endpoints must NOT share an axis (must be diagonal)
        // This prevents rigid configurations where there's only one hex on the line between endpoints
        if chain.ctype == ChainType::Long && new_chain_length == 2 {
            if new_pos.shares_axis(&other_end) {
                return false; // Axis-aligned positions at distance 2 are not allowed
            }
        }

        // Short chains (max_len=1): distance must be exactly 1 (adjacent)
        // Long chains (max_len=2): distance 1 (any neighbor) OR distance 2 (diagonal only)

        true
    }

    /// Returns all hexes within a given distance from the center hex.
    /// Used to find valid movement destinations for satellites.
    fn get_reachable_hexes(&self, center: Hex, max_dist: i32) -> Vec<Hex> {
        center.get_hexes_in_range(max_dist)
    }

    /// Applies a move to the game state, updating all relevant data structures.
    ///
    /// This method:
    /// 1. Updates the `occupied` HashMap by removing pieces from old positions
    ///    and adding them to new positions
    /// 2. Updates chain head/tail positions for satellite moves
    /// 3. Updates comet positions for comet moves
    /// 4. Switches the turn to the opponent player
    ///
    /// # Arguments
    /// * `mv` - The move to apply (must be a legal move)
    ///
    /// # Returns
    /// * `Ok(())` if the move was successfully applied
    /// * `Err(String)` if the move is invalid or cannot be applied
    ///
    /// # Example
    /// ```
    /// use eclipse::states::GameState;
    /// let mut game = GameState::new();
    /// let moves = game.get_legal_moves();
    /// if let Some(mv) = moves.first() {
    ///     game.apply_move(mv.clone()).expect("Failed to apply move");
    /// }
    /// ```
    pub fn apply_move(&mut self, mv: Move) -> Result<(), String> {
        // Clear previous status messages
        self.status_messages.clear();

        // Increment move number
        self.move_number += 1;

        match mv {
            Move::MoveComet(new_pos) => {
                // Get current comet position based on whose turn it is
                let old_pos = if self.current_turn == Player::Light {
                    self.comet_light
                } else {
                    self.comet_dark
                };

                // Validate the move is legal
                if !self.is_valid_comet_move(old_pos, new_pos, self.current_turn) {
                    return Err(format!(
                        "Invalid comet move from {:?} to {:?}",
                        old_pos, new_pos
                    ));
                }

                // Update occupied HashMap: remove from old position
                self.occupied.remove(&old_pos);

                // Update occupied HashMap: add to new position
                self.occupied.insert(new_pos, Occupant::Comet(self.current_turn));

                // Update comet position and last_moved tracking
                if self.current_turn == Player::Light {
                    self.comet_light = new_pos;
                    self.comet_light_last_moved = self.move_number;
                } else {
                    self.comet_dark = new_pos;
                    self.comet_dark_last_moved = self.move_number;
                }
            }

            Move::MoveSatellite { chain_id, old_pos, new_pos } => {
                // Get the chain
                let chain = self.chains.get(&chain_id)
                    .ok_or_else(|| format!("Chain {:?} not found", chain_id))?;

                // Verify the chain belongs to the current player
                if chain.owner != self.current_turn {
                    return Err(format!(
                        "Chain {:?} belongs to {:?}, but it's {:?}'s turn",
                        chain_id, chain.owner, self.current_turn
                    ));
                }

                // Determine which end of the chain is moving
                let is_moving_head = chain.head == old_pos;
                let is_moving_tail = chain.tail == old_pos;

                if !is_moving_head && !is_moving_tail {
                    return Err(format!(
                        "Position {:?} is not part of chain {:?}",
                        old_pos, chain_id
                    ));
                }

                // Determine the other end (the one that stays fixed)
                let other_end = if is_moving_head { chain.tail } else { chain.head };

                // Validate the move is legal (including chain length constraint)
                if !self.is_valid_satellite_move(chain, old_pos, new_pos, other_end) {
                    return Err(format!(
                        "Invalid satellite move for chain {:?} from {:?} to {:?}",
                        chain_id, old_pos, new_pos
                    ));
                }

                // Update occupied HashMap: remove from old position
                self.occupied.remove(&old_pos);

                // Update occupied HashMap: add to new position
                self.occupied.insert(
                    new_pos,
                    Occupant::Satellite(chain_id, self.current_turn)
                );

                // Update chain positions in the chains HashMap
                let chain = self.chains.get_mut(&chain_id)
                    .ok_or_else(|| format!("Chain {:?} not found", chain_id))?;

                if is_moving_head {
                    chain.head = new_pos;
                    chain.head_last_moved = self.move_number;
                } else {
                    chain.tail = new_pos;
                    chain.tail_last_moved = self.move_number;
                }

                // Validate that the new chain length is within the allowed range
                // Chains can be any length from 1 to max_len
                let new_distance = chain.head.distance(&chain.tail);
                if new_distance == 0 || new_distance > chain.ctype.max_len() {
                    return Err(format!(
                        "Chain {:?} would have length {} but must be between 1 and {} for {:?}",
                        chain_id, new_distance, chain.ctype.max_len(), chain.ctype
                    ));
                }

                // For Long chains at distance 2: endpoints must NOT share an axis (must be diagonal)
                if chain.ctype == ChainType::Long && new_distance == 2 {
                    if chain.head.shares_axis(&chain.tail) {
                        return Err(format!(
                            "Chain {:?} at distance 2 must be diagonal (not axis-aligned)",
                            chain_id
                        ));
                    }
                }

                // Invalidate immobilization cache since chain geometry has changed
                *self.cache_valid.borrow_mut() = false;
            }
        }

        // Add move to history
        self.move_history.push(mv.clone());

        // Add success message
        match mv {
            Move::MoveComet(pos) => {
                self.status_messages.push(format!("Comet moved to ({}, {})", pos.q, pos.r));
            }
            Move::MoveSatellite { chain_id, old_pos, new_pos } => {
                let chain = &self.chains[&chain_id];
                let chain_letter = chain_id.to_letter();

                self.status_messages.push(format!(
                    "Chain '{}' [{}] ({:?}) satellite moved from ({}, {}) to ({}, {})",
                    chain_letter, chain_id.0, chain.ctype, old_pos.q, old_pos.r, new_pos.q, new_pos.r
                ));

                // Check if this move resulted in crossing opponent chains
                for other_chain in self.chains.values() {
                    if other_chain.owner != chain.owner {
                        if self.chains_cross_internal(chain, other_chain) {
                            self.status_messages.push(format!(
                                "⚠ Chain '{}' [{}] is now crossing opponent chain '{}' [{}]!",
                                chain_letter, chain_id.0, other_chain.id.to_letter(), other_chain.id.0
                            ));
                        }
                    }
                }
            }
        }

        // Switch turn to the opponent
        self.current_turn = self.current_turn.opponent();

        // Check for immobilized chains for the new current player
        let mut immobilized_count = 0;
        for chain in self.chains.values() {
            if chain.owner == self.current_turn && self.is_chain_immobilized(chain.id) {
                immobilized_count += 1;
            }
        }

        if immobilized_count > 0 {
            self.status_messages.push(format!(
                "ℹ {:?} has {} immobilized chain(s)",
                self.current_turn, immobilized_count
            ));
        }

        // Check for win condition after the move
        let prev_status = self.status;
        self.check_winner();

        // Add win message if game just ended
        if prev_status != self.status {
            if let GameStatus::Won(winner) = self.status {
                self.status_messages.push(format!(
                    "🎉 Game Over! {:?} wins! Opponent comet immobilized.",
                    winner
                ));
            }
        }

        Ok(())
    }

    /// Helper method to check if two chains cross (internal version without borrowing issues)
    fn chains_cross_internal(&self, chain_a: &Chain, chain_b: &Chain) -> bool {
        let (a1, a2) = (chain_a.head.to_pixel(), chain_a.tail.to_pixel());
        let (b1, b2) = (chain_b.head.to_pixel(), chain_b.tail.to_pixel());

        crate::board::segments_intersect(a1, a2, b1, b2)
    }

    /// Checks if the game has been won by detecting if a player has no comet moves.
    ///
    /// A player wins if their opponent's comet has no legal moves available.
    /// This method updates the game status to Won(Player) if a winner is detected.
    ///
    /// # Win Condition
    /// After a player makes a move, if the opponent has no legal comet moves,
    /// then the current player wins.
    ///
    /// # Returns
    /// Returns the current game status (InProgress or Won(Player))
    pub fn check_winner(&mut self) -> GameStatus {
        // If game is already won, return the status
        if let GameStatus::Won(_) = self.status {
            return self.status;
        }

        // Check if the current player (who just started their turn) has any legal comet moves
        let has_comet_moves = self.has_legal_comet_moves(self.current_turn);

        // If the current player has no legal comet moves, the opponent wins
        if !has_comet_moves {
            let winner = self.current_turn.opponent();
            self.status = GameStatus::Won(winner);
        }

        self.status
    }

    /// Returns whether the specified player has any legal comet moves.
    pub fn has_legal_comet_moves(&self, player: Player) -> bool {
        let comet_pos = if player == Player::Light { self.comet_light } else { self.comet_dark };
        comet_pos
            .neighbors()
            .into_iter()
            .any(|neighbor| self.is_valid_comet_move(comet_pos, neighbor, player))
    }
}

// =============================================================================
// SERDE HELPERS
// =============================================================================

#[derive(Serialize, Deserialize)]
struct OccupiedEntry {
    hex: Hex,
    occupant: Occupant,
}

/// Custom serialization for occupied HashMap - serialize as Vec of objects
fn serialize_occupied<S>(
    map: &HashMap<Hex, Occupant>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let vec: Vec<OccupiedEntry> = map.iter()
        .map(|(k, v)| OccupiedEntry { hex: *k, occupant: *v })
        .collect();
    vec.serialize(serializer)
}

/// Custom deserialization for occupied HashMap - deserialize from Vec of objects
fn deserialize_occupied<'de, D>(
    deserializer: D,
) -> Result<HashMap<Hex, Occupant>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let vec: Vec<OccupiedEntry> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().map(|entry| (entry.hex, entry.occupant)).collect())
}

/// Custom serialization for chains HashMap - serialize as Vec
fn serialize_chains<S>(
    map: &HashMap<ChainId, Chain>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let vec: Vec<Chain> = map.values().cloned().collect();
    vec.serialize(serializer)
}

/// Custom deserialization for chains HashMap - deserialize from Vec
fn deserialize_chains<'de, D>(
    deserializer: D,
) -> Result<HashMap<ChainId, Chain>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let vec: Vec<Chain> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().map(|chain| (chain.id, chain)).collect())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Hex;
    use crate::moves::Move;

    #[test]
    fn test_apply_comet_move() {
        let mut game = GameState::new();
        let initial_turn = game.current_turn;
        let initial_comet_pos = game.comet_light;

        // Get a legal comet move
        let moves = game.get_legal_moves();
        let comet_move = moves.iter().find(|m| matches!(m, Move::MoveComet(_)));

        if let Some(Move::MoveComet(new_pos)) = comet_move {
            // Apply the move
            let result = game.apply_move(Move::MoveComet(*new_pos));
            assert!(result.is_ok(), "Move should be valid");

            // Check that comet position was updated
            assert_eq!(game.comet_light, *new_pos);
            assert_ne!(game.comet_light, initial_comet_pos);

            // Check that occupied map was updated
            assert!(!game.occupied.contains_key(&initial_comet_pos));
            assert_eq!(
                game.occupied.get(new_pos),
                Some(&Occupant::Comet(Player::Light))
            );

            // Check that turn was switched
            assert_eq!(game.current_turn, initial_turn.opponent());
        }
    }

    #[test]
    fn test_apply_satellite_move() {
        let mut game = GameState::new();
        let initial_turn = game.current_turn;

        // Get a legal satellite move
        let moves = game.get_legal_moves();
        let satellite_move = moves.iter().find(|m| matches!(m, Move::MoveSatellite { .. }));

        if let Some(Move::MoveSatellite { chain_id, old_pos, new_pos }) = satellite_move {
            let chain_id = *chain_id;
            let old_pos = *old_pos;
            let new_pos = *new_pos;

            // Apply the move
            let result = game.apply_move(Move::MoveSatellite { chain_id, old_pos, new_pos });
            assert!(result.is_ok(), "Move should be valid");

            // Check that chain position was updated
            let updated_chain = game.chains.get(&chain_id).unwrap();
            assert!(
                updated_chain.head == new_pos || updated_chain.tail == new_pos,
                "Chain should have new position"
            );
            assert!(
                updated_chain.head != old_pos && updated_chain.tail != old_pos,
                "Chain should not have old position"
            );

            // Check that occupied map was updated
            assert!(!game.occupied.contains_key(&old_pos));
            assert_eq!(
                game.occupied.get(&new_pos),
                Some(&Occupant::Satellite(chain_id, Player::Light))
            );

            // Check that turn was switched
            assert_eq!(game.current_turn, initial_turn.opponent());

            // Verify chain length is still valid
            let distance = updated_chain.head.distance(&updated_chain.tail);
            assert!(
                distance <= updated_chain.ctype.max_len(),
                "Chain length should be within maximum"
            );
        }
    }

    #[test]
    fn test_turn_switching() {
        let mut game = GameState::new();

        // Initial turn should be Light
        assert_eq!(game.current_turn, Player::Light);

        // Apply a move
        let moves = game.get_legal_moves();
        if let Some(mv) = moves.first() {
            game.apply_move(mv.clone()).unwrap();

            // Turn should now be Dark
            assert_eq!(game.current_turn, Player::Dark);

            // Apply another move
            let moves = game.get_legal_moves();
            if let Some(mv) = moves.first() {
                game.apply_move(mv.clone()).unwrap();

                // Turn should be back to Light
                assert_eq!(game.current_turn, Player::Light);
            }
        }
    }

    #[test]
    fn test_invalid_move_to_occupied_position() {
        let mut game = GameState::new();

        // Try to move comet to an occupied position (where a satellite is)
        let occupied_pos = game.chains.values().next().unwrap().head;
        let result = game.apply_move(Move::MoveComet(occupied_pos));

        assert!(result.is_err(), "Moving to occupied position should fail");
    }

    #[test]
    fn test_invalid_move_wrong_player_chain() {
        let mut game = GameState::new();

        // Set turn to Light
        game.current_turn = Player::Light;

        // Try to move a Dark player's chain
        let dark_chain = game.chains.values()
            .find(|c| c.owner == Player::Dark)
            .unwrap();

        let result = game.apply_move(Move::MoveSatellite {
            chain_id: dark_chain.id,
            old_pos: dark_chain.head,
            new_pos: Hex::new(0, 0),
        });

        assert!(result.is_err(), "Moving opponent's chain should fail");
    }

    #[test]
    fn test_occupied_map_consistency() {
        let mut game = GameState::new();
        let initial_occupied_count = game.occupied.len();

        // Apply several moves and check that occupied map size stays consistent
        for _ in 0..5 {
            let moves = game.get_legal_moves();
            if let Some(mv) = moves.first() {
                game.apply_move(mv.clone()).unwrap();

                // Occupied map should have same number of entries
                assert_eq!(
                    game.occupied.len(),
                    initial_occupied_count,
                    "Occupied map should maintain consistent size"
                );
            } else {
                break;
            }
        }
    }

    #[test]
    fn test_chain_length_validation() {
        let mut game = GameState::new();

        // Find a short chain
        let short_chain = game.chains.values()
            .find(|c| c.ctype == ChainType::Short && c.owner == Player::Light)
            .unwrap();

        // Try to move it beyond maximum length
        let far_position = short_chain.head.add(Hex::new(5, 0)); // Very far away

        let result = game.apply_move(Move::MoveSatellite {
            chain_id: short_chain.id,
            old_pos: short_chain.tail,
            new_pos: far_position,
        });

        // This should fail during validation in apply_move or is_valid_satellite_move
        assert!(result.is_err(), "Moving chain beyond max length should fail");
    }

    #[test]
    fn test_validate_setup() {
        let game = GameState::new();

        // Initial setup should be valid
        match game.validate_setup() {
            Ok(()) => {}, // Success
            Err(e) => panic!("Initial setup validation failed: {}", e),
        }
    }

    #[test]
    fn test_reset() {
        let mut game = GameState::new();

        // Apply some moves
        let moves = game.get_legal_moves();
        if let Some(mv) = moves.first() {
            game.apply_move(mv.clone()).unwrap();
        }

        // Current turn should be Dark after one move
        assert_eq!(game.current_turn, Player::Dark);

        // Reset the game
        game.reset();

        // Should be back to initial state
        assert_eq!(game.current_turn, Player::Light);
        assert_eq!(game.comet_light, Hex::new(3, 0));
        assert_eq!(game.comet_dark, Hex::new(-3, 0));
        assert!(game.validate_setup().is_ok());
    }

    #[test]
    fn test_get_legal_moves_generates_moves() {
        let game = GameState::new();
        let moves = game.get_legal_moves();

        // Should have legal moves at the start
        // With fixed-length chains, there are fewer possible moves than with flexible chains
        assert!(!moves.is_empty(), "Should have legal moves at game start");
        println!("Generated {} legal moves", moves.len());
        assert!(moves.len() > 0, "Should have legal moves at game start");
    }

    #[test]
    fn test_initial_game_status() {
        let game = GameState::new();
        assert_eq!(game.status, GameStatus::InProgress, "New game should be in progress");
    }

    #[test]
    fn test_check_winner_no_winner_at_start() {
        let mut game = GameState::new();
        let status = game.check_winner();
        assert_eq!(status, GameStatus::InProgress, "No winner at game start");
    }

    #[test]
    fn test_win_condition_immobilized_comet() {
        // Create a scenario where Dark has no legal moves (comet surrounded AND all chains blocked)
        let mut game = GameState::new();

        // Move Dark comet to corner where it can be trapped
        game.comet_dark = Hex::new(-3, 0);
        game.current_turn = Player::Dark;

        // Surround the comet with Light satellites
        let surrounding_positions = game.comet_dark.neighbors();
        for (i, pos) in surrounding_positions.iter().take(6).enumerate() {
            if pos.is_on_board() {
                game.occupied.insert(*pos, Occupant::Satellite(ChainId(100 + i), Player::Light));
            }
        }

        // Remove all Dark chains so they have no satellite moves either
        // Keep only Light chains
        game.chains.retain(|_, chain| chain.owner == Player::Light);

        // Update occupied map to remove Dark satellites
        game.occupied.retain(|_, occupant| {
            match occupant {
                Occupant::Comet(player) => *player == Player::Light || *player == Player::Dark,
                Occupant::Satellite(_, player) => *player == Player::Light,
            }
        });

        // Invalidate cache since we modified chains directly
        *game.cache_valid.borrow_mut() = false;

        // Now check for winner - Dark should have no moves at all
        let status = game.check_winner();

        // Dark has no legal moves (comet immobilized and no chains), so Light should win
        assert_eq!(status, GameStatus::Won(Player::Light), "Light should win when Dark has no legal moves");
        assert_eq!(game.status, GameStatus::Won(Player::Light), "Game status should be updated");
    }

    #[test]
    fn test_game_status_persists_after_win() {
        let mut game = GameState::new();

        // Manually set a win condition
        game.status = GameStatus::Won(Player::Dark);

        // Check winner again - should still be the same
        let status = game.check_winner();
        assert_eq!(status, GameStatus::Won(Player::Dark), "Win status should persist");
    }

    #[test]
    fn test_apply_move_checks_winner() {
        let mut game = GameState::new();

        // Apply a move
        let moves = game.get_legal_moves();
        if let Some(mv) = moves.first() {
            game.apply_move(mv.clone()).unwrap();

            // After a normal move, game should still be in progress
            assert_eq!(game.status, GameStatus::InProgress, "Game should still be in progress after normal move");
        }
    }

    #[test]
    fn test_reset_clears_win_status() {
        let mut game = GameState::new();

        // Manually set a win condition
        game.status = GameStatus::Won(Player::Dark);

        // Reset the game
        game.reset();

        // Status should be back to in progress
        assert_eq!(game.status, GameStatus::InProgress, "Reset should clear win status");
    }

    // =============================================================================
    // TASK 1.4: MOVE VALIDATION REFINEMENT TESTS
    // =============================================================================

    #[test]
    fn test_chain_length_constraint_short_chain() {
        let mut game = GameState::new();

        // Find a short chain (max_len = 2)
        let short_chain = game.chains.values()
            .find(|c| c.ctype == ChainType::Short && c.owner == Player::Light)
            .unwrap();
        let chain_id = short_chain.id;
        let head = short_chain.head;
        let tail = short_chain.tail;

        // Try to move the head to a position that would make the chain length = 3 (exceeds max_len of 2)
        // Find a hex that is distance 3 from tail
        let too_far = tail.add(Hex::new(3, 0));

        // This move should NOT be in legal moves
        game.current_turn = Player::Light;
        let legal_moves = game.get_legal_moves();
        let invalid_move = legal_moves.iter().find(|m| {
            if let Move::MoveSatellite { chain_id: id, old_pos, new_pos } = m {
                *id == chain_id && *old_pos == head && *new_pos == too_far
            } else {
                false
            }
        });

        assert!(invalid_move.is_none(), "Move that exceeds chain length should not be generated");

        // Verify that trying to apply such a move would fail
        let result = game.apply_move(Move::MoveSatellite {
            chain_id,
            old_pos: head,
            new_pos: too_far,
        });

        assert!(result.is_err(), "Move exceeding chain max_len should fail");
    }

    #[test]
    fn test_chain_length_constraint_long_chain() {
        let mut game = GameState::new();

        // Find a long chain (max_len = 3)
        let long_chain = game.chains.values()
            .find(|c| c.ctype == ChainType::Long && c.owner == Player::Light)
            .unwrap();
        let chain_id = long_chain.id;
        let head = long_chain.head;
        let tail = long_chain.tail;

        // Try to move the tail to a position that would make the chain length = 4 (exceeds max_len of 3)
        // Find a hex that is distance 4 from head
        let too_far = head.add(Hex::new(4, 0));

        // This move should NOT be in legal moves
        game.current_turn = Player::Light;
        let legal_moves = game.get_legal_moves();
        let invalid_move = legal_moves.iter().find(|m| {
            if let Move::MoveSatellite { chain_id: id, old_pos, new_pos } = m {
                *id == chain_id && *old_pos == tail && *new_pos == too_far
            } else {
                false
            }
        });

        assert!(invalid_move.is_none(), "Move that exceeds chain length should not be generated");
    }

    #[test]
    fn test_chain_length_at_boundary() {
        let mut game = GameState::new();

        // Test that a move AT the max_len boundary is allowed
        // Find a short chain and move it to exactly distance 2
        let short_chain = game.chains.values()
            .find(|c| c.ctype == ChainType::Short && c.owner == Player::Light)
            .cloned()
            .unwrap();

        game.current_turn = Player::Light;

        // Find a valid move that results in exactly max_len distance
        let legal_moves = game.get_legal_moves();
        let boundary_move = legal_moves.iter().find(|m| {
            if let Move::MoveSatellite { chain_id, old_pos, new_pos } = m {
                if *chain_id == short_chain.id {
                    // Calculate what the new chain length would be
                    let other_end = if *old_pos == short_chain.head {
                        short_chain.tail
                    } else {
                        short_chain.head
                    };
                    let new_length = new_pos.distance(&other_end);
                    new_length == ChainType::Short.max_len()
                } else {
                    false
                }
            } else {
                false
            }
        });

        // At least one move at the boundary should exist
        if let Some(mv) = boundary_move {
            // This move should be valid
            let result = game.apply_move(mv.clone());
            assert!(result.is_ok(), "Move at max_len boundary should be valid");
        }
    }

    #[test]
    fn test_immobilized_chain_generates_no_moves() {
        let mut game = GameState::new();

        // Create a scenario where chains cross
        // Manually set up a crossing by placing chains
        game.current_turn = Player::Light;

        // Get initial move count for Light player's chains (for reference)
        let initial_moves = game.get_legal_moves();
        let _light_satellite_moves_initial = initial_moves.iter()
            .filter(|m| matches!(m, Move::MoveSatellite { .. }))
            .count();

        // Find two chains that can be made to cross (one Light, one Dark)
        let light_chain_id = game.chains.values()
            .find(|c| c.owner == Player::Light)
            .unwrap()
            .id;

        let dark_chain_id = game.chains.values()
            .find(|c| c.owner == Player::Dark)
            .unwrap()
            .id;

        // Manually create a crossing situation by setting chains to cross each other
        // This is a simplified test - in reality, crossings would happen through gameplay
        let light_chain = game.chains.get_mut(&light_chain_id).unwrap();
        light_chain.head = Hex::new(0, -1);
        light_chain.tail = Hex::new(0, 1);
        light_chain.head_last_moved = 1;  // Light moved first
        light_chain.tail_last_moved = 1;
        game.occupied.remove(&light_chain.head);
        game.occupied.remove(&light_chain.tail);
        game.occupied.insert(Hex::new(0, -1), Occupant::Satellite(light_chain_id, Player::Light));
        game.occupied.insert(Hex::new(0, 1), Occupant::Satellite(light_chain_id, Player::Light));

        let dark_chain = game.chains.get_mut(&dark_chain_id).unwrap();
        dark_chain.head = Hex::new(-1, 0);
        dark_chain.tail = Hex::new(1, 0);
        dark_chain.head_last_moved = 2;  // Dark moved second, crosses Light's chain
        dark_chain.tail_last_moved = 2;
        game.occupied.remove(&dark_chain.head);
        game.occupied.remove(&dark_chain.tail);
        game.occupied.insert(Hex::new(-1, 0), Occupant::Satellite(dark_chain_id, Player::Dark));
        game.occupied.insert(Hex::new(1, 0), Occupant::Satellite(dark_chain_id, Player::Dark));

        // Manually invalidate the immobilization cache since we modified chains directly
        // (normally this would be done automatically in apply_move)
        *game.cache_valid.borrow_mut() = false;

        // Verify the chains actually cross
        let light_chain = game.chains.get(&light_chain_id).unwrap();
        let dark_chain = game.chains.get(&dark_chain_id).unwrap();
        assert!(game.chains_cross(light_chain, dark_chain), "Chains should be crossing");

        // Now check that the Light chain is immobilized
        assert!(game.is_chain_immobilized(light_chain_id), "Light chain should be immobilized");

        // Get legal moves - the immobilized chain should generate NO moves
        game.current_turn = Player::Light;
        let moves = game.get_legal_moves();

        // Count moves for the immobilized chain
        let immobilized_chain_moves = moves.iter()
            .filter(|m| {
                if let Move::MoveSatellite { chain_id, .. } = m {
                    *chain_id == light_chain_id
                } else {
                    false
                }
            })
            .count();

        assert_eq!(immobilized_chain_moves, 0, "Immobilized chain should generate 0 moves");
    }

    #[test]
    fn test_non_immobilized_chains_still_movable() {
        let game = GameState::new();

        // At game start, no chains should be immobilized
        for (chain_id, chain) in &game.chains {
            let is_immobilized = game.is_chain_immobilized(*chain_id);
            assert!(!is_immobilized, "Chain {:?} should not be immobilized at game start", chain_id);

            // Verify this chain can generate moves (if it's the current player's chain)
            if chain.owner == game.current_turn {
                let moves = game.get_legal_moves();
                let chain_moves = moves.iter()
                    .filter(|m| {
                        if let Move::MoveSatellite { chain_id: id, .. } = m {
                            *id == *chain_id
                        } else {
                            false
                        }
                    })
                    .count();

                // Each non-immobilized chain should have multiple possible moves
                assert!(chain_moves > 0, "Non-immobilized chain {:?} should have moves", chain_id);
            }
        }
    }

    #[test]
    fn test_chain_crossing_perpendicular() {
        let game = GameState::new();

        // Test perpendicular crossing (like a + sign)
        let chain_a = Chain {
            id: ChainId(999),
            owner: Player::Light,
            ctype: ChainType::Short,
            head: Hex::new(-1, 0),
            tail: Hex::new(1, 0),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        let chain_b = Chain {
            id: ChainId(998),
            owner: Player::Dark,
            ctype: ChainType::Short,
            head: Hex::new(0, -1),
            tail: Hex::new(0, 1),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        assert!(game.chains_cross(&chain_a, &chain_b), "Perpendicular chains should cross");
    }

    #[test]
    fn test_chain_crossing_diagonal() {
        let game = GameState::new();

        // Test diagonal crossing (like an X)
        let chain_a = Chain {
            id: ChainId(999),
            owner: Player::Light,
            ctype: ChainType::Long,
            head: Hex::new(-2, 0),
            tail: Hex::new(2, 0),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        let chain_b = Chain {
            id: ChainId(998),
            owner: Player::Dark,
            ctype: ChainType::Long,
            head: Hex::new(0, -2),
            tail: Hex::new(0, 2),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        assert!(game.chains_cross(&chain_a, &chain_b), "Diagonal chains should cross");
    }

    #[test]
    fn test_chain_not_crossing_parallel() {
        let game = GameState::new();

        // Test parallel chains (should NOT cross)
        let chain_a = Chain {
            id: ChainId(999),
            owner: Player::Light,
            ctype: ChainType::Short,
            head: Hex::new(-1, 0),
            tail: Hex::new(1, 0),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        let chain_b = Chain {
            id: ChainId(998),
            owner: Player::Dark,
            ctype: ChainType::Short,
            head: Hex::new(-1, 1),
            tail: Hex::new(1, 1),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        assert!(!game.chains_cross(&chain_a, &chain_b), "Parallel chains should not cross");
    }

    #[test]
    fn test_chain_not_crossing_adjacent() {
        let game = GameState::new();

        // Test adjacent chains that touch but don't cross
        let chain_a = Chain {
            id: ChainId(999),
            owner: Player::Light,
            ctype: ChainType::Short,
            head: Hex::new(0, 0),
            tail: Hex::new(1, 0),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        let chain_b = Chain {
            id: ChainId(998),
            owner: Player::Dark,
            ctype: ChainType::Short,
            head: Hex::new(1, 0),
            tail: Hex::new(2, 0),
            head_last_moved: 0,
            tail_last_moved: 0,
        };

        assert!(!game.chains_cross(&chain_a, &chain_b), "Adjacent chains should not cross");
    }

    #[test]
    fn test_validate_all_legal_moves_respect_max_len() {
        let game = GameState::new();

        // Get all legal moves
        let moves = game.get_legal_moves();

        // For each satellite move, verify it respects max_len
        for mv in moves {
            if let Move::MoveSatellite { chain_id, old_pos, new_pos } = mv {
                let chain = game.chains.get(&chain_id).unwrap();

                // Determine the other end
                let other_end = if old_pos == chain.head {
                    chain.tail
                } else {
                    chain.head
                };

                // Calculate the new chain length
                let new_length = new_pos.distance(&other_end);

                // Verify it doesn't exceed max_len
                assert!(
                    new_length <= chain.ctype.max_len(),
                    "Move from {:?} to {:?} for chain {:?} would have length {} exceeding max {}",
                    old_pos, new_pos, chain_id, new_length, chain.ctype.max_len()
                );
            }
        }
    }

    #[test]
    fn test_long_chain_can_be_diagonal() {
        let mut game = GameState::new();
        game.current_turn = Player::Light;

        // Chain 'e' (ChainId(4)) is a Long chain with diagonal initial positions:
        // head: (2, -1), tail: (1, 1)
        let chain_e = game.chains.get(&ChainId(4)).unwrap();
        assert_eq!(chain_e.ctype, ChainType::Long);
        assert_eq!(chain_e.head, Hex::new(2, -1));
        assert_eq!(chain_e.tail, Hex::new(1, 1));

        // Verify initial setup: distance should be 2
        let distance = chain_e.head.distance(&chain_e.tail);
        assert_eq!(distance, 2, "Long chain should have distance 2");

        // Verify they DON'T share an axis (diagonal configuration)
        let shares_q = chain_e.head.q == chain_e.tail.q;
        let shares_r = chain_e.head.r == chain_e.tail.r;
        let shares_s = chain_e.head.s() == chain_e.tail.s();
        assert!(!shares_q && !shares_r && !shares_s, "Initial diagonal chain should not share any axis");
    }

    #[test]
    fn test_long_chain_moves_allow_diagonal() {
        let mut game = GameState::new();
        game.current_turn = Player::Light;

        // Get all legal moves for chain 'e'
        let legal_moves = game.get_legal_moves();
        let chain_e_moves: Vec<_> = legal_moves.iter()
            .filter(|m| {
                if let Move::MoveSatellite { chain_id, .. } = m {
                    *chain_id == ChainId(4)
                } else {
                    false
                }
            })
            .collect();

        // Chain 'e' should have some legal moves (can be diagonal or axis-aligned)
        assert!(!chain_e_moves.is_empty(), "Chain 'e' should have some legal moves");

        // Verify all legal moves stay within max distance
        for mv in chain_e_moves {
            if let Move::MoveSatellite { chain_id, old_pos, new_pos } = mv {
                let chain = game.chains.get(chain_id).unwrap();
                let other_end = if *old_pos == chain.head {
                    chain.tail
                } else {
                    chain.head
                };

                // Verify distance is within limits (1 or 2 for long chains)
                let distance = new_pos.distance(&other_end);
                assert!(
                    distance >= 1 && distance <= 2,
                    "Legal long chain move from {:?} to {:?} must be within distance 1-2 from other end {:?}, got {}",
                    old_pos, new_pos, other_end, distance
                );
            }
        }
    }

    #[test]
    fn test_short_chain_must_be_adjacent() {
        let game = GameState::new();

        // Find a short chain
        let short_chain = game.chains.values()
            .find(|c| c.ctype == ChainType::Short)
            .unwrap();

        // Short chains should have distance 1 (adjacent)
        let distance = short_chain.head.distance(&short_chain.tail);
        assert_eq!(distance, 1, "Short chains must be adjacent (distance 1)");

        // Adjacent hexes (distance 1) always share exactly one axis
        // This is a fundamental property of hex grids
        assert!(
            short_chain.head.shares_axis(&short_chain.tail),
            "Adjacent hexes must share at least one axis"
        );
    }
}
