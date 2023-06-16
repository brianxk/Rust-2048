use web_sys::HtmlElement;
use yew::prelude::*;
use yew::NodeRef;
use rust_2048::*;
use gloo_console::log;
use gloo::events::EventListener;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Element, window};
use std::rc::Rc;
use std::cell::RefCell;
use substring::Substring;

const BORDER_SPACING: u16 = 4;
const TILE_DIMENSION: u16 = 120;

#[derive(Properties, PartialEq)]
struct ScoreProps {
    score: u64,
}

#[function_component(GameBoard)]
fn game_board() -> Html {
    html! {
        <table>
            { for (0..BOARD_DIMENSION).map(|_| {
                 html! {
                     <tr>
                         { for (0..BOARD_DIMENSION).map(|_| {
                             html! {
                                 <td class="cell"/>
                             }
                         })}
                     </tr>
                 }
             })}
        </table>        
    }
}

#[derive(Properties, PartialEq)]
struct TileProps {
    value: u32,
    id: u8,
    background_color: String,
    left_offset: u16,
    top_offset: u16,
}

#[function_component(Tile)]
fn tile(props: &TileProps) -> Html {
    let tile_args = format!("--top: {}px; --left: {}px;", 
                           props.top_offset,
                           props.left_offset);

    let tile_id = props.id.to_string();

    html! {
        <div id={tile_id} class="tile cell" style={tile_args}>{props.value}</div>
    }
}

#[function_component(Content)]
fn content() -> Html {
    let game_state = Rc::new(RefCell::new(Game::new()));
    let game_state_for_listener = Rc::clone(&game_state);

    use_effect(move || {
        // Attach a keydown event listener to the document.
        let document = gloo::utils::document();
        let listener = EventListener::new(&document, "keydown", move |event| {
            let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
            match game_state_for_listener.borrow_mut().receive_input(&event.code()) {
                InputResult::Ok(tiles) => {
                    log!("Move successful");

                    let document = gloo::utils::document();

                    match document.query_selector_all("[class='tile cell']") {
                        Ok(node_list) => {
                            // log!("NodeList length: ", node_list.length());

                            for i in 0..node_list.length() {
                                // if let Some(node) = node_list.get(i) {
                                let node = node_list.get(i).unwrap();
                                // Cast Node to Element to access html/css properties
                                let tile = node.dyn_ref::<HtmlElement>().unwrap();
                                let tile_id = tile.get_attribute("id").unwrap().parse::<u8>().unwrap();
                                let computed_style = window().unwrap().get_computed_style(&tile).unwrap().unwrap();

                                // How to access css properties of an element
                                let top = computed_style.get_property_value("top").unwrap();
                                let left = computed_style.get_property_value("left").unwrap();

                                match get_tile_by_id(&tiles, tile_id) {
                                    Some(updated_tile) => {
                                        let (new_top_offset, new_left_offset) = convert_to_pixels(updated_tile.row, updated_tile.col);

                                        let new_top_offset = format!("{}px", new_top_offset);
                                        let new_left_offset = format!("{}px", new_left_offset);

                                        log!("Tile ID:", tile_id, "Value:", tile.inner_text());
                                        log!("Old top:", top);
                                        log!("Old left:", left);

                                        log!("New i:", updated_tile.row);
                                        log!("New j:", updated_tile.col);

                                        // log!("New top:", &new_top_offset);
                                        // log!("New left:", &new_left_offset);


                                        tile.style().set_property("top", &new_top_offset);
                                        tile.style().set_property("left", &new_left_offset);


                                        window().unwrap().get_computed_style(&tile).unwrap();
                                    },
                                    None => {
                                        // Tile with specified id doesn't exist anymore - likely
                                        // got merged
                                        log!("Updated tile not found.");
                                    }
                                }
                            }
                        }
                        Err(e) => log!("NodeList could not be found."),
                    }
                },
                InputResult::Err(InvalidMove) => {
                    log!("Move unsuccessful");
                },
            }
        });

        // Called when the component is unmounted.  The closure has to hold on to `listener`, because if it gets
        // dropped, `gloo` detaches it from the DOM. So it's important to do _something_, even if it's just dropping it.
        || drop(listener)
    });

    // Not actually keeping track of the state of any variable.
    // use_state is being used to trigger a re-render whenever the 'New Game' button is clicked.
    // `true` is merely a placeholder value.
    let new_game = use_state(|| 0);
    let onclick = {
        let new_game = new_game.clone();
        Callback::from(move |_| new_game.set(*new_game + 1))
    };

    html! {
        <div class="content">
            <div class="metadata">
                <Score score={game_state.borrow().score}/>
                <button class="new-game" onclick={onclick}>{ "New Game" }</button>
                <button class="move-tile">{ "Move Tile" }</button>
            </div>
            <div class="board-container">
                <GameBoard/>
                { 
                    for game_state.borrow().get_tiles().iter().map(|tile| {
                        let value = tile.value;
                        let background_color = tile.background_color.clone();
                        let id = tile.id;
                        let (top_offset, left_offset) = 
                            convert_to_pixels(tile.row, tile.col);
                        let animation_key = *new_game.clone();

                        html! {
                            <Tile 
                                key={animation_key}
                                value={value}
                                background_color={background_color}
                                id={id}
                                top_offset={top_offset}
                                left_offset={left_offset}
                            />
                        }
                    })
                }
            </div>
        </div>
    }
}

