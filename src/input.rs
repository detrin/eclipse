use crate::board::Hex;
use crate::moves::Move;
use crate::states::{ChainId, GameState, Occupant};

// =============================================================================
// INPUT PARSING
// =============================================================================

/// Represents a parsed user command
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Move comet to a hex position
    MoveComet { target: Hex },
    /// Move satellite from one position to another
    MoveSatellite {
        from: Hex,
        to: Hex,
        /// Optional chain type validation ('s' for short, 'l' for long)
        chain_type: Option<char>,
        /// Optional chain ID validation ('a' through 'j')
        chain_letter: Option<char>,
    },
    /// Show help message
    Help,
    /// Display all legal moves
    ShowMoves,
    /// Quit the game
    Quit,
}

/// Errors that can occur during input parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Empty input string
    EmptyInput,
    /// Unknown command format
    UnknownCommand(String),
    /// Invalid move notation format
    InvalidFormat(String),
    /// Invalid coordinate values
    InvalidCoordinates(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Empty input. Type 'help' for instructions."),
            ParseError::UnknownCommand(cmd) => write!(f, "Unknown command: '{}'. Type 'help' for instructions.", cmd),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::InvalidCoordinates(msg) => write!(f, "Invalid coordinates: {}", msg),
        }
    }
}

/// Parses user input into a Command
///
/// # Supported formats:
/// - `c (q,r)` - Move comet to hex (q, r)
/// - `s (q1,r1) (q2,r2)` - Move satellite from (q1,r1) to (q2,r2)
/// - `help` - Show help message
/// - `moves` - Display all legal moves
/// - `quit` or `exit` - Quit the game
///
/// # Examples
/// ```
/// use eclipse::input::parse_input;
/// let cmd = parse_input("c (1,2)").unwrap();
/// ```
pub fn parse_input(input: &str) -> Result<Command, ParseError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Check for special commands
    match input.to_lowercase().as_str() {
        "help" | "h" | "?" => return Ok(Command::Help),
        "quit" | "exit" | "q" => return Ok(Command::Quit),
        "moves" | "m" => return Ok(Command::ShowMoves),
        _ => {}
    }

    // Parse move commands
    // First, check if it starts with 'c' or 's'
    let first_char = input.chars().next().unwrap().to_lowercase().to_string();

    match first_char.as_str() {
        "c" => parse_comet_move(input),
        "s" => parse_satellite_move(input),
        _ => {
            let parts: Vec<&str> = input.split_whitespace().collect();
            Err(ParseError::UnknownCommand(parts[0].to_string()))
        }
    }
}

/// Parses a comet move command: c (q,r)
fn parse_comet_move(input: &str) -> Result<Command, ParseError> {
    // Remove the 'c' or 'C' prefix and trim
    let input = input[1..].trim();

    if input.is_empty() {
        return Err(ParseError::InvalidFormat(
            "Comet move format: c (q,r)".to_string()
        ));
    }

    // Count the number of opening parentheses
    let paren_count = input.chars().filter(|&c| c == '(').count();

    // If there's more than one coordinate pair, it's an invalid format
    if paren_count > 1 {
        return Err(ParseError::InvalidFormat(
            "Comet move format: c (q,r)".to_string()
        ));
    }

    // Extract coordinates (parse_hex_coords will validate parentheses exist)
    let coords = parse_hex_coords(input)?;
    Ok(Command::MoveComet { target: coords })
}

