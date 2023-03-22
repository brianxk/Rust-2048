mod model;

fn main() {
    for i in 0..15 {
        println!("New game . . .");
        let mut game = model::Game::new();
        game.print_board();
    }
}

