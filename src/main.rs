use gloo::utils::document;
use gloo_console::log;
use lazy_static::lazy_static;
use rust_2048::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement, window, CssAnimation, AnimationPlayState};
use yew::platform::time::sleep;
use yew::prelude::*;
use core::time::Duration;
mod counted_channel;

const BORDER_SPACING: u16 = 4;
const TILE_DIMENSION: u16 = 120;
const COLORS: Colors = Colors::new();

// Durations in milliseconds.
const DEFAULT_SLIDE_DURATION: u64 = 120;
const DEFAULT_EXPAND_DURATION: u64 = 110;
const DEFAULT_SLEEP_DURATION: u64 = 1;

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

fn set_animation_durations(input_counter: Arc<AtomicU16>) {
    let enqueued_keystrokes = input_counter.load(Ordering::SeqCst);

    let mut slide_duration = DEFAULT_SLIDE_DURATION;
    let mut expand_duration = DEFAULT_EXPAND_DURATION;

    if enqueued_keystrokes > 0 {
        slide_duration /= enqueued_keystrokes as u64;
        expand_duration /= enqueued_keystrokes as u64;
    }

    *CURRENT_SLIDE_DURATION.lock().unwrap() = slide_duration;
    *CURRENT_EXPAND_DURATION.lock().unwrap() = expand_duration;
}

fn handle_game_over(game_won: bool, keydown_handler: Arc<Closure<dyn FnMut(yew::KeyboardEvent)>>) {
    // Disable keyboard events when game is over.
    let document = gloo::utils::document();
    document.remove_event_listener_with_callback("keydown", Closure::as_ref(&keydown_handler).unchecked_ref()).unwrap();

    let mut game_over_type = ".gameover".to_string();

    if game_won {
        game_over_type += ".won";
    } else {
        game_over_type += ".lost";
    }

    let game_over_layer = document.query_selector(&game_over_type).unwrap().unwrap();
    let game_over_layer = game_over_layer.dyn_ref::<HtmlElement>().unwrap();

    game_over_layer.remove_attribute("hidden").expect("Failed to remove hidden attribute.");
    game_over_layer.style().set_property("z-index", "4").unwrap();

    // Enable buttons on gameover layer.
    match document.query_selector_all(&format!("{}>div.buttons>button", game_over_type)) {
        Ok(node_list) => {
            for i in 0..node_list.length() {
                let node = node_list.get(i).unwrap();
                let html_node = node.dyn_ref::<HtmlElement>().unwrap();
                html_node.remove_attribute("disabled").unwrap();
            }
        },
        Err(_) => {
            log!("Error obtaining button.metadata");
        }
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

fn remove_tiles(removed_tile_ids: Vec<usize>) {
    for id in removed_tile_ids {
        remove_tile(id);
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
                        update_tile(&html_tile, &merged_value);
                        expand_tile(&html_tile);
                    }
                }

            }
        },
        Err(_) => log!("NodeList could not be found."),
    } 
}

fn update_tile(html_tile: &HtmlElement, merged_value: &String) {
    // Adjust font size and number value.
    html_tile.style().set_property("font-size", &compute_font_size(&merged_value)).unwrap();
    html_tile.set_inner_html(&merged_value);

    // Obtain and set appropriate Tile colors.
    let new_background_color = html_tile.style().get_property_value("--background_color").unwrap();
    let new_text_color = html_tile.style().get_property_value("--text_color").unwrap();

    html_tile.style().set_property("background-color", &new_background_color).unwrap();
    html_tile.style().set_property("color", &new_text_color).unwrap();

    // Reset all of these properties.
    html_tile.style().remove_property("--merged_value").unwrap();
    html_tile.style().remove_property("--background_color").unwrap();
    html_tile.style().remove_property("--text_color").unwrap();
}

