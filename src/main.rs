use tokio::sync::mpsc::UnboundedReceiver;
use yew::prelude::*;
use rust_2048::*;
use gloo_console::log;
use gloo::events::EventListener;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, window, CssAnimation, AnimationPlayState};
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use tokio::sync::mpsc;
use yew::platform::time::sleep;
use core::time::Duration;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

const BORDER_SPACING: u16 = 4;
const TILE_DIMENSION: u16 = 120;
const COLORS: Colors = Colors::new();

// Durations in milliseconds.
const DEFAULT_SLIDE_DURATION: u64 = 125;
const DEFAULT_EXPAND_DURATION: u64 = 125;
const DEFAULT_SLEEP_DURATION: u64 = 5;

// Globally mutable variables. If the number of player moves in the queue is greater than 1, all
// animation durations will be set to 0. Otherwise the original values will be restored.
lazy_static! {
    static ref CURRENT_SLIDE_DURATION: Mutex<u64> = Mutex::new(DEFAULT_SLIDE_DURATION);
    static ref CURRENT_EXPAND_DURATION: Mutex<u64> = Mutex::new(DEFAULT_EXPAND_DURATION);
    static ref CURRENT_SLEEP_DURATION: Mutex<u64> = Mutex::new(DEFAULT_SLEEP_DURATION);
}


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
    let expand_init_animation = format!("expand-init {}ms ease-in-out;", CURRENT_EXPAND_DURATION.lock().unwrap());
    let style_args = format!("top: {}px; left: {}px; background-color: {}; color: {}; font-size: {}; animation: {};", 
                           props.top_offset,
                           props.left_offset,
                           props.background_color,
                           props.text_color,
                           compute_font_size(&props.value.to_string()),
                           expand_init_animation,
                           );

    let tile_id = props.id.to_string();

    html! {
        <div id={tile_id} class="tile cell" style={style_args}>{props.value}</div>
    }
}

fn set_animation_durations(input_counter: Arc<AtomicU16>, threshold: u16, duration: u64) {
    if input_counter.load(Ordering::SeqCst) >  threshold {
        *CURRENT_SLIDE_DURATION.lock().unwrap() = 0;
        *CURRENT_EXPAND_DURATION.lock().unwrap() = 0;
        *CURRENT_SLEEP_DURATION.lock().unwrap() = 20;
    } else {
        *CURRENT_SLIDE_DURATION.lock().unwrap() = duration;
        *CURRENT_EXPAND_DURATION.lock().unwrap() = duration;
        *CURRENT_SLEEP_DURATION.lock().unwrap() = duration;
    }
}

fn remove_tile(id: usize) {
    let document = gloo::utils::document();
    let id = convert_id_unicode(&id.to_string());

    let removed_tile_node = document.query_selector(&id).unwrap().unwrap();
    let parent_node = removed_tile_node.parent_node().unwrap();
    parent_node.remove_child(&removed_tile_node).unwrap();
}

fn update_score(new_score: u32) {
    let document = gloo::utils::document();
    let score_node = document.query_selector(".score").unwrap().unwrap();
    score_node.set_inner_html(&new_score.to_string());
}

/// Waits for the given animation to complete.
async fn await_animations(animation_name: String) {
    let document = gloo::utils::document();
    let animations = document.get_animations();
    let mut id = 0;

    for animation in animations {
        let animation = CssAnimation::from(animation);
        if animation.animation_name() == animation_name {
            let c_id = format!("_{}", id);
            animation.set_id(&c_id);
            let mut play_state = animation.play_state();

            while !matches!(play_state, AnimationPlayState::Finished) {
                // log!(&animation_name, "sleeping . . .");
                sleep(Duration::from_millis(*CURRENT_SLEEP_DURATION.lock().unwrap())).await;
                
                let animations = document.get_animations();

                let mut animation: Option<CssAnimation> = None;
                for a in animations {
                    let a = CssAnimation::from(a);
                    if a.animation_name() == animation_name {
                        if a.id() == c_id {
                            animation = Some(a);
                            break;
                        }
                    }
                };

                play_state = match animation {
                    Some(a) => a.play_state(),
                    None => break
                }
            }

            id += 1;
        }
    }
}

