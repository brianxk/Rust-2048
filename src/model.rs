use rand::{distributions::WeightedIndex, prelude::Distribution, Rng, seq::SliceRandom};

const BOARD_DIMENSION: usize = 4;

pub struct Game {
    board: [[Option<u32>; BOARD_DIMENSION]; BOARD_DIMENSION],
    new_tile_params: NewTileParams,
    free_slots: Vec<(usize, usize)>,
}

/// Struct that holds the choices for new tiles and the probability with which they will appear.
struct NewTileParams {
    tile_choices: [u32; 2],
    tile_weights: [u8; 2],
}

impl NewTileParams {
    /// Represents the index position for accessing parameters related to 2-tiles in the
    /// `tile_choices` and `tile_weights` arrays.
    const TWO: usize = 0;
    
    /// Represents the index position for accessing parameters related to 4-tiles in the
    /// `tile_choices` and `tile_weights` arrays.
    const FOUR: usize = 1;

    /// Initializes the default settings for new tile creation such that 2-tiles appear more
    /// frequently than 4-tiles at a 4:1 ratio.
    fn new() -> Self {
        NewTileParams {
            tile_choices: [2, 4],
            tile_weights: [4, 1],
        }
    }
}

impl Game {
    /// Generates a new game board in a ready-to-play state.
    ///
    /// This means that the board will be empty save for two starting tiles.
    ///
    /// The two tiles will either both be 2's or one 2 and one 4, always in random positions.
    pub fn new() -> Game {
        let mut game = Game {
            board: [[None; BOARD_DIMENSION]; BOARD_DIMENSION],
            new_tile_params: NewTileParams::new(),
            free_slots: Vec::with_capacity(BOARD_DIMENSION * BOARD_DIMENSION),
        };

        // If first tile is 4, second tile must be 2.
        // If first tile is 2, second tile may either be 2 or 4.
        let first_tile = game.generate_tile();
        let second_tile;
        
        if first_tile == game.new_tile_params.tile_choices[NewTileParams::FOUR] {
            second_tile = game.new_tile_params.tile_choices[NewTileParams::TWO];
        } else {
            second_tile = game.generate_tile();
        }

        let first_tile_pos = game.get_free_tile().expect("New game board, should not panic.");
        game.board[first_tile_pos.0][first_tile_pos.1] = Some(first_tile);

        let second_tile_pos = game.get_free_tile().expect("New game board, should not panic.");
        game.board[second_tile_pos.0][second_tile_pos.1] = Some(second_tile);

        game
    }

    /// Generates a new tile - either 2 or 4 according to the weights defined in
    /// `self.NewTileParams`
    pub fn generate_tile(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let dist = WeightedIndex::new(self.new_tile_params.tile_weights).unwrap();

        let tile = self.new_tile_params.tile_choices[dist.sample(&mut rng)];

        tile
    }

    /// Returns the coordinates of a free board slot at random. 
    pub fn get_free_tile(&mut self) -> Option<(usize, usize)> {
        // Update vector of free slots
        self.free_slots.clear();

        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if let None = self.board[row][col] {
                    self.free_slots.push((row, col));
                }
            }
        }

        let mut rng = rand::thread_rng();

        self.free_slots.choose(&mut rng).copied()
    }

    /// Prints a text representation of the game board to stdout.
    pub fn print_board(&self) {
        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                match self.board[row][col] {
                    Some(u) => print!("{u:^10}"),
                    None => print!("{:^10}", '-'),
                }
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tile_rng() {
        let game = Game::new();
        let num_trials = 100;

        for i in 0..num_trials {
            println!("Test iteration: {i}");

            let mut two_count = 0;
            let mut four_count = 0;

            const TEST_SAMPLE_SIZE: u32 = 10000;

            for _ in 0..TEST_SAMPLE_SIZE {
                let tile = game.generate_tile();

                if tile == game.new_tile_params.tile_choices[NewTileParams::TWO] {
                    two_count += 1;
                } else {
                    four_count += 1;
                }
            }

            let two_dist = two_count as f32 / TEST_SAMPLE_SIZE as f32;
            let four_dist = four_count as f32 / TEST_SAMPLE_SIZE as f32;

            let expected_ratio = game.new_tile_params.tile_weights[NewTileParams::TWO] as f32;
            let actual_ratio = two_dist / four_dist;

            // Run `cargo test -- --nocapture` to show stdout
            println!("Expected 2:4 ratio: {expected_ratio}:1");
            println!("Actual 2:4 ratio: {actual_ratio}:1");
            
            let error_margin = expected_ratio * 0.20;
            let expected_ratio_range = (expected_ratio - error_margin)..=(expected_ratio + error_margin);

            assert!(expected_ratio_range.contains(&actual_ratio));
        }
    }
}