fn expand_tile(html_tile: &HtmlElement) {
    let expanding_animation = format!("expand-merge {}ms ease-out", CURRENT_EXPAND_DURATION.lock().unwrap());
    html_tile.style().set_property("animation", &expanding_animation).unwrap();
    re_append(html_tile);
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

/// Calls slide_tile() in a loop to move each tile into position. Returns the number of merged tiles.
fn slide_tiles(node_list: web_sys::NodeList, tiles: &Vec<&rust_2048::Tile>) -> (Vec<usize>, u16) {
    let document = gloo::utils::document();

    let mut removed_ids = Vec::new();
    let mut num_merged = 0;

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
                removed_ids.push(removed_tile.id);
                num_merged += 1;

                let removed_html_node = document.query_selector(&convert_id_unicode(&removed_tile.id.to_string())).unwrap().unwrap();
                let removed_html_tile = removed_html_node.dyn_ref::<HtmlElement>().unwrap();

                slide_tile(removed_html_tile, removed_tile);

                // Mark this tile for removal from the frontend.
                html_tile.style().set_property("--remove_id", &removed_tile.id.to_string()).unwrap();
            }

            slide_tile(html_tile, updated_tile);
        }
    }

    (removed_ids, num_merged)
}

async fn process_keydown_messages(game_state: Rc<RefCell<Game>>, mut keydown_rx: UnboundedReceiver<String>, mut animationend_rx: counted_channel::CountedReceiver, input_counter: Arc<AtomicU16>, keydown_handler: Arc<Closure<dyn FnMut(yew::KeyboardEvent)>>) {
    let game_state_mut = game_state.clone();
    let mut game_state_mut = game_state_mut.borrow_mut();

    while let Some(key_code) = keydown_rx.recv().await {
        match game_state_mut.receive_input(&key_code) {
            InputResult::Ok(new_tile_id, tiles, game_won) => {
                let document = gloo::utils::document();

                match document.query_selector_all("[class='tile cell']") {
                    Ok(node_list) => {
                        let mut now = instant::Instant::now();

                        let num_elements_slide = node_list.length() as u16;
                        set_animation_durations(input_counter.clone());
                        let (removed_ids, num_merged) = slide_tiles(node_list, &tiles);
                        animationend_rx.recv_qty(num_elements_slide).await;
                        remove_tiles(removed_ids);

                        log!(format!("{:?}", instant::Instant::now() - now));

                        decrement_counter(input_counter.clone());
                        set_animation_durations(input_counter.clone());

                        now = instant::Instant::now();
                        animationend_rx.recv_qty(num_merged).await;
                        log!(format!("{:?}", instant::Instant::now() - now));

                        add_tile(get_tile_by_id(&tiles, new_tile_id).expect("Failed to find new Tile."));
                        update_score(game_state_mut.score);
                    },
                    Err(_) => log!("NodeList could not be found."),
                }

                if game_state_mut.game_over() || game_won {
                // if true || game_won {
                    document.remove_event_listener_with_callback("keydown", Closure::as_ref(&keydown_handler).unchecked_ref()).unwrap();

                    log!("Inputs remaining:", input_counter.load(Ordering::SeqCst));

                    while input_counter.load(Ordering::SeqCst) > 0 && matches!(keydown_rx.recv().await, Some(_)) {
                        decrement_counter(input_counter.clone());
                    }

                    handle_game_over(game_won, keydown_handler.clone());
                }
            },
            InputResult::Err(InvalidMove) => {
                decrement_counter(input_counter.clone());
            },
        }
    }
}

fn produce_keydown_handler(keydown_tx: UnboundedSender<String>, input_counter: Arc<AtomicU16>) -> Box<dyn FnMut(KeyboardEvent) -> ()> {
    Box::new(move |event: KeyboardEvent| {
        let key_code = event.code();
        increment_counter(input_counter.clone());
        keydown_tx.send(key_code).expect("Sending key_code failed.");
    })
}