fn add_tile(game_tile: &rust_2048::Tile) {
    let (top_offset, left_offset) = convert_to_pixels(game_tile.row, game_tile.col);

    let font_size = compute_font_size(&game_tile.value.to_string());
    let expand_init_animation = format!("expand-init {}ms ease-out;", CURRENT_EXPAND_DURATION.lock().unwrap());

    let style_args = format!("top: {}px; left: {}px; background-color: {}; color: {}; font-size: {}; animation: {};",
       top_offset,
       left_offset,
       &game_tile.background_color,
       &game_tile.text_color,
       font_size,
       expand_init_animation,
    );

    let document = gloo::utils::document();

    let html_tile = document.create_element("div").expect("Failed to create new tile node.");
    let html_tile = html_tile.dyn_ref::<HtmlElement>().unwrap();

    html_tile.set_inner_html(&game_tile.value.to_string());
    html_tile.set_class_name("tile cell");
    html_tile.set_attribute("style", &style_args).unwrap();
    html_tile.set_id(&game_tile.id.to_string());

    let board_container = document.query_selector(".board-container").unwrap().unwrap();
    board_container.append_child(&html_tile).unwrap();
}

/// Removes and re-appends html_tile to ensure animations trigger each time rather than only once.
fn re_append(html_tile: &HtmlElement) {
    let parent_node = html_tile.parent_node().unwrap();
    parent_node.remove_child(&html_tile).unwrap();
    parent_node.append_child(&html_tile).unwrap();
}

fn merge_tiles() {
    let document = gloo::utils::document();

    match document.query_selector_all("[class='tile cell']") {
        Ok(node_list) => {
            for i in 0..node_list.length() {
                let node = node_list.get(i).unwrap();
                let html_tile = node.dyn_ref::<HtmlElement>().unwrap();

                if let Ok(merged_value) = html_tile.style().get_property_value("--merged_value") {
                    if !merged_value.is_empty() {
                        // Adjust font size and number value.
                        html_tile.style().set_property("font-size", &compute_font_size(&merged_value)).unwrap();
                        html_tile.set_inner_html(&merged_value);

                        // Obtain and set appropriate Tile colors.
                        let new_background_color = html_tile.style().get_property_value("--background_color").unwrap();
                        let new_text_color = html_tile.style().get_property_value("--text_color").unwrap();

                        html_tile.style().set_property("background-color", &new_background_color).unwrap();
                        html_tile.style().set_property("color", &new_text_color).unwrap();

                        // Reset all of these properties.
                        html_tile.style().set_property("--merged_value", "").expect("Failed to reset --merged_value to empty.");
                        html_tile.style().set_property("--background_color", "").unwrap();
                        html_tile.style().set_property("--text_color", "").unwrap();

                        // Initiate merging expand animation.
                        let expanding_animation = format!("expand-merge {}ms ease-in-out", CURRENT_EXPAND_DURATION.lock().unwrap());
                        html_tile.style().set_property("animation", &expanding_animation).unwrap();
                        re_append(html_tile);
                    }
                }

            }
        },
        Err(_) => log!("NodeList could not be found."),
    } 
}

