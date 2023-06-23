use gloo::utils::document;
use yew::prelude::*;
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
    let style_args = format!("--top: {}px; --left: {}px; --background_color: {}; --text_color: {}", 
                           props.top_offset,
                           props.left_offset,
                           props.background_color,
                           props.text_color);

    let tile_id = props.id.to_string();

    html! {
        <div id={tile_id} class="tile cell" style={style_args}>{props.value}</div>
    }
}

#[function_component(Content)]
fn content() -> Html {
    let game_state = Rc::new(RefCell::new(Game::new()));
    let game_state_for_move_listener = Rc::clone(&game_state);
    let game_state_for_sliding_listener = Rc::clone(&game_state);
    let keypressed_for_keydown = Rc::new(RefCell::new(false));
    let keypressed_for_keyup = keypressed_for_keydown.clone();
    
    use_effect(move || {
        // Attach a keydown event listener to the document.
        let document = gloo::utils::document();
        
        let listener = EventListener::new(&document, "keydown", move |event| {

            {
                let mut key_event_fired = keypressed_for_keydown.borrow_mut();

                if !*key_event_fired {
                    *key_event_fired = true;
                } else {
                    return
                }
            }

            let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();

            // Prevents use of arrow keys for scrolling the page
            preventDefaultScrolling();

            let mut game_state_borrow_mut = game_state_for_move_listener.borrow_mut();

            match game_state_borrow_mut.receive_input(&event.code()) {
                InputResult::Ok(new_tile_id) => {
                    log!("Move successful");
                    let tiles = game_state_borrow_mut.get_tiles();

                    let document = gloo::utils::document();

                    match document.query_selector_all("[class='tile cell']") {
                        Ok(node_list) => {
                            for i in 0..node_list.length() {
                                let node = node_list.get(i).unwrap();
                                let tile = node.dyn_ref::<HtmlElement>().unwrap();
                                let tile_id = tile.get_attribute("id").unwrap().parse::<usize>().unwrap();
                                let computed_style = window().unwrap().get_computed_style(&tile).unwrap().unwrap();

                                let current_top_offset = &computed_style.get_property_value("top").unwrap();
                                let current_left_offset = &computed_style.get_property_value("left").unwrap();

                                match get_tile_by_id(&tiles, tile_id) {
                                    Some(updated_tile) => {
                                        let (new_top_offset, new_left_offset) = convert_to_pixels(updated_tile.row, updated_tile.col);

                                        let new_top_offset = format!("{}px", new_top_offset);
                                        let new_left_offset = format!("{}px", new_left_offset);

                                        tile.style().set_property("--current_left", &current_left_offset).unwrap();
                                        tile.style().set_property("--current_top", &current_top_offset).unwrap();

                                        tile.style().set_property("--new_left", &new_left_offset).unwrap();
                                        tile.style().set_property("--new_top", &new_top_offset).unwrap();

                                        if updated_tile.merged {
                                            // Will be used by callback that handles merge_expand animations
                                            tile.style().set_property("--merged", &updated_tile.value.to_string()).unwrap();
                                            log!("Merged:", tile.style().get_property_value("--merged").unwrap());
                                            tile.style().set_property("--background_color", &updated_tile.background_color).unwrap();
                                            tile.style().set_property("--text_color", &updated_tile.text_color).unwrap();
                                        }

                                        let parent_node = tile.parent_node().unwrap();
                                        parent_node.remove_child(&tile).unwrap();
                                        parent_node.append_child(&tile).unwrap();

                                        tile.style().set_property("animation", "sliding 0.90s ease-in-out forwards").unwrap();

                                        tile.style().set_property("top", &new_top_offset).unwrap();
                                        tile.style().set_property("left", &new_left_offset).unwrap();

                                    },
                                    None => {
                                        // Tile with specified id was merged and should be removed.
                                        let parent_node = tile.parent_node().unwrap();
                                        parent_node.remove_child(&tile).unwrap();
                                    }
                                }
                            }
                        }
                        Err(_) => log!("NodeList could not be found."),
                    }

                    // Render the new tile
                    let new_tile = get_tile_by_id(&tiles, new_tile_id).expect("New tile ID not found.");
                    let (top_offset, left_offset) = convert_to_pixels(new_tile.row, new_tile.col);
                    
                    let style_args = format!("--top: {}px; --left: {}px; --background_color: {}", 
                       top_offset,
                       left_offset,
                       &new_tile.background_color,
                    );

                    let new_tile_node = document.create_element("div").expect("Failed to create new tile node.");
                    let new_tile_node = new_tile_node.dyn_ref::<HtmlElement>().unwrap();

                    new_tile_node.set_inner_html(&new_tile.value.to_string());
                    new_tile_node.set_class_name("tile cell");
                    new_tile_node.set_attribute("style", &style_args).unwrap();
                    new_tile_node.set_id(&new_tile_id.to_string());
                    new_tile_node.style().set_property("--background_color", &new_tile.background_color).unwrap();
                    new_tile_node.style().set_property("--text_color", &new_tile.text_color).unwrap();

                    let board_container = document.query_selector(".board-container").unwrap().unwrap();
                    board_container.append_child(&new_tile_node).unwrap();
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


    use_effect(move || {
        // Attach a keydown event listener to the document.
        let document = gloo::utils::document();
        
        let listener = EventListener::new(&document, "keyup", move |_event| {
            let mut key_event_fired = keypressed_for_keyup.borrow_mut();

            *key_event_fired = false;
        });

        || drop(listener)
    });

    // Add event listener for sliding animation end to then begin the expanding animation for merged tiles.
    let body = gloo::utils::body();
    let merge_expand = Closure::wrap(Box::new(move |event: AnimationEvent| {
        if event.animation_name() == "sliding" {
            let event_target = event.target().expect("No event target found.");
            let tile = event_target.dyn_ref::<HtmlElement>().unwrap();

            let computed_style = window().unwrap().get_computed_style(&tile).unwrap().unwrap();

            match tile.style().get_property_value("--merged") {
                Ok(merged) => {
                    if merged != "-1" && merged != "" {
                        
                        tile.set_inner_html(&merged);
                        tile.style().set_property("animation", "expand-merge 0.15s ease-in-out").unwrap();

                        let parent_node = tile.parent_node().unwrap();
                        parent_node.remove_child(&tile).unwrap();
                        parent_node.append_child(&tile).unwrap();

                        tile.style().set_property("--merged", "-1").unwrap();
                    }
                },
                Err(_) => (),
            }

            // let tile_id = tile.get_attribute("id").unwrap().parse::<usize>().expect("Failed to parse id as usize.");
            // let tiles = game_state_for_sliding_listener.borrow();
            // let tiles = tiles.get_tiles();

            // match get_tile_by_id(&tiles, tile_id) {
            //     Some(tile_ref) => {
            //         if tile_ref.merged {
            //             tile.style().set_property("animation", "expand-merge 0.20s ease-in-out").unwrap();

            //             let parent_node = tile.parent_node().unwrap();
            //             parent_node.remove_child(&tile).unwrap();
            //             parent_node.append_child(&tile).unwrap();
            //         }
                    
            //     },
            //     None => (),
            // }
        }
    }) as Box<dyn FnMut(AnimationEvent)>);

    body.add_event_listener_with_callback("animationend", merge_expand.as_ref().unchecked_ref()).unwrap();
    merge_expand.forget();

    // use_state() hook is used to trigger a re-render whenever the `New Game` button is clicked.
    // The value is used as a key to each Tile component in order to its `expand-init` animation.
    let new_game = use_state(|| 0);
    let onclick = {
        // Elements added via `append_child` do not get removed when this component is re-rendered.
        // Must remove them manually here.
        let document = gloo::utils::document();
        match document.query_selector_all("[class='tile cell']") {
            Ok(node_list) => {
                for i in 0..node_list.length() {
                    let element = node_list.get(i).unwrap();
                    let element = element.dyn_ref::<HtmlElement>().unwrap();
                    element.remove();
                }
            }

            Err(_) => log!("Tiles could not be found."),
        }

        let new_game = new_game.clone();
        Callback::from(move |_| new_game.set(*new_game + 1))
    };

    html! {
        <div class="content">
            <Metadata score={game_state.borrow().score} {onclick}/>
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
                        let animation_key = *new_game.clone();

                        html! {
                            <Tile 
                                key={animation_key}
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
    score: u64,
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
            <Score score={props.score}/>
            <button class="new-game" {onclick}>{ "New Game" }</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ScoreProps {
    score: u64,
}

#[function_component(Score)]
fn score(props: &ScoreProps) -> Html {
    html! {
        <div class="score">{props.score}</div>
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