fn keep_playing_callback(keydown_handler: Arc<Closure<dyn FnMut(yew::KeyboardEvent)>>) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        // Re-enable keyboard events.
        let document = gloo::utils::document();
        document.add_event_listener_with_callback("keydown", Closure::as_ref(&keydown_handler).unchecked_ref()).unwrap();

        // Remove gameover layer.
        let gameover_layer = document.query_selector(".gameover.won").unwrap().unwrap();
        gameover_layer.remove();
    })
}

fn new_game_callback(new_game_hook: UseStateHandle<u32>) -> Callback<MouseEvent> {
    Callback::from(move |_| {
        // Elements manipulated manually using web_sys do not get removed when this component is re-rendered.
        // Must remove them manually here.

        let document = gloo::utils::document();
        let bc = document.query_selector(".board-container").unwrap().unwrap();
        let bc = bc.dyn_ref::<HtmlElement>().unwrap();
        bc.remove();
        // match document.query_selector_all("[class='tile cell']") {
        //     Ok(node_list) => {
        //         for i in 0..node_list.length() {
        //             let element = node_list.get(i).unwrap();
        //             let element = element.dyn_ref::<HtmlElement>().unwrap();
        //             element.remove();
        //         }
        //     },
        //     Err(_) => log!("Tiles could not be found."),
        // }

        new_game_hook.set(*new_game_hook + 1);
    })
}

fn animationend_callback(input_counter: Arc<AtomicU16>, animationend_tx: counted_channel::CountedSender) -> Closure<dyn FnMut(web_sys::AnimationEvent)> {
    Closure::wrap(Box::new(move |event: AnimationEvent| {
        if event.animation_name() == "sliding" {
            if event.type_() == "animationcancel" {
                log!("canceled");
            }

            let event_target = event.target().unwrap();
            let html_tile = event_target.dyn_ref::<HtmlElement>().unwrap();

            if let Ok(merged_value) = html_tile.style().get_property_value("--merged_value") {
                if !merged_value.is_empty() {
                    // remove_tile(html_tile.style().get_property_value("--remove_id").unwrap());
                    // html_tile.style().remove_property("--remove_id").unwrap();
                    update_tile(&html_tile, &merged_value);
                    expand_tile(&html_tile);
                }
            }

            animationend_tx.send(String::from(event.animation_name())).unwrap();
        } else if event.animation_name() == "expand-merge" {
            animationend_tx.send(String::from(event.animation_name())).unwrap();
        }

    }) as Box<dyn FnMut(AnimationEvent)>)

}

fn transitionend_callback() -> Closure<dyn FnMut(web_sys::TransitionEvent)> {
    Closure::wrap(Box::new(move |event: TransitionEvent| {
        let event_target = event.target().unwrap();
        let target_element = event_target.dyn_ref::<HtmlElement>().unwrap();

        let target_element_class = target_element.class_name();
        let target_element_classes = target_element_class.split(" ");
        let mut target_element_selector = String::new();

        for class in target_element_classes {
            target_element_selector = format!("{}.{}", target_element_selector, class);
        }

        if target_element.class_name() == "metadata" {
            target_element.style().set_property("transition", "var(--hover_transition_duration) background-color").unwrap();
        } else if target_element_selector == ".gameover.won" || target_element_selector == ".gameover.lost" {
            let text_selector = format!("{}>.text", target_element_selector);
            let text_node = document().query_selector(&text_selector).unwrap().unwrap();
            let text_element = text_node.dyn_ref::<HtmlElement>().unwrap();
            text_element.class_list().add_1("gameover_typed").unwrap();
        }
    }) as Box<dyn FnMut(TransitionEvent)>)
}

fn increment_counter(input_counter: Arc<AtomicU16>) {
    // log!("Incrementing", input_counter.load(Ordering::SeqCst));
    input_counter.fetch_add(1, Ordering::SeqCst);
}