fn slide_tile(html_tile: &HtmlElement, game_tile: &rust_2048::Tile) {
    // Obtain current top and left offsets.
    let computed_style = window().unwrap().get_computed_style(&html_tile).unwrap().unwrap();
    let current_top_offset = computed_style.get_property_value("top").unwrap();
    let current_left_offset = computed_style.get_property_value("left").unwrap();

    // Compute new top and left offsets.
    let (new_top_offset, new_left_offset) = convert_to_pixels(game_tile.row, game_tile.col);

    let new_top_offset = format!("{}px", new_top_offset);
    let new_left_offset = format!("{}px", new_left_offset);

    html_tile.style().set_property("--current_top", &current_top_offset).unwrap();
    html_tile.style().set_property("--current_left", &current_left_offset).unwrap();
    
    html_tile.style().set_property("--new_top", &new_top_offset).unwrap();
    html_tile.style().set_property("--new_left", &new_left_offset).unwrap();

    let sliding_animation = format!("sliding {}ms ease-in-out forwards", CURRENT_SLIDE_DURATION.lock().unwrap());

    if let Some(_) = &game_tile.merged {
        // Tiles with the --merged_value property set will be marked for the merging animation
        // later, along with having their value and colors updated as well.
        html_tile.style().set_property("--merged_value", &game_tile.value.to_string()).unwrap();
        html_tile.style().set_property("--background_color", &game_tile.background_color).unwrap();
        html_tile.style().set_property("--text_color", &game_tile.text_color).unwrap();
    }

    html_tile.style().set_property("animation", &sliding_animation).unwrap();
    re_append(html_tile);

    html_tile.style().set_property("top", &new_top_offset).unwrap();
    html_tile.style().set_property("left", &new_left_offset).unwrap();
}

/// Calls slide_tile() in a loop to move each tile into position. Returns a Vec containing the IDs
/// of every Tile that needs to be deleted from the frontend.
fn slide_tiles(node_list: web_sys::NodeList, tiles: &Vec<&rust_2048::Tile>) -> Vec<usize> {
    let document = gloo::utils::document();
    let mut removed_tile_ids = Vec::new();

    for i in 0..node_list.length() {
        let node = node_list.get(i).unwrap();
        let html_tile = node.dyn_ref::<HtmlElement>().unwrap();
        let tile_id = html_tile.get_attribute("id").unwrap().parse::<usize>().unwrap();

        if let Some(updated_tile) = get_tile_by_id(&tiles, tile_id) {
            // If a tile is merged, its corresponding tile was removed from the backend.
            // However, the backend provides a clone of the removed Tile in the `updated_tile.merged` field.
            // This clone can be used to obtain the Tile's final position so the frontend can slide it 
            // into that position before deleting it, thereby ensuring animation integrity.
            // If the `merged` field is the `None` variant, that means that Tile was not merged.
            if let Some(removed_tile) = &updated_tile.merged {
                let removed_html_node = document.query_selector(&convert_id_unicode(&removed_tile.id.to_string())).unwrap().unwrap();
                let removed_html_tile = removed_html_node.dyn_ref::<HtmlElement>().unwrap();

                slide_tile(removed_html_tile, removed_tile);

                // Mark this tile for removal from the frontend.
                removed_tile_ids.push(removed_tile.id);
            }

            slide_tile(html_tile, updated_tile);
        }
    }

    removed_tile_ids
}

