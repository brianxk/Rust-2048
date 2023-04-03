use std::io;
use std::io::{Write};

mod model;

fn main() {
    let game = model::Game::new();

    println!("Welcome to rust-2048");

    let mut user_selection = String::new();

    loop {
        game.print_board();
        print!("Enter your next move: ");
        io::stdout().flush().expect("Error flushing output buffer.");

        io::stdin().read_line(&mut user_selection).expect("Error reading user input.");

        let points_scored = match game.receive_input(&user_selection) {
            Ok(s) => (),
            Err(e) => break,
        };

        user_selection.clear();
    }
}

