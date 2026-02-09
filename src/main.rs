mod board;
mod moves;
mod states;
mod display;
mod input;
mod bot;
mod randombot;
mod simplebot;
mod minimaxbot;

use states::{GameState, GameStatus, Player};
use display::display_board;
use input::{parse_input, command_to_move, show_help, show_legal_moves, Command};
use bot::Bot;
use randombot::RandomBot;
use simplebot::SimpleBot;
use minimaxbot::{MinimaxBot, Difficulty};
use std::io::{self, Write};

// =============================================================================
// GAME MODE SELECTION
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameMode {
    HumanVsHuman,
    HumanVsRandomBot,
    HumanVsSimpleBot,
    HumanVsMinimaxEasy,
    HumanVsMinimaxMedium,
    HumanVsMinimaxHard,
}

fn show_main_menu() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    SELECT GAME MODE                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  1. Human vs Human");
    println!("  2. Human vs Random Bot");
    println!("  3. Human vs Simple Bot (Strategic)");
    println!("  4. Human vs Minimax Bot (Easy)");
    println!("  5. Human vs Minimax Bot (Medium)");
    println!("  6. Human vs Minimax Bot (Hard)");
    println!();
    print!("Enter your choice (1-6): ");
    io::stdout().flush().unwrap();
}

fn get_game_mode() -> GameMode {
    loop {
        show_main_menu();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input. Please try again.\n");
            continue;
        }

        match input.trim() {
            "1" => {
                println!("\n✓ Starting Human vs Human game\n");
                return GameMode::HumanVsHuman;
            }
            "2" => {
                println!("\n✓ Starting Human vs Random Bot game\n");
                println!("You will play as Light (right side), Bot plays as Dark (left side)\n");
                return GameMode::HumanVsRandomBot;
            }
            "3" => {
                println!("\n✓ Starting Human vs Simple Bot (Strategic) game\n");
                println!("You will play as Light (right side), SimpleBot plays as Dark (left side)\n");
                println!("The SimpleBot uses strategic heuristics to make intelligent moves.\n");
                return GameMode::HumanVsSimpleBot;
            }
            "4" => {
                println!("\n✓ Starting Human vs Minimax Bot (Easy) game\n");
                println!("You will play as Light (right side), MinimaxBot plays as Dark (left side)\n");
                println!("The MinimaxBot uses minimax algorithm with alpha-beta pruning (Depth: 2).\n");
                return GameMode::HumanVsMinimaxEasy;
            }
            "5" => {
                println!("\n✓ Starting Human vs Minimax Bot (Medium) game\n");
                println!("You will play as Light (right side), MinimaxBot plays as Dark (left side)\n");
                println!("The MinimaxBot uses minimax algorithm with alpha-beta pruning (Depth: 3).\n");
                return GameMode::HumanVsMinimaxMedium;
            }
            "6" => {
                println!("\n✓ Starting Human vs Minimax Bot (Hard) game\n");
                println!("You will play as Light (right side), MinimaxBot plays as Dark (left side)\n");
                println!("The MinimaxBot uses minimax algorithm with alpha-beta pruning (Depth: 4).\n");
                println!("⚠ Warning: Hard difficulty may take longer to compute moves.\n");
                return GameMode::HumanVsMinimaxHard;
            }
            _ => {
                println!("\nInvalid choice. Please enter 1-6.\n");
            }
        }
    }
}

// =============================================================================
// MAIN - INTERACTIVE GAME LOOP
// =============================================================================

