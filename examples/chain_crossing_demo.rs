use eclipse::*;

fn main() {
    let mut game = states::GameState::new();

    println!("=== Eclipse Chain Crossing Demonstration ===\n");
    println!("{}", display::display_board(&game));

    // Make some moves to demonstrate chain crossing
    println!("Making moves to create a chain crossing...\n");

    // Move a Light chain satellite to create a potential crossing
    if let Some(chain) = game.chains.values().find(|c| c.owner == states::Player::Light && c.id.0 == 4) {
        let mv = moves::Move::MoveSatellite {
            chain_id: chain.id,
            old_pos: board::Hex::new(2, -1),
            new_pos: board::Hex::new(0, -1),
        };

        match game.apply_move(mv) {
            Ok(()) => println!("Move 1: Light chain moved"),
            Err(e) => println!("Move 1 failed: {}", e),
        }
    }

    // Move Dark chain to cross it
    if let Some(chain) = game.chains.values().find(|c| c.owner == states::Player::Dark && c.id.0 == 8) {
        let mv = moves::Move::MoveSatellite {
            chain_id: chain.id,
            old_pos: board::Hex::new(-3, 1),
            new_pos: board::Hex::new(-1, 1),
        };

        match game.apply_move(mv) {
            Ok(()) => println!("Move 2: Dark chain moved"),
            Err(e) => println!("Move 2 failed: {}", e),
        }
    }

    // Display the board with crossed chains
    println!("\n{}", display::display_board(&game));

    println!("\n=== End of Demonstration ===");
}