// TODO: 
// If I know the starting and ending position of a tile after a move, I'm assuming I can program
// the animation.
//
// Problem with recycling IDs is that a new tile could use the ID of a tile that was recently
// merged. I don't remember how I implemented this - look into this next time.
//
// Possibly worth implementing a method that returns a vector of all current tiles. If a tile
// doesn't exist after a move, it was merged. If a tile with the same ID but different number
// exists, the previous tile was merged and its value/color need to be updated.
//
// Not all tiles will be moved, and for those the animation should not occur.
//
// Is it worth implementing this method, or is it better to return a list of all moved tiles from
// the receive_input() method? In the latter approach I don't need to search through and see what
// tiles ended up where manually. I could also include info about which tiles were removed due to a
// merge.
//
// let row = &props.game_state.borrow().board[i];
// let row = &GAME_STATE.board[i];
// let tile = &row[j];
// <td class="cell">{ tile.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "\u{00a0}".to_string()) }</td>

#[function_component(Score)]
fn score(props: &ScoreProps) -> Html {
    html! {
        <div class="score">{props.score}</div>
    }
}

#[function_component(Header)]
fn header() -> Html {
    html! {
        <div class="header">
            <br/>
            <h1 class="typed">{ "Welcome to 2048!" }</h1>
        </div>
    }
}

#[function_component(Footer)]
fn footer() -> Html {
    html! {
        <div class="footer">
            <p>
                { "This project is a Rust practice implementation of the "}
                <a href="https://play2048.co/" target="_blank">
                    { "2048 game" }
                </a>
                { " developed by Gabriele Cirulli." }
            </p>
            <br/>
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <Header/>
            <Content/>
            <Footer/>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

// Utility functions

/// Accepts a i, j pair of grid coordinates and returns the pixel offset equivalents for CSS positioning
fn convert_to_pixels(i: usize, j: usize) -> (u16, u16) {
    let i = i as u16;
    let j = j as u16;

    let top_offset = (BORDER_SPACING * (i + 1)) + (TILE_DIMENSION * i);
    let left_offset = (BORDER_SPACING * (j + 1)) + (TILE_DIMENSION * j);
    
    (top_offset, left_offset)
}

/// Accepts a top, left pair of pixel offsets and returns the grid coordinate equivalents for array indexing
/// This function will handle the String parsing, as such the offset values should be given "as is" e.g. "252px".
fn convert_to_indexes(top: &str, left: &str) -> (usize, usize) {
    let top = top.substring(0, top.chars().count() - 2).parse::<u16>().unwrap();
    let left = left.substring(0, left.chars().count() - 2).parse::<u16>().unwrap();
    
    (pixel_to_index(top), pixel_to_index(left))
}

/// Helper function for `convert_to_indexes` that will match pixel offsets to indexes
fn pixel_to_index(pixel_value: u16) -> usize {
    match pixel_value {
        4 => 0,
        128 => 1,
        252 => 2,
        376 => 3,
        _ => unreachable!(),
    }
}

fn get_tile_by_id<'a>(tiles: &Vec<&'a rust_2048::Tile>, id: u8) -> Option<&'a rust_2048::Tile> {
    for tile in tiles {
        if tile.id == id {
            return Some(tile)
        }
    }

    None
}

