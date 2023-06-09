use yew::{prelude::*};
use rust_2048::*;
use gloo_console::log;
use gloo::events::EventListener;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Properties, PartialEq)]
struct GameBoardProps {
    game_state: Rc<RefCell<Game>>,
}

#[derive(Properties, PartialEq)]
struct ScoreProps {
    score: u64,
}

#[function_component(GameBoard)]
fn game_board(props: &GameBoardProps) -> Html {
// fn game_board() -> Html {
    html! {
        <table>
            { for (0..BOARD_DIMENSION).map(|i| {
                 let row = &props.game_state.borrow().board[i];
                 // let row = &GAME_STATE.board[i];
                 html! {
                     <tr class="row">
                         { for (0..BOARD_DIMENSION).map(|j| {
                             let tile = &row[j];
                             html! {
                                 // <td class="cell">{ "\u{00a0}" }</td>
                                 // <td class="cell"></td>
                                 // <td class="cell">{ props.board[i][j].map_or("\u{00a0}".to_string(), |t| t.to_string()) }</td>
                                 // <td class="cell" key={j}>{ props.board[i][j].as_ref().map_or("\u{00a0}".into(), |tile| tile.to_string()) }</td>
                                 <td class="cell">{ tile.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "\u{00a0}".to_string()) }</td>
                             }
                         })}
                     </tr>
                 }
             })}
        </table>        
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

#[function_component(Score)]
fn score(props: &ScoreProps) -> Html {
    html! {
        <div class="score">{props.score}</div>
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
                InputResult::Ok(()) => log!("Move successful!"),
                InputResult::Err(()) => log!("Move failed..."),
            }
        });

        // Called when the component is unmounted.  The closure has to hold on to `listener`, because if it gets
        // dropped, `gloo` detaches it from the DOM. So it's important to do _something_, even if it's just dropping it.
        || drop(listener)
    });

    // Not actually keeping track of the state of any variable.
    // use_state is being used to trigger a re-render whenever the 'New Game' button is clicked.
    // `true` is merely a placeholder value.
    let new_game = use_state(|| true);
    let onclick = {
        Callback::from(move |_| new_game.set(true))
    };

    html! {
        <div class="content">
            <div class="metadata">
                <Score score={game_state.borrow().score}/>
                <button class="new-game" onclick={onclick}>{ "New Game" }</button>
            </div>
            <div class="board-container">
                <GameBoard game_state={game_state}/>
                // <GameBoard/>
            </div>
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

// ArrowUp: 0xE048
// ArrowLeft: 0xE04B
// ArrowRight: 0xE04D
// ArrowDown: 0xE050
