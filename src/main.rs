use yew::prelude::*;
use yew::NodeRef;
use rust_2048::*;
use gloo_console::log;
use gloo::events::EventListener;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, window};
use std::rc::Rc;
use std::cell::RefCell;
use substring::Substring;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use tokio::sync::mpsc::{self, Receiver};
use yew::platform::time::sleep;
use core::time::Duration;

const BORDER_SPACING: u16 = 4;
const TILE_DIMENSION: u16 = 120;
const COLORS: Colors = Colors::new();

#[wasm_bindgen(module = "/prevent_arrow_scrolling.js")]
extern "C" {
    fn preventDefaultScrolling();
}

#[function_component(GameBoard)]
fn game_board() -> Html {
    let table_style = format!("--table_background: {};", COLORS.board);
    let cell_style = format!("--cell_background: {};", COLORS.cell);

    html! {
        <table style={table_style}>
            { for (0..BOARD_DIMENSION).map(|_| {
                 html! {
                     <tr>
                         { for (0..BOARD_DIMENSION).map(|_| {
                             html! {
                                 <td class="cell" style={cell_style.clone()}/>
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
    id: usize,
    background_color: String,
    text_color: String,
    left_offset: u16,
    top_offset: u16,
}

#[function_component(Tile)]
fn tile(props: &TileProps) -> Html {
    let style_args = format!("--top: {}px; --left: {}px; --background_color: {}; --text_color: {}; font-size: {}", 
                           props.top_offset,
                           props.left_offset,
                           props.background_color,
                           props.text_color,
                           compute_font_size(&props.value.to_string()));

    let tile_id = props.id.to_string();

    html! {
        <div id={tile_id} class="tile cell" style={style_args}>{props.value}</div>
    }
}

async fn slide_tile(html_tile: &HtmlElement, game_tile: &rust_2048::Tile) {
    let computed_style = window().unwrap().get_computed_style(html_tile).unwrap().unwrap();
    let current_top_offset = &computed_style.get_property_value("top").unwrap();
    let current_left_offset = &computed_style.get_property_value("left").unwrap();

    let (new_top_offset, new_left_offset) = convert_to_pixels(game_tile.row, game_tile.col);

    let new_top_offset = format!("{}px", new_top_offset);
    let new_left_offset = format!("{}px", new_left_offset);

    html_tile.style().set_property("--current_left", &current_left_offset).unwrap();
    html_tile.style().set_property("--current_top", &current_top_offset).unwrap();

    html_tile.style().set_property("--new_left", &new_left_offset).unwrap();
    html_tile.style().set_property("--new_top", &new_top_offset).unwrap();

    // let parent_node = html_tile.parent_node().unwrap();
    // parent_node.remove_child(&html_tile).unwrap();
    // parent_node.append_child(&html_tile).unwrap();

    // html_tile.style().set_property("animation", "sliding 0.10s ease-in-out forwards").unwrap();

    // sleep(Duration::from_millis(100)).await;
    
    html_tile.style().set_property("top", &new_top_offset).unwrap();
    html_tile.style().set_property("left", &new_left_offset).unwrap();
}

async fn process_keydown_messages(game_state: Rc<RefCell<Game>>, mut keydown_rx: Receiver<String>) {
    while let Some(key_code) = keydown_rx.recv().await {
        match game_state.borrow_mut().receive_input(&key_code) {
            InputResult::Ok(new_tile_id, tiles) => {
                log!("Move successful!");
                let document = gloo::utils::document();
                
                match document.query_selector_all("[class='tile cell']") {
                    Ok(node_list) => {
                        for i in 0..node_list.length() {
                            let node = node_list.get(i).unwrap();
                            let tile = node.dyn_ref::<HtmlElement>().unwrap();
                            let tile_id = tile.get_attribute("id").unwrap().parse::<usize>().unwrap();


                            if let Some(updated_tile) = get_tile_by_id(&tiles, tile_id) {
                                slide_tile(tile, updated_tile).await;

                                // If a tile is merged, its corresponding tile was removed from the backend. However, the backend provides that
                                // Tile's ID, row, and col so the frontend knows how to slide the tile into position before merging and deleting it.
                                // If the `merged` field is the `None` variant, that means that Tile was not merged.
                                if let Some((removed_id, removed_row, removed_col)) = updated_tile.merged {
                                    // Save the Tile's ID for later removal.
                                    tile.style().set_property("--merged_value", &updated_tile.value.to_string()).unwrap();
                                    tile.style().set_property("--merged_id", &removed_id.to_string()).unwrap();
                                    tile.style().set_property("--bg_color", &updated_tile.background_color).unwrap();
                                    tile.style().set_property("--txt_color", &updated_tile.text_color).unwrap();

                                    // removed_tile.style().set_property("--current_left", &current_left_offset).unwrap();
                                    // removed_tile.style().set_property("--current_top", &current_top_offset).unwrap();

                                    // removed_tile.style().set_property("--new_left", &new_left_offset).unwrap();
                                    // removed_tile.style().set_property("--new_top", &new_top_offset).unwrap();

                                    // let parent_node = removed_tile.parent_node().unwrap();
                                    // parent_node.remove_child(&removed_tile).unwrap();
                                    // parent_node.append_child(&removed_tile).unwrap();

                                    // removed_tile.style().set_property("animation", "sliding 0.10s ease-in-out forwards").unwrap();

                                    // removed_tile.style().set_property("top", &new_top_offset).unwrap();
                                    // removed_tile.style().set_property("left", &new_left_offset).unwrap();
                                }
                            }
                        }

                        // let board = document.query_selector(".board-container").unwrap().unwrap();
                        // let board = board.dyn_ref::<HtmlElement>().unwrap();
                        // let board_parent = board.parent_node().unwrap();
                        // board_parent.remove_child(&board).unwrap();
                        // board_parent.append_child(&board).unwrap();
                        // sleep(Duration::from_millis(100)).await;
                    },
                    Err(_) => log!("NodeList could not be found."),
                }
            },
            InputResult::Err(InvalidMove) => {
                log!("Invalid move.");
            },
        }
    }
}

#[function_component(Content)]
fn content() -> Html {
    let game_state = Rc::new(RefCell::new(Game::new()));
    let game_state_for_move_listener = Rc::clone(&game_state);
    let game_state_for_sliding_listener = Rc::clone(&game_state);
    let keypressed_for_keydown = Rc::new(RefCell::new(false));
    let keypressed_for_keyup = keypressed_for_keydown.clone();
    let score_ref = use_node_ref();
    let score_merge_clone = score_ref.clone();
    
    // Prevents use of arrow keys for scrolling the page
    preventDefaultScrolling();

    let (keydown_tx, keydown_rx) = mpsc::channel(100);
    
    // Attach a keydown event listener to the document.
    use_effect(move || {
        spawn_local(process_keydown_messages(game_state_for_move_listener, keydown_rx));

        let document = gloo::utils::document();
        let listener = EventListener::new(&document, "keydown", move |event| {
            
            let key_code = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw().code();
            keydown_tx.blocking_send(key_code).expect("Sending key_code failed.");
        });

        // Called when the component is unmounted.  The closure has to hold on to `listener`, because if it gets
        // dropped, `gloo` detaches it from the DOM. So it's important to do _something_, even if it's just dropping it.
        || drop(listener)
    });
    
    // Add event listener for sliding animation end to then begin the expanding animation for merged tiles.
    use_effect(move || {
        let body = gloo::utils::body();
        let merge_expand = Closure::wrap(Box::new(move |event: AnimationEvent| {
        }) as Box<dyn FnMut(AnimationEvent)>);

        body.add_event_listener_with_callback("animationend", merge_expand.as_ref().unchecked_ref()).unwrap();

        || {
            // Must remove the callback or else memory leak will occur each time New Game is clicked.
            let body = gloo::utils::body();
            body.remove_event_listener_with_callback("animationend", merge_expand.as_ref().unchecked_ref()).unwrap();
            drop(merge_expand)
        }
    });

    // use_state() hook is used to trigger a re-render whenever the `New Game` button is clicked.
    // The value is used as a key to each Tile component in order to its `expand-init` animation.
    let new_game = use_state(|| 0);
    let onclick = {
        // Elements manipulated manually using web_sys do not get removed when this component is re-rendered.
        // Must remove them manually here.
        let document = gloo::utils::document();
        match document.query_selector_all("[class='tile cell']") {
            Ok(node_list) => {
                for i in 0..node_list.length() {
                    let element = node_list.get(i).unwrap();
                    let element = element.dyn_ref::<HtmlElement>().unwrap();
                    element.remove();
                }
            },
            Err(_) => log!("Tiles could not be found."),
        }

        let new_game = new_game.clone();
        Callback::from(move |_| new_game.set(*new_game + 1))
    };

    let new_game_render = *new_game.clone();

    html! {
        <div class="content" key={new_game_render}>
            <Metadata score={0} node_ref={score_ref} {onclick}/>
            <div class="board-container">
                <GameBoard/>
                { 
                    for game_state.borrow().get_tiles().iter().map(|tile| {
                        let value = tile.value;
                        let background_color = tile.background_color.clone();
                        let text_color = tile.text_color.clone();
                        let id = tile.id;
                        let (top_offset, left_offset) = 
                            convert_to_pixels(tile.row, tile.col);

                        html! {
                            <Tile 
                                value={value}
                                background_color={background_color}
                                text_color={text_color}
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

#[derive(Properties, PartialEq)]
struct MetadataProps {
    onclick: Callback<MouseEvent>,
    score: u32,
    node_ref: NodeRef,
}

#[function_component(Metadata)]
fn metadata(props: &MetadataProps) -> Html {
    let style_args = format!("--button_border: {}; 
                              --button_background: {};
                              --button_hover: {};
                              --button_text: {};",
                              COLORS.text_dark,
                              COLORS.button,
                              COLORS.button_hover,
                              COLORS.text_dark,
                              );

    let onclick = props.onclick.clone();

    html! {
        <div class="metadata" style={style_args}>
            <Score node_ref={props.node_ref.clone()} score={props.score}/>
            <button class="new-game" {onclick}>{ "New Game" }</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ScoreProps {
    score: u32,
    node_ref: NodeRef,
}

#[function_component(Score)]
fn score(props: &ScoreProps) -> Html {
    html! {
        <div ref={props.node_ref.clone()} class="score">{props.score}</div>
    }
}

#[function_component(Header)]
fn header() -> Html {
    let header_style = format!("--header_text: {}", COLORS.text_light);
    let cursor_style = format!("--blinking_cursor: {}", COLORS.text_light);

    html! {
        <div class="header" style={header_style}>
            <br/>
            <h1 class="typed" style={cursor_style}>{ "Welcome to 2048!" }</h1>
        </div>
    }
}

#[function_component(Footer)]
fn footer() -> Html {
    let style_args = format!("--footer_text: {}", COLORS.text_light);

    html! {
        <div class="footer" style={style_args}>
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
    set_background_colors();

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

// Helper functions

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
    let top = pixel_to_u16(top);
    let left = pixel_to_u16(left);
    
    (pixel_to_index(top), pixel_to_index(left))
}

/// Determines font-size based on number of digits to prevent overflow.
fn compute_font_size(value: &String) -> String {
    let mut font_size = "";
    let len = value.len();

    if len > 5 {
        font_size = "1.85em";
    } else if len > 4 {
        font_size = "2.25em";
    } else if len > 3 {
        font_size = "2.70em";
    } else {
        font_size = "3.50em";
    }

    font_size.to_string()
}

/// Accepts a String pixel value e.g. "252px" and returns it parsed as u16.
fn pixel_to_u16(pixel_value: &str) -> u16 {
    pixel_value.substring(0, pixel_value.chars().count() - 2).parse::<u16>().unwrap()
}

/// Accepts pixel values as u16 and returns the corresponding grid index.
fn pixel_to_index(pixel_value: u16) -> usize {
    match pixel_value {
        4 => 0,
        128 => 1,
        252 => 2,
        376 => 3,
        _ => unreachable!(),
    }
}

/// Accepts a Vec of Tile references and an ID and returns an Option Tile with the corresponding ID if it
/// is found, otherwise returns None.
fn get_tile_by_id<'a>(tiles: &Vec<&'a rust_2048::Tile>, id: usize) -> Option<&'a rust_2048::Tile> {
    for tile in tiles {
        if tile.id == id {
            return Some(tile)
        }
    }

    None
}

/// Sets the background-image to a linear-gradient determined by the `Colors` struct defined in lib.rs.
fn set_background_colors() {
    let body = gloo::utils::body();

    let linear_gradient = format!("linear-gradient({}, {})", COLORS.background_dark, COLORS.background_light);
    body.style().set_property("background-image", &linear_gradient).unwrap();
}

fn convert_id_unicode(id: &String) -> String {
    let mut converted_id = String::from("#\\3");

    for c in id.chars() {
        converted_id.push_str(&(c.to_string() + " "));
    }

    converted_id
}

