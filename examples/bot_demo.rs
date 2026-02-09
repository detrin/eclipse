use eclipse::bot::Bot;
use eclipse::randombot::RandomBot;
use eclipse::display::display_board;
use eclipse::states::{GameState, GameStatus, Player};

/// This example demonstrates two RandomBots playing against each other
fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              BOT vs BOT DEMONSTRATION                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut game = GameState::new();
    let light_bot = RandomBot::new(Player::Light);
    let dark_bot = RandomBot::new(Player::Dark);

    println!("Initial board:\n");
    println!("{}", display_board(&game));

    let max_turns = 100;
    let mut turn_count = 0;

    while game.status == GameStatus::InProgress && turn_count < max_turns {
        let bot: &dyn Bot = if game.current_turn == Player::Light {
            &light_bot
        } else {
            &dark_bot
        };

        println!("Turn {}: {:?}'s turn ({} legal moves available)",
                 turn_count + 1, game.current_turn, game.get_legal_moves().len());

        match bot.choose_move(&game) {
            Some(mv) => {
                match game.apply_move(mv) {
                    Ok(()) => {
                        // Display status messages
                        for msg in &game.status_messages {
                            println!("  {}", msg);
                        }
                        println!();
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
            }
            None => {
                println!("No legal moves available!");
                break;
            }
        }

        turn_count += 1;

        // Show board every 5 turns or at the end
        if turn_count % 5 == 0 || game.status != GameStatus::InProgress {
            println!("{}", display_board(&game));
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                     GAME OVER                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    match game.status {
        GameStatus::Won(winner) => {
            println!("🎉 {:?} wins after {} turns! 🎉\n", winner, turn_count);
        }
        GameStatus::InProgress => {
            println!("Game reached maximum turn limit ({} turns)\n", max_turns);
        }
    }

    println!("Final board:\n");
    println!("{}", display_board(&game));
}
