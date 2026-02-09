use eclipse::states::GameState;

fn main() {
    // Test with GameState::new()
    let game1 = GameState::new();
    let moves1 = game1.get_legal_moves();
    println!("Moves from GameState::new(): {}", moves1.len());
    
    // Test with JSON deserialization
    let json = std::fs::read_to_string("/tmp/initial_state.json").unwrap();
    let game2: GameState = serde_json::from_str(&json).unwrap();
    let moves2 = game2.get_legal_moves();
    println!("Moves from JSON deserialization: {}", moves2.len());
}
