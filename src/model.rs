use rand::{distributions::WeightedIndex, prelude::Distribution};

const BOARD_DIMENSION: usize = 4;

pub struct Game {
    board: [[Option<u32>; BOARD_DIMENSION]; BOARD_DIMENSION],
    new_tile_params: NewTileParams,
}

struct NewTileParams {
    tile_choices: [u32; 2],
    tile_weights: [u8; 2],
}

impl NewTileParams {
    // Constants that represent the index positions of 2 and 4 respectively.
    // Code that accesses the `tile_choices` and `tile_weights` arrays should use these constants
    // as indices.
    const TWO: usize = 0;
    const FOUR: usize = 1;

    fn new() -> Self {
        NewTileParams {
            // Probability of 2-tile vs. 4-tile should be roughly 4:1
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
        let game = Game {
            board: [[None; BOARD_DIMENSION]; BOARD_DIMENSION],
            new_tile_params: NewTileParams::new(),
        };

        // If 1st tile is 4, 2nd tile must be 2.
        // If 1st tile is 2, 2nd tile may either be 2 or 4.
        let first_tile = game.generate_tile();
        let second_tile;
        
        if first_tile == game.new_tile_params.tile_choices[NewTileParams::FOUR] {
            second_tile = game.new_tile_params.tile_choices[NewTileParams::TWO];
        } else {
            second_tile = game.generate_tile();
        }

        // First tile coordinates
        let row = rng.gen_range(0..BOARD_DIMENSION);
        let col = rng.gen_range(0..BOARD_DIMENSION);

        game
    }

    /// Generates a new tile - either 2 or 4 according to pre-defined weighted probability
    pub fn generate_tile(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let dist = WeightedIndex::new(self.new_tile_params.tile_weights).unwrap();

        let tile = self.new_tile_params.tile_choices[dist.sample(&mut rng)];

        tile
    }

    /// Prints a text representation of the game board to stdout.
    pub fn print_board(&self) {
        for row in self.board {
            println!("{:?}", row);
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