/// Parses a satellite move command: s [type] [chain] (q1,r1) (q2,r2)
/// where type is optional ('s' for short, 'l' for long) and chain is optional ('a'-'j')
fn parse_satellite_move(input: &str) -> Result<Command, ParseError> {
    // Remove the 's' or 'S' prefix and trim
    let input = input[1..].trim();

    // Find where the coordinates start (first occurrence of '(')
    let first_paren_idx = input.find('(').ok_or_else(|| {
        ParseError::InvalidFormat("Satellite move format: s [type] [chain] (q1,r1) (q2,r2)".to_string())
    })?;

    // Extract the prefix before coordinates
    let prefix = input[..first_paren_idx].trim();
    let coords_str = &input[first_paren_idx..];

    // Parse optional type and chain letter from prefix
    let mut chain_type = None;
    let mut chain_letter = None;

    if !prefix.is_empty() {
        let prefix_parts: Vec<&str> = prefix.split_whitespace().collect();

        for part in prefix_parts {
            if part.len() == 1 {
                let ch = part.chars().next().unwrap().to_lowercase().next().unwrap();

                // Check if it's a valid type letter
                if ch == 's' || ch == 'l' {
                    if chain_type.is_none() {
                        chain_type = Some(ch);
                    } else {
                        return Err(ParseError::InvalidFormat(
                            "Duplicate chain type specified".to_string()
                        ));
                    }
                }
                // Check if it's a valid chain letter
                else if ('a'..='j').contains(&ch) {
                    if chain_letter.is_none() {
                        chain_letter = Some(ch);
                    } else {
                        return Err(ParseError::InvalidFormat(
                            "Duplicate chain letter specified".to_string()
                        ));
                    }
                } else {
                    return Err(ParseError::InvalidFormat(
                        format!("Invalid parameter '{}'. Expected: s/l (type) or a-j (chain)", part)
                    ));
                }
            } else {
                return Err(ParseError::InvalidFormat(
                    format!("Invalid parameter '{}'. Expected single character", part)
                ));
            }
        }
    }

    // Find the two sets of parentheses
    let coords = extract_two_coords(coords_str)?;
    Ok(Command::MoveSatellite {
        from: coords.0,
        to: coords.1,
        chain_type,
        chain_letter,
    })
}

