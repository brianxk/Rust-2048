
fn main() {
    let game = model::Game::new();
    game.print_board();
}

mod model {
    use rand::{Rng, distributions::WeightedIndex};

    const BOARD_DIMENSION: usize = 4;
    const EMPTY_SLOT: u32 = 0;
    const NEW_TILE_CHOICES: [u8; 2] = [2, 4];
    const NEW_TILE_WEIGHTS: [u8; 2] = [4, 1];

    pub struct Game {
        board: [[u32; BOARD_DIMENSION]; BOARD_DIMENSION],
    }

    impl Game {
        /// Generates a new game board that is empty except for two initial tiles.
        ///
        /// The two tiles will either both be 2's or one 2 and one 4, always in random positions.
        pub fn new() -> Game {
            let game = Game {
                board: [[EMPTY_SLOT; BOARD_DIMENSION]; BOARD_DIMENSION],
            };
            
            let mut rng = rand::thread_rng();

            // First tile coordinates
            let row = rng.gen_range(0..BOARD_DIMENSION);
            let col = rng.gen_range(0..BOARD_DIMENSION);

            // game.board[row][col]

            game
        }

        /// Prints a text representation of the game board to the standard output.
        pub fn print_board(&self) {
            for row in self.board {
                println!("{:?}", row);
            }
        }
    }
}

