use std::io;

mod model;

fn main() {
    let game = model::Game::new();

    println!("Welcome to rust-2048");

    let mut user_selection = String::new();

    loop {
        game.print_board();
        println!("Enter your next move: ");
        io::stdin().read_line(&mut user_selection).expect("Unable to read user input.");

        game.receive_input(&user_selection);

        user_selection.clear();
    }
}

