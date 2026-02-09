use eclipse::states::GameState;

fn main() {
    let game = GameState::new();
    let json = serde_json::to_string_pretty(&game).unwrap();
    println!("{}", json);
}
