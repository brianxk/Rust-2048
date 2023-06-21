use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom};
use std::collections::LinkedList;
use gloo_console::log;

pub const BOARD_DIMENSION: usize = 4;
const NUM_TILES: usize = BOARD_DIMENSION * BOARD_DIMENSION;

#[derive(PartialEq)]
pub struct Tile {
    pub value: u32,
    pub id: usize,
    pub background_color: String,
    pub row: usize,
    pub col: usize,
    pub merged: bool,
}

impl Tile {
    fn new(value: u32, id: usize, background_color: String, row: usize, col: usize) -> Tile {
        Tile {
            value,
            id,
            background_color,
            row,
            col,
            merged: false,
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "value: {}\nid: {}\nrow: {}\n col:{}",
               self.value,
               self.id,
               self.row,
               self.col)
    }
}

#[derive(PartialEq)]
pub struct Game {
    pub board: [[Option<Tile>; BOARD_DIMENSION]; BOARD_DIMENSION],
    new_tile_params: NewTileParams,
    free_slots: Vec<(usize, usize)>,
    pub score: u64,
    id_list: LinkedList<usize>,
}

/// Struct that holds the choices for new tiles and the probability with which they will appear.
#[derive(PartialEq)]
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

pub struct InvalidMove;

pub enum InputResult {
    // Ok(&'a Tile),
    Ok(usize),
    Err(InvalidMove),
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
        
        // Tile IDs will be recycled, but we are making the number of available IDs 1 greater than
        // the maximum number of tiles. This is because a new tile should not recycle an ID from a
        // tile that was just merged on the current turn. The edge case here is the entire board is
        // occupied with 16 tiles but a player move is still possible; in this case the new tile
        // created after this move will need a 17th ID to use.
        let tile_ids: [usize; NUM_TILES + 1] = std::array::from_fn(|i| i as usize);

        let mut game = Game {
            board: [EMPTY_ROW; BOARD_DIMENSION],
            new_tile_params: NewTileParams::new(),
            free_slots: Vec::with_capacity(BOARD_DIMENSION * BOARD_DIMENSION),
            score: 0,
            id_list: LinkedList::from(tile_ids),
        };

        // If first tile is 4, second tile must be 2.
        // If first tile is 2, second tile may either be 2 or 4.
        let first_tile_value = game.generate_tile();
        let second_tile_value;
        
        if first_tile_value == game.new_tile_params.tile_choices[NewTileParams::FOUR] {
            second_tile_value = game.new_tile_params.tile_choices[NewTileParams::TWO];
        } else {
            second_tile_value = game.generate_tile();
        }

        let first_tile_pos = game.get_random_free_slot().expect("New game board, should not panic.");
        let first_tile_id = game.get_id().unwrap();
        let first_tile = Tile::new(first_tile_value, first_tile_id, "pink".to_string(), first_tile_pos.0, first_tile_pos.1);
        game.board[first_tile_pos.0][first_tile_pos.1] = Some(first_tile);
        
        let second_tile_pos = game.get_random_free_slot().expect("New game board, should not panic.");
        let second_tile_id = game.get_id().unwrap();
        let second_tile = Tile::new(second_tile_value, second_tile_id, "pink".to_string(), second_tile_pos.0, second_tile_pos.1);
        game.board[second_tile_pos.0][second_tile_pos.1] = Some(second_tile);

