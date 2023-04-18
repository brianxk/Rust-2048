use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom};
use std::collections::LinkedList;

const BOARD_DIMENSION: usize = 4;
const NUM_TILES: usize = BOARD_DIMENSION * BOARD_DIMENSION;

struct Tile {
    value: u32,
    id: u8,
}

pub struct Game {
    board: [[Option<Tile>; BOARD_DIMENSION]; BOARD_DIMENSION],
    new_tile_params: NewTileParams,
    free_slots: Vec<(usize, usize)>,
    pub score: u64,
    tile_ids: LinkedList<u8>,
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

pub enum InputError {
    InvalidDirectionError,
    MovementNotPossibleError,
}

impl Game {
    /// Generates a new game board in a ready-to-play state.
    ///
    /// This means that the board will be empty save for two starting tiles.
    ///
    /// The two tiles will either both be 2's or one 2 and one 4, always in random positions.
    pub fn new() -> Game {
        const EMPTY_TILE: Option<Tile> = None;
        const EMPTY_ROW: [Option<Tile>; BOARD_DIMENSION] = [EMPTY_TILE; BOARD_DIMENSION];
        let tile_ids: [u8; NUM_TILES] = std::array::from_fn(|i| i as u8);

        let mut game = Game {
            board: [EMPTY_ROW; BOARD_DIMENSION],
            new_tile_params: NewTileParams::new(),
            free_slots: Vec::with_capacity(BOARD_DIMENSION * BOARD_DIMENSION),
            score: 0,
            tile_ids: LinkedList::from(tile_ids),
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

        // Construct `Tile` structs from the randomly generated numbers
        let first_tile = Tile {
            value: first_tile,
            id: *game.tile_ids.iter().next().expect("Error retrieving next tile ID."),
        };

        let second_tile = Tile {
            value: second_tile,
            id: *game.tile_ids.iter().next().expect("Error retrieving next tile ID."),
        };

        let first_tile_pos = game.get_random_free_slot().expect("New game board, should not panic.");

        game.board[first_tile_pos.0][first_tile_pos.1] = Some(first_tile);

        let second_tile_pos = game.get_random_free_slot().expect("New game board, should not panic.");
        game.board[second_tile_pos.0][second_tile_pos.1] = Some(second_tile);

        game
    }

    /// Generates a new tile - either 2 or 4 according to the weights defined in
    /// `self.new_tile_params`
    fn generate_tile(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let dist = WeightedIndex::new(self.new_tile_params.tile_weights).unwrap();

        let tile = self.new_tile_params.tile_choices[dist.sample(&mut rng)];

        tile
    }

    /// Updates the list of free slots.
    fn update_free_slots(&mut self) {
        self.free_slots.clear();

        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if let None = self.board[row][col] {
                    self.free_slots.push((row, col));
                }
            }
        }
    }

    /// Returns the coordinates of a free board slot at random. 
    /// Will return `None` if no free slots exist, indicating the game is over.
    fn get_random_free_slot(&mut self) -> Option<(usize, usize)> {
        self.update_free_slots();

        let mut rng = rand::thread_rng();

        self.free_slots.choose(&mut rng).copied()
    }