/// Extracts two coordinate pairs from a string like "(q1,r1) (q2,r2)"
fn extract_two_coords(s: &str) -> Result<(Hex, Hex), ParseError> {
    // Find all matches of the pattern (q,r)
    let mut coords = Vec::new();
    let mut current = s;

    while let Some(start) = current.find('(') {
        if let Some(end) = current.find(')') {
            if end > start {
                let coord_str = &current[start..=end];
                coords.push(parse_hex_coords(coord_str)?);
                current = &current[end+1..];
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if coords.len() != 2 {
        return Err(ParseError::InvalidFormat(
            "Satellite move format: s (q1,r1) (q2,r2)".to_string()
        ));
    }

    Ok((coords[0], coords[1]))
}

/// Parses hex coordinates from a string like "(q,r)"
fn parse_hex_coords(s: &str) -> Result<Hex, ParseError> {
    let s = s.trim();

    // Check that the string starts with '(' and ends with ')'
    if !s.starts_with('(') || !s.ends_with(')') {
        return Err(ParseError::InvalidCoordinates(
            "Expected format: (q,r)".to_string()
        ));
    }

    // Remove parentheses
    let s = &s[1..s.len()-1].trim();

    // Split by comma
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();

    if parts.len() != 2 {
        return Err(ParseError::InvalidCoordinates(
            "Expected format: (q,r)".to_string()
        ));
    }

    // Parse integers
    let q = parts[0].parse::<i32>().map_err(|_| {
        ParseError::InvalidCoordinates(format!("'{}' is not a valid integer", parts[0]))
    })?;

    let r = parts[1].parse::<i32>().map_err(|_| {
        ParseError::InvalidCoordinates(format!("'{}' is not a valid integer", parts[1]))
    })?;

    Ok(Hex::new(q, r))
}

/// Converts a Command into a Move by validating it against the game state
///
/// # Arguments
/// * `command` - The parsed command
/// * `game` - The current game state
///
/// # Returns
/// * `Ok(Move)` if the command is valid and represents a legal move
/// * `Err(String)` with an error message if the command is invalid
pub fn command_to_move(command: Command, game: &GameState) -> Result<Move, String> {
    match command {
        Command::MoveComet { target } => {
            // Check if this is a valid comet move
            let comet_pos = if game.current_turn == crate::states::Player::Light {
                game.comet_light
            } else {
                game.comet_dark
            };

            // Verify target is adjacent to current comet position
            if !comet_pos.neighbors().contains(&target) {
                return Err(format!(
                    "Comet can only move to adjacent hexes. Target ({}, {}) is not adjacent to current position ({}, {})",
                    target.q, target.r, comet_pos.q, comet_pos.r
                ));
            }

            // Verify the move is in the legal moves list
            let legal_moves = game.get_legal_moves();
            let comet_move = Move::MoveComet(target);

            if legal_moves.iter().any(|m| matches!(m, Move::MoveComet(t) if *t == target)) {
                Ok(comet_move)
            } else {
                Err(format!(
                    "Cannot move comet to ({}, {}). Position may be occupied or blocked by opponent chain.",
                    target.q, target.r
                ))
            }
        }

        Command::MoveSatellite { from, to, chain_type, chain_letter } => {
            // Find which chain has a satellite at 'from' position
            let chain_id = game.occupied.get(&from)
                .and_then(|occ| match occ {
                    Occupant::Satellite(id, player) if *player == game.current_turn => Some(*id),
                    _ => None,
                })
                .ok_or_else(|| format!(
                    "No satellite of yours at position ({}, {})",
                    from.q, from.r
                ))?;

            // Validate chain letter if provided
            if let Some(letter) = chain_letter {
                use crate::states::ChainId;
                let expected_id = ChainId::from_letter(letter)
                    .ok_or_else(|| format!("Invalid chain letter '{}'", letter))?;

                if chain_id != expected_id {
                    let actual_letter = chain_id.to_letter();
                    return Err(format!(
                        "Chain mismatch: position ({}, {}) belongs to chain '{}', but you specified '{}'",
                        from.q, from.r, actual_letter, letter
                    ));
                }
            }

            // Validate chain type if provided
            if let Some(type_letter) = chain_type {
                use crate::states::ChainType;
                let expected_type = ChainType::from_letter(type_letter)
                    .ok_or_else(|| format!("Invalid chain type '{}'", type_letter))?;

                let chain = game.chains.get(&chain_id).unwrap();
                if chain.ctype != expected_type {
                    let actual_type = chain.ctype.to_letter();
                    return Err(format!(
                        "Chain type mismatch: chain '{}' is '{}' type, but you specified '{}'",
                        chain_id.to_letter(), actual_type, type_letter
                    ));
                }
            }

            // Create the move
            let satellite_move = Move::MoveSatellite {
                chain_id,
                old_pos: from,
                new_pos: to,
            };

            // Verify the move is in the legal moves list
            let legal_moves = game.get_legal_moves();

            if legal_moves.iter().any(|m| {
                matches!(m, Move::MoveSatellite { chain_id: id, old_pos, new_pos }
                    if *id == chain_id && *old_pos == from && *new_pos == to)
            }) {
                Ok(satellite_move)
            } else {
                // Provide more specific error messages
                let chain = game.chains.get(&chain_id).unwrap();

                // Check if chain is immobilized
                if game.get_legal_moves().iter().all(|m| {
                    !matches!(m, Move::MoveSatellite { chain_id: id, .. } if *id == chain_id)
                }) {
                    return Err(format!(
                        "Chain [{}] is immobilized (crossed by opponent chain) and cannot move",
                        chain_id.0
                    ));
                }

                // Check if 'from' is part of the chain
                if chain.head != from && chain.tail != from {
                    return Err(format!(
                        "Position ({}, {}) is not part of chain [{}]",
                        from.q, from.r, chain_id.0
                    ));
                }

                // Check if 'to' is occupied
                if game.occupied.contains_key(&to) {
                    return Err(format!(
                        "Target position ({}, {}) is already occupied",
                        to.q, to.r
                    ));
                }

                // Check if 'to' is on the board
                if !to.is_on_board() {
                    return Err(format!(
                        "Target position ({}, {}) is not on the board",
                        to.q, to.r
                    ));
                }

                // Check chain length constraint
                let other_end = if chain.head == from { chain.tail } else { chain.head };
                let new_length = to.distance(&other_end);
                if new_length > chain.ctype.max_len() {
                    return Err(format!(
                        "Move would stretch chain beyond maximum length ({} > {})",
                        new_length, chain.ctype.max_len()
                    ));
                }

                Err(format!(
                    "Cannot move satellite from ({}, {}) to ({}, {}). This is not a legal move.",
                    from.q, from.r, to.q, to.r
                ))
            }
        }

        _ => Err("Command is not a move".to_string()),
    }
}

/// Displays the help message with move notation and commands
pub fn show_help() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                   ECLIPSE - HELP                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("OBJECTIVE:");
    println!("  Immobilize your opponent's comet by blocking all adjacent hexes.");
    println!();
    println!("MOVE NOTATION:");
    println!("  c (q,r)                         - Move your comet to hex (q, r)");
    println!("                                    Example: c (1,2)");
    println!();
    println!("  s [type] [chain] (q1,r1) (q2,r2) - Move satellite from (q1,r1) to (q2,r2)");
    println!("                                    Examples:");
    println!("                                      s (2,1) (3,2)");
    println!("                                      s l a (2,1) (3,2)");
    println!();
    println!("    Optional parameters:");
    println!("      [type]  : s = Short chain, l = Long chain");
    println!("      [chain] : a-e = Light chains, f-j = Dark chains");
    println!("                (shown on board, validates if provided)");
    println!();
    println!("COMMANDS:");
    println!("  help, h, ?           - Show this help message");
    println!("  moves, m             - Display all legal moves");
    println!("  quit, exit, q        - Exit the game");
    println!();
    println!("RULES:");
    println!("  • Comets move to adjacent hexes (cannot cross opponent chains)");
    println!("  • Satellites move within chain length (Short=2, Long=3)");
    println!("  • Chains that cross opponent chains are immobilized");
    println!("  • You win when opponent's comet has no legal moves");
    println!();
}

/// Displays all legal moves for the current player
pub fn show_legal_moves(game: &GameState) {
    let moves = game.get_legal_moves();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  LEGAL MOVES for {:?} ({} total)", game.current_turn, moves.len());
    println!("═══════════════════════════════════════════════════════════════");

    // Group moves by type
    let mut comet_moves = Vec::new();
    let mut satellite_moves = Vec::new();

    for mv in &moves {
        match mv {
            Move::MoveComet(_) => comet_moves.push(mv),
            Move::MoveSatellite { .. } => satellite_moves.push(mv),
        }
    }

    // Display comet moves
    if !comet_moves.is_empty() {
        println!("\nCOMET MOVES ({}):", comet_moves.len());
        for mv in comet_moves {
            if let Move::MoveComet(target) = mv {
                println!("  c ({},{})", target.q, target.r);
            }
        }
    }

    // Display satellite moves grouped by chain
    if !satellite_moves.is_empty() {
        println!("\nSATELLITE MOVES ({}):", satellite_moves.len());

        // Group by chain_id
        use std::collections::HashMap;
        let mut moves_by_chain: HashMap<ChainId, Vec<&Move>> = HashMap::new();

        for mv in &satellite_moves {
            if let Move::MoveSatellite { chain_id, .. } = mv {
                moves_by_chain.entry(*chain_id).or_insert_with(Vec::new).push(mv);
            }
        }

        // Display moves for each chain
        let mut chain_ids: Vec<_> = moves_by_chain.keys().collect();
        chain_ids.sort_by_key(|id| id.0);

        for chain_id in chain_ids {
            let chain_moves = &moves_by_chain[chain_id];
            let chain = game.chains.get(chain_id).unwrap();
            let chain_letter = chain_id.to_letter();
            let type_letter = chain.ctype.to_letter();

            println!("  Chain '{}' [{}] ({:?}, {} moves, satellites at ({},{}) and ({},{})):",
                chain_letter, chain_id.0, chain.ctype, chain_moves.len(),
                chain.head.q, chain.head.r, chain.tail.q, chain.tail.r);

            for mv in chain_moves {
                if let Move::MoveSatellite { old_pos, new_pos, .. } = mv {
                    println!("    s {} {} ({},{}) ({},{})",
                        type_letter, chain_letter,
                        old_pos.q, old_pos.r, new_pos.q, new_pos.r);
                }
            }
        }
    }

    if moves.is_empty() {
        println!("\n  No legal moves available!");
    }

    println!();
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comet_move() {
        let cmd = parse_input("c (1,2)").unwrap();
        assert_eq!(cmd, Command::MoveComet { target: Hex::new(1, 2) });

        let cmd = parse_input("c (0,0)").unwrap();
        assert_eq!(cmd, Command::MoveComet { target: Hex::new(0, 0) });

        let cmd = parse_input("c (-3,4)").unwrap();
        assert_eq!(cmd, Command::MoveComet { target: Hex::new(-3, 4) });
    }

    #[test]
    fn test_parse_satellite_move() {
        let cmd = parse_input("s (1,2) (3,4)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(1, 2),
            to: Hex::new(3, 4),
            chain_type: None,
            chain_letter: None,
        });

        let cmd = parse_input("s (0,0) (-1,-1)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(0, 0),
            to: Hex::new(-1, -1),
            chain_type: None,
            chain_letter: None,
        });
    }

    #[test]
    fn test_parse_satellite_move_with_type() {
        let cmd = parse_input("s l (1,2) (3,4)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(1, 2),
            to: Hex::new(3, 4),
            chain_type: Some('l'),
            chain_letter: None,
        });

        let cmd = parse_input("s s (0,0) (-1,-1)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(0, 0),
            to: Hex::new(-1, -1),
            chain_type: Some('s'),
            chain_letter: None,
        });
    }

    #[test]
    fn test_parse_satellite_move_with_chain() {
        let cmd = parse_input("s a (1,2) (3,4)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(1, 2),
            to: Hex::new(3, 4),
            chain_type: None,
            chain_letter: Some('a'),
        });

        let cmd = parse_input("s f (0,0) (-1,-1)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(0, 0),
            to: Hex::new(-1, -1),
            chain_type: None,
            chain_letter: Some('f'),
        });
    }

    #[test]
    fn test_parse_satellite_move_with_type_and_chain() {
        let cmd = parse_input("s l a (1,2) (3,4)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(1, 2),
            to: Hex::new(3, 4),
            chain_type: Some('l'),
            chain_letter: Some('a'),
        });

        let cmd = parse_input("s s b (0,0) (-1,-1)").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(0, 0),
            to: Hex::new(-1, -1),
            chain_type: Some('s'),
            chain_letter: Some('b'),
        });
    }

    #[test]
    fn test_parse_help_commands() {
        assert_eq!(parse_input("help").unwrap(), Command::Help);
        assert_eq!(parse_input("h").unwrap(), Command::Help);
        assert_eq!(parse_input("?").unwrap(), Command::Help);
    }

    #[test]
    fn test_parse_quit_commands() {
        assert_eq!(parse_input("quit").unwrap(), Command::Quit);
        assert_eq!(parse_input("exit").unwrap(), Command::Quit);
        assert_eq!(parse_input("q").unwrap(), Command::Quit);
    }

    #[test]
    fn test_parse_show_moves() {
        assert_eq!(parse_input("moves").unwrap(), Command::ShowMoves);
        assert_eq!(parse_input("m").unwrap(), Command::ShowMoves);
    }

    #[test]
    fn test_parse_empty_input() {
        assert!(matches!(parse_input(""), Err(ParseError::EmptyInput)));
        assert!(matches!(parse_input("   "), Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_parse_unknown_command() {
        assert!(matches!(
            parse_input("foo bar"),
            Err(ParseError::UnknownCommand(_))
        ));
    }

    #[test]
    fn test_parse_invalid_format() {
        // Comet move with wrong number of arguments
        assert!(matches!(
            parse_input("c"),
            Err(ParseError::InvalidFormat(_))
        ));

        assert!(matches!(
            parse_input("c (1,2) (3,4)"),
            Err(ParseError::InvalidFormat(_))
        ));

        // Satellite move with wrong number of arguments
        assert!(matches!(
            parse_input("s (1,2)"),
            Err(ParseError::InvalidFormat(_))
        ));
    }

    #[test]
    fn test_parse_invalid_coordinates() {
        // Non-numeric coordinates
        assert!(matches!(
            parse_input("c (a,b)"),
            Err(ParseError::InvalidCoordinates(_))
        ));

        // Missing comma
        assert!(matches!(
            parse_input("c (1 2)"),
            Err(ParseError::InvalidCoordinates(_))
        ));

        // Wrong format
        assert!(matches!(
            parse_input("c 1,2"),
            Err(ParseError::InvalidCoordinates(_))
        ));
    }

    #[test]
    fn test_parse_whitespace_handling() {
        // Extra whitespace should be handled
        let cmd = parse_input("  c   (  1  ,  2  )  ").unwrap();
        assert_eq!(cmd, Command::MoveComet { target: Hex::new(1, 2) });

        let cmd = parse_input("s  ( 0 , 0 )  ( 1 , 1 )").unwrap();
        assert_eq!(cmd, Command::MoveSatellite {
            from: Hex::new(0, 0),
            to: Hex::new(1, 1),
            chain_type: None,
            chain_letter: None,
        });
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(parse_input("C (1,2)").unwrap(), Command::MoveComet { target: Hex::new(1, 2) });
        assert_eq!(parse_input("S (0,0) (1,1)").unwrap(), Command::MoveSatellite {
            from: Hex::new(0, 0),
            to: Hex::new(1, 1),
            chain_type: None,
            chain_letter: None,
        });
        assert_eq!(parse_input("S L A (0,0) (1,1)").unwrap(), Command::MoveSatellite {
            from: Hex::new(0, 0),
            to: Hex::new(1, 1),
            chain_type: Some('l'),
            chain_letter: Some('a'),
        });
        assert_eq!(parse_input("HELP").unwrap(), Command::Help);
        assert_eq!(parse_input("QUIT").unwrap(), Command::Quit);
    }
}