        game
    }

    /// Returns the next available ID. Will return None if all IDs are used.
    fn get_id(&mut self) -> Option<usize> {
        self.id_list.pop_front()
    }

    /// Receives a vector of IDs to recycle
    fn recycle_ids(&mut self, ids: Vec<usize>) {
        for id in ids {
            self.id_list.push_back(id);
        }
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

    /// Returns a vec of all current tiles.
    pub fn get_tiles(&self) -> Vec<&Tile> {
        let mut tiles = Vec::new();

        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if let Some(tile) = &self.board[row][col] {
                    tiles.push(tile);
                }
            }
        }

        tiles
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

    /// Receives the user's input and slides tiles in the specified direction.
    pub fn receive_input(&mut self, input: &str) -> InputResult {
        let mut move_occurred = false;

        // i in the loops below represents the index difference between the Tile's starting slot
        // and its destination slot.
        // i will be incremented each time the Tile is shifted by one slot and until it can 
        // no longer be shifted.
        match input {
            "ArrowUp" | "KeyK" | "KeyW" => {
                for col in 0..BOARD_DIMENSION {
                    for row in 1..BOARD_DIMENSION {
                        let mut i = 1;

                        if let Some(tile) = self.board[row][col].take() {
                            // Loop until an occupied cell is found.
                            while row.checked_sub(i).is_some_and(|diff| self.board[diff][col].is_none()) {
                                i += 1;
                            }

                            if i > 1 {
                                move_occurred = true;
                            }

                            self.update_tile_and_board(tile, row - (i - 1), col);
                        }
                    }
                }
            },
            "ArrowDown" | "KeyJ" | "KeyS" => {
                for col in 0..BOARD_DIMENSION {
                    for row in (0..BOARD_DIMENSION - 1).rev() {
                        let mut i = 1;

                        if let Some(tile) = self.board[row][col].take() {
                            while row.checked_add_max(i, BOARD_DIMENSION).is_some_and(|sum| self.board[sum][col].is_none()) {
                                i += 1;
                            }

                            if i > 1 {
                                move_occurred = true;
                            }

                            self.update_tile_and_board(tile, row + (i - 1), col);
                        }
                    }
                }
            }
            "ArrowLeft" | "KeyH" | "KeyA" => {
                for row in 0..BOARD_DIMENSION {
                    for col in 1..BOARD_DIMENSION {
                        let mut i = 1;

                        if let Some(tile) = self.board[row][col].take() {
                            while col.checked_sub(i).is_some_and(|diff| self.board[row][diff].is_none()) {
                                i += 1
                            }

                            if i > 1 {
                                move_occurred = true;
                            }

                            self.update_tile_and_board(tile, row, col - (i - 1));
                        }
                    }
                }
            },
            "ArrowRight" | "KeyL" | "KeyD" => {
                for row in 0..BOARD_DIMENSION {
                    for col in (0..BOARD_DIMENSION - 1).rev() {
                        if let Some(tile) = self.board[row][col].take() {
                            let mut i = 1;

                            while col.checked_add_max(i, BOARD_DIMENSION).is_some_and(|sum| self.board[row][sum].is_none()) {
                                i += 1;
                            }

                            if i > 1 {
                                move_occurred = true;
                            }

                            self.update_tile_and_board(tile, row, col + (i - 1));
                        }
                    }
                }
            }
            _ => (),
        }

        match move_occurred {
            true => match self.get_random_free_slot() {
                Some((i, j)) => {
                    let new_id = self.get_id().unwrap();
                    let new_tile = Tile::new(self.generate_tile(), new_id, "lightcyan".to_string(), i, j);
                    self.board[i][j] = Some(new_tile);
                    // Create a systematic way to decide background color. Modulus?
                    // Consider re-doing the colorscheme of the board in this step.
                    // Tile merging: If the destination tile has the same value, we merge them.
                    // Need to figure out a way to have sequential animations. 
                    InputResult::Ok(new_id)
                },
                None => unreachable!(),
            }
                ,
            false => InputResult::Err(InvalidMove),
        }
    }

    /// Receives a tile, the new row and col indexes, and updates both the tile's internal row and
    /// col fields and places the tile in self.board's new location.
    fn update_tile_and_board(&mut self, mut tile: Tile, new_row: usize, new_col: usize) {
        tile.row = new_row;
        tile.col = new_col;

        self.board[new_row][new_col] = Some(tile);
    }
}

// Helper functions

trait CheckedAdd {
    fn checked_add_max(self, rhs: usize, max: usize) -> Option<usize>;
}

/// Similar to the builtin `checked_add()` method but allows for defining a custom max
impl CheckedAdd for usize {
    fn checked_add_max(self, rhs: Self, max: Self) -> Option<Self> {
        let sum = self + rhs;

        if sum < max {
            Some(sum)
        } else {
            None
        }
    }
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
                Some((row, col)) => game.board[row][col] = 
                    Some(Tile::new(0, 0, "orange".to_string(), row, col)),
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