fn decrement_counter(input_counter: Arc<AtomicU16>) {
    // log!("Decrementing", input_counter.load(Ordering::SeqCst));
    input_counter.fetch_sub(1, Ordering::SeqCst);
}

#[function_component(Content)]
fn content() -> Html {
    let game_state = Rc::new(RefCell::new(Game::new()));
    let game_state_for_move_processor = Rc::clone(&game_state);
 
    // Prevents use of arrow keys for scrolling the page
    preventDefaultScrolling();

    // Attach a keydown event listener to the document.
    let (keydown_tx, keydown_rx) = mpsc::unbounded_channel();
    let input_counter = Arc::new(AtomicU16::new(0));

    let keydown_handler = Arc::new(Closure::wrap(produce_keydown_handler(keydown_tx, input_counter.clone())));
    let keydown_handler_clone = keydown_handler.clone();
    let keep_playing_clone = keydown_handler.clone();

    // Channel for animationend events to notify the keydown processor to process the next keystroke.
    let (animationend_tx, animationend_rx) = counted_channel::CountedChannel::new();

    // Channel for communicating to keydown processor that a re-render was triggered and to ignore
    // all remaining keyboard inputs in queue.
    // let (re_render_tx, re_render_rx) = mpsc::unbounded_channel();

    spawn_local(process_keydown_messages(game_state_for_move_processor, keydown_rx, animationend_rx, input_counter.clone(), keydown_handler_clone));

    use_effect(move || {
        let document = gloo::utils::document();
        document.add_event_listener_with_callback("keydown", Closure::as_ref(&keydown_handler).unchecked_ref()).unwrap();


        || {
            let document = gloo::utils::document();
            document.remove_event_listener_with_callback("keydown", Closure::as_ref(&keydown_handler).unchecked_ref()).unwrap();
            drop(keydown_handler)
        }
    });

    use_effect(move || {
        let body = gloo::utils::body();

        let animationend_callback = animationend_callback(input_counter.clone(), animationend_tx);

        body.add_event_listener_with_callback("animationend", animationend_callback.as_ref().unchecked_ref()).unwrap();
        body.add_event_listener_with_callback("animationcancel", animationend_callback.as_ref().unchecked_ref()).unwrap();

        || {
            let body = gloo::utils::body();
            body.remove_event_listener_with_callback("animationend", animationend_callback.as_ref().unchecked_ref()).unwrap();
            body.remove_event_listener_with_callback("animationcancel", animationend_callback.as_ref().unchecked_ref()).unwrap();
            drop(animationend_callback)
        }
    });

    // Set transitionend listener for when game over layer transitions from hidden to visible.
    // The transition property for the buttons must be overwritten to allow for their color to
    // change when hovered over.
    use_effect(move || {
        let body = gloo::utils::body();

        let transitionend_callback = transitionend_callback();

        body.add_event_listener_with_callback("transitionend", transitionend_callback.as_ref().unchecked_ref()).unwrap();

        || {
            let body = gloo::utils::body();
            body.remove_event_listener_with_callback("transitionend", transitionend_callback.as_ref().unchecked_ref()).unwrap();
            drop(transitionend_callback)
        }
    });

    // use_state() hook is used to trigger a re-render whenever the `New Game` button is clicked.
    let new_game = use_state(|| 0);
    let new_game_render = *new_game.clone();
    let new_game_callback = new_game_callback(new_game.clone());
    let keep_playing_callback = keep_playing_callback(keep_playing_clone);
    let placeholder_callback = Callback::from(|_| {});

    html! {
        <div class="content noselect" key={new_game_render}>
            <MetadataContainer score={0} onclick={&new_game_callback}/>
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
                // GameLostLayer does not use `keep_playing_callback` but not worth creating a
                // separate props struct for this.
                <GameWonLayer new_game_callback={&new_game_callback} keep_playing_callback={&keep_playing_callback}/>
                <GameLostLayer new_game_callback={&new_game_callback} keep_playing_callback={&placeholder_callback}/>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct MetadataContainerProps {
    onclick: Callback<MouseEvent>,
    score: u32,
}

#[function_component(MetadataContainer)]
fn metadata_container(props: &MetadataContainerProps) -> Html {
    html! {
        <div class="metadata-container">
            <Score score={props.score}/>
            <NewGameButton onclick={props.onclick.clone()} button_text={"New Game"} disabled={false}/>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ScoreProps {
    score: u32,
}

#[function_component(Score)]
fn score(props: &ScoreProps) -> Html {
    let style_args = format!("--button_border: {};
                              --button_background: {};
                              --button_text: {};",
                              COLORS.text_dark,
                              COLORS.button,
                              COLORS.text_dark,
                              );

    html! {
        <div class="metadata score" style={style_args}>{props.score}</div>
    }
}

#[derive(Properties, PartialEq)]
struct NewGameProps {
    onclick: Callback<MouseEvent>,
    button_text: String,
    disabled: bool,
}

#[function_component(NewGameButton)]
fn new_game_button(props: &NewGameProps) -> Html {
    let style_args = format!("--button_border: {};
                              --button_background: {};
                              --button_text: {};
                              --button_hover: {};
                              --hover_transition_duration: {}s",
                              COLORS.text_dark,
                              COLORS.button,
                              COLORS.text_dark,
                              COLORS.button_hover,
                              0.20,
                              );

    html! {
        <button class="metadata" onclick={props.onclick.clone()} disabled={props.disabled} style={style_args}>{ &props.button_text }</button>
    }
}

#[derive(Properties, PartialEq)]
struct GameOverProps {
    new_game_callback: Callback<MouseEvent>,
    keep_playing_callback: Callback<MouseEvent>,
}

#[function_component(GameWonLayer)]
fn game_won_layer(props: &GameOverProps) -> Html {
    let style_args = game_over_layer_style_args(true);

    html! {
        <div hidden=true class="gameover won" style={style_args}>
            <div class="text">{"VICTORY"}</div>
            <div class="buttons">
                <NewGameButton onclick={props.keep_playing_callback.clone()} button_text={"Keep Playing"} disabled={true}/>
                <NewGameButton onclick={props.new_game_callback.clone()} button_text={"Start Over"} disabled={true}/>
            </div>
        </div>
    }
}

#[function_component(GameLostLayer)]
fn game_lost_layer(props: &GameOverProps) -> Html {
    let style_args = game_over_layer_style_args(false);

    html! {
        <div hidden=true class="gameover lost" style={style_args}>
            <div class="text">{"DEFEAT"}</div>
            <div class="buttons">
                <NewGameButton onclick={props.new_game_callback.clone()} button_text={"Start Over"} disabled={true}/>
            </div>
        </div>
    }
}

fn game_over_layer_style_args(victory: bool) -> String {
    let (layer_color, text_color) = if victory {(COLORS.text_light, COLORS.text_dark)} else {(COLORS.button_hover, COLORS.text_dark)};

    format!("--game_over: {}{};
              --game_over_hidden: {}00;
              --button_border_hidden: {}00;
              --button_background_hidden: {}00;
              --button_text_hidden: {}00;
              --game_over_text: {};
              --game_over_text_hidden: {}00;
              --fade_in_duration: {}s; --fade_in_delay: {}s;",
              layer_color, COLORS.opacity,
              COLORS.text_light,
              COLORS.text_dark,
              COLORS.button,
              COLORS.text_dark,
              text_color,
              text_color,
              0.5, 0.0
            )
}

#[function_component(Header)]
fn header() -> Html {
    let header_style = format!("--header_text: {}", COLORS.text_light);

    html! {
        <div class="header" style={header_style}>
            <br/>
            <div class="typed">{ "Welcome to 2048!" }</div>
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
        font_size = "2.05em";
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

