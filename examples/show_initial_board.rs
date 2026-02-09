use eclipse::display::display_board;
use eclipse::states::GameState;

fn main() {
    let game = GameState::new();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              INITIAL BOARD SETUP                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("{}", display_board(&game));
}