async fn process_keydown_messages(game_state: Rc<RefCell<Game>>, mut keydown_rx: UnboundedReceiver<String>, input_counter: Arc<AtomicU16>) {
    let game_state_mut = game_state.clone();
    let mut game_state_mut = game_state_mut.borrow_mut();

    let mut moves_pending = false;

    while let Some(key_code) = keydown_rx.recv().await {
        match game_state_mut.receive_input(&key_code) {
            InputResult::Ok(new_tile_id, tiles) => {
                let document = gloo::utils::document();
                
                match document.query_selector_all("[class='tile cell']") {
                    Ok(node_list) => {
                        log!("Sliding");
                        let mut slide_duration = DEFAULT_SLIDE_DURATION;

                        if moves_pending {
                            slide_duration = DEFAULT_SLIDE_DURATION / (input_counter.load(Ordering::SeqCst) as u64);
                        }

                        set_animation_durations(input_counter.clone(), 1, slide_duration);
                        let removed_tile_ids = slide_tiles(node_list, &tiles);
                        await_animations("sliding".to_string()).await;
                        
                        // Removing marked tiles, adding new tile, and updating score 
                        // should occur simultaneously with merge-expand animation.
                        for id in removed_tile_ids {
                            remove_tile(id);
                        }

                        log!("Decrementing input_counter");
                        input_counter.fetch_sub(1, Ordering::SeqCst);

                        // let expand_duration = DEFAULT_EXPAND_DURATION;
                        set_animation_durations(input_counter.clone(), 0, DEFAULT_EXPAND_DURATION);

                        log!("Expanding");
                        // Render the new Tile.
                        add_tile(get_tile_by_id(&tiles, new_tile_id).expect("Failed to find new Tile."));

                        merge_tiles();
                        await_animations("expand-merge".to_string()).await;

                        if input_counter.load(Ordering::SeqCst) > 0 {
                            moves_pending = true;
                        } else {
                            moves_pending = false;
                        }
                        
                        // Update the score.
                        update_score(game_state_mut.score);
                    },
                    Err(_) => log!("NodeList could not be found."),
                }
            },
            InputResult::Err(InvalidMove) => {
                log!("Decrementing input_counter");
                input_counter.fetch_sub(1, Ordering::SeqCst);
            },
        }
    }
}

#[function_component(Content)]
fn content() -> Html {
    let game_state = Rc::new(RefCell::new(Game::new()));
    let game_state_for_move_processor = Rc::clone(&game_state);
    let input_counter = Arc::new(AtomicU16::new(0));
 
    // Prevents use of arrow keys for scrolling the page
    preventDefaultScrolling();

    // Attach a keydown event listener to the document.
    use_effect(move || {
        let (keydown_tx, keydown_rx) = mpsc::unbounded_channel();

        spawn_local(process_keydown_messages(game_state_for_move_processor, keydown_rx, input_counter.clone()));

        let document = gloo::utils::document();
        let listener = EventListener::new(&document, "keydown", move |event| {
            let key_code = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw().code();
            log!("Incrementing input_counter");
            input_counter.fetch_add(1, Ordering::SeqCst);
            keydown_tx.send(key_code).expect("Sending key_code failed.");
        });

        // Called when the component is unmounted.  The closure has to hold on to `listener`, because if it gets
        // dropped, `gloo` detaches it from the DOM. So it's important to do _something_, even if it's just dropping it.
        || drop(listener)
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
            <Metadata score={0} {onclick}/>
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
    score: u32,
}

#[function_component(Score)]
fn score(props: &ScoreProps) -> Html {
    html! {
        <div class="score">{props.score}</div>
    }
}

#[function_component(WinLayer)]
fn win_layer() -> Html {
    html! {

    }
}

#[function_component(LossLayer)]
fn loss_layer() -> Html {
    let style_args = format!("");

    html! {
        <div class="gameover" style={style_args}/>
    }
}

#[function_component(Header)]
fn header() -> Html {
    let header_style = format!("--header_text: {}", COLORS.text_light);
    let cursor_style = format!("--blinking_cursor: {}", COLORS.text_light);

    html! {
        <div class="header" style={header_style}>
            <br/>
            <div class="typed" style={cursor_style}>{ "Welcome to 2048!" }</div>
        </div>
    }
}

#[function_component(Footer)]
fn footer() -> Html {
    let style_args = format!("--footer_text: {}; --visited_color: {}",
                             COLORS.text_light,
                             COLORS.cell,
                             );

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

/// Determines font-size based on number of digits to prevent overflow.
fn compute_font_size(value: &String) -> String {
    let font_size;
    let len = value.len();

    if len > 5 {
        font_size = "2.15em";
    } else if len > 4 {
        font_size = "2.50em";
    } else if len > 3 {
        font_size = "3.00em";
    } else if len > 2 {
        font_size = "4.00em";
    } else if len > 1 {
        font_size = "4.25em";
    } else {
        font_size = "4.5em";
    }

    font_size.to_string()
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