    /// Prints a text representation of the game board to stdout.
    pub fn print_board(&self) {
        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                match &self.board[row][col] {
                    Some(u) => print!("{:^10}", u.value),
                    None => print!("{:^10}", '-'),
                }
            }
            println!();
        }
    }

    /// Receives the user's input.
    /// Result will either return the updated score or an error.
    /// Error can occur when the user's input is not a valid direction, or when movement is not
    /// possible.
    pub fn receive_input(&self, input: &String) -> Result<u64, InputError> {
        match input.to_lowercase().trim() {
            "k" => {
                println!("Up");
                Ok( 0 )
            }
            "j" => {
                println!("Down");
                Ok( 1 )
            }
            "h" => {
                println!("Left");
                Ok( 2 )
            }
            "l" => {
                println!("Right");
                Ok( 3 )
            }
            _ => Err(InputError::InvalidDirectionError),
        }

    }

    fn shift_tiles(&self) {

    }
    // Player makes a move: L, R, U, D
    // Game updates the board according to rules
    // 1) Start from the direction of movement
    // 2) Combine first, then move next tiles
    // 3) Update score and display board (should main call print or should updated board be
    //    returned?)
    //
    // Potential solutions:
    // 1) Create a method which takes a direction and axis ( row vs. col ) and shifts/merges the 4
    //    rows/columns in the prescribed direction

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Ensure that the generation of 2-tiles outnumbers the generation of 4-tiles 4:1 given a
    /// sufficiently large sample size and across multiple trials.
    fn test_new_tile_rng() {
        let game = Game::new();
        let num_trials = 100;

        for i in 0..num_trials {
            println!("Test iteration: {i}");

            let mut two_count = 0;
            let mut four_count = 0;

            const SAMPLE_SIZE: u32 = 10000;

            for _ in 0..SAMPLE_SIZE {
                let tile = game.generate_tile();

                if tile == game.new_tile_params.tile_choices[NewTileParams::TWO] {
                    two_count += 1;
                } else {
                    four_count += 1;
                }
            }

            let two_dist = two_count as f32 / SAMPLE_SIZE as f32;
            let four_dist = four_count as f32 / SAMPLE_SIZE as f32;

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

    #[test]
    /// Ensure that the maintainance and random selection of free slots is working correctly. 
    fn test_updating_and_randomly_selecting_free_slots() {
        let mut game = Game::new();
        const NUM_STARTING_TILES: usize = 2;

        // Ensure that number of starting tiles is correct.
        game.update_free_slots();
        assert_eq!(game.free_slots.len(), NUM_TILES - NUM_STARTING_TILES);

        // Fill all empty slots with placeholders.
        for _ in NUM_STARTING_TILES..NUM_TILES {
            let coord = game.get_random_free_slot();

            match coord {
                Some((row, col)) => game.board[row][col] = Some(Tile { value: 0, id: 0 }),
                None => panic!("Game board filled up unexpectedly."),
            }
        }

        // Ensure that all board slots are filled.
        game.update_free_slots();
        assert_eq!(game.free_slots.len(), 0);

        // Brute force assurance that all board slots are filled.
        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if let None = game.board[row][col] {
                    panic!("Free board slots remain after filling with placeholders.");
                }
            }
        }

        // Ensure that attempting to obtain a free slot when the board is full returns `None`.
        assert_eq!(game.get_random_free_slot(), None);
    }

    #[test]
    /// A new game should:
    ///
    /// 1) Be empty save for two initial tiles.
    /// 2) Have one 4-tile and one 2-tile -OR- two 2-tiles.
    ///
    /// Running multiple trials due the random nature of new game generation.
    fn test_new_game() {
        const NUM_STARTING_TILES: usize = 2;
        let num_trials = 100;

        for _ in 0..num_trials {
            let game = Game::new();
            let mut starting_tiles = Vec::new();

            for row in 0..BOARD_DIMENSION {
                for col in 0..BOARD_DIMENSION {
                    if let Some(u) = &game.board[row][col] {
                        starting_tiles.push(u);
                    }
                }
            }

            // Check that number of starting tiles is correct.
            assert_eq!(starting_tiles.len(), NUM_STARTING_TILES);
            
            // Check that starting tiles are valid.
            assert!(game.new_tile_params.tile_choices.contains(&starting_tiles[0].value));
            assert!(game.new_tile_params.tile_choices.contains(&starting_tiles[1].value));
            
            // Check condition 2)
            if starting_tiles[0].value == starting_tiles[1].value {
                assert_eq!(starting_tiles[0].value, game.new_tile_params.tile_choices[NewTileParams::TWO]);
            }
        }
    }
}

