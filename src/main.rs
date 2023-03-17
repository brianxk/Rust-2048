fn main() {
    let game = model::Game::new();

    println!("{:?}", game);
}

mod model {
    const BOARD_DIMENSION: usize = 4;

    #[derive(Debug)]
    pub struct Game {
        board: [[u32; BOARD_DIMENSION]; BOARD_DIMENSION],
    }

    impl Game {
        pub fn new() -> Game {
            Game {
                board: [[0; BOARD_DIMENSION]; BOARD_DIMENSION],
            }
        }
    }
}

