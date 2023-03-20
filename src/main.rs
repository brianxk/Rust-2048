mod model;

fn main() {
    let game = model::Game::new();
    game.print_board();
}