fn main() {
    // Create a fresh game state - no persistence to disk
    // Game state exists only in memory and is dropped when the program exits
    let mut game = GameState::new();

    // Print welcome banner
    print_welcome();

    // Get game mode selection
    let game_mode = get_game_mode();

    // Create bot if needed (using Box<dyn Bot> for trait objects)
    let bot: Option<Box<dyn Bot>> = match game_mode {
        GameMode::HumanVsRandomBot => Some(Box::new(RandomBot::new(Player::Dark))),
        GameMode::HumanVsSimpleBot => Some(Box::new(SimpleBot::new(Player::Dark))),
        GameMode::HumanVsMinimaxEasy => Some(Box::new(MinimaxBot::new(Player::Dark, Difficulty::Easy))),
        GameMode::HumanVsMinimaxMedium => Some(Box::new(MinimaxBot::new(Player::Dark, Difficulty::Medium))),
        GameMode::HumanVsMinimaxHard => Some(Box::new(MinimaxBot::new(Player::Dark, Difficulty::Hard))),
        GameMode::HumanVsHuman => None,
    };

    // Track which player the bot is playing as (always Dark in current setup)
    let bot_player = if bot.is_some() { Some(Player::Dark) } else { None };

    // Validate the initial setup
    match game.validate_setup() {
        Ok(()) => println!("✓ Initial setup validation passed!\n"),
        Err(e) => {
            eprintln!("✗ Setup validation failed: {}", e);
            return;
        }
    }

    // Display the initial board
    println!("{}", display_board(&game));

    // Show initial instructions
    println!("Type 'help' for instructions, 'moves' to see legal moves, or enter a move.");
    println!();

    // Main game loop
    loop {
        // Check game status
        match game.status {
            GameStatus::Won(winner) => {
                println!("\n╔══════════════════════════════════════════════════════════════╗");
                println!("║                    GAME OVER!                                ║");
                println!("╚══════════════════════════════════════════════════════════════╝");
                println!("\n  🎉 {:?} wins! The opponent's comet is immobilized. 🎉\n", winner);
                println!("Type 'quit' to exit or press Ctrl+C.");
                println!();

                // Wait for quit command
                loop {
                    print!(">>> ");
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_err() {
                        break;
                    }

                    match parse_input(&input) {
                        Ok(Command::Quit) => break,
                        Ok(Command::Help) => show_help(),
                        _ => println!("Game is over. Type 'quit' to exit."),
                    }
                }
                break;
            }
            GameStatus::InProgress => {
                // Check if it's the bot's turn
                if let (Some(ref bot_instance), Some(bot_player_side)) = (&bot, bot_player) {
                    if game.current_turn == bot_player_side {
                        // Bot's turn - make a move automatically
                        println!("{} ({:?}) is thinking...", bot_instance.name(), game.current_turn);

                        match bot_instance.choose_move(&game) {
                            Some(mv) => {
                                // Apply the bot's move
                                match game.apply_move(mv.clone()) {
                                    Ok(()) => {
                                        println!();
                                        // Display status messages
                                        for msg in &game.status_messages {
                                            println!("{}", msg);
                                        }
                                        println!();

                                        // Display the updated board
                                        println!("{}", display_board(&game));
                                    }
                                    Err(e) => {
                                        eprintln!("Error: Bot made an invalid move: {}\n", e);
                                        break;
                                    }
                                }
                            }
                            None => {
                                println!("Bot has no legal moves available!");
                                break;
                            }
                        }
                        continue;
                    }
                }

                // Human player's turn
                print!("{:?}'s turn >>> ", game.current_turn);
                io::stdout().flush().unwrap();

                // Read user input
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() {
                    eprintln!("Error reading input");
                    continue;
                }

                // Parse the input
                let command = match parse_input(&input) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        println!("Error: {}\n", e);
                        continue;
                    }
                };

                // Handle the command
                match command {
                    Command::Help => {
                        show_help();
                        continue;
                    }

                    Command::ShowMoves => {
                        show_legal_moves(&game);
                        continue;
                    }

                    Command::Quit => {
                        println!("\nThanks for playing Eclipse! Goodbye.\n");
                        break;
                    }

                    Command::MoveComet { .. } | Command::MoveSatellite { .. } => {
                        // Convert command to move and validate
                        let mv = match command_to_move(command, &game) {
                            Ok(m) => m,
                            Err(e) => {
                                println!("Invalid move: {}\n", e);
                                continue;
                            }
                        };

                        // Apply the move
                        match game.apply_move(mv.clone()) {
                            Ok(()) => {
                                println!();
                                // Display status messages
                                for msg in &game.status_messages {
                                    println!("{}", msg);
                                }
                                println!();

                                // Display the updated board
                                println!("{}", display_board(&game));
                            }
                            Err(e) => {
                                println!("Error applying move: {}\n", e);
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn print_welcome() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      ECLIPSE                                 ║");
    println!("║           A Two-Player Abstract Strategy Game                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}
