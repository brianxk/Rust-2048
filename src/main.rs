use yew::prelude::*;
use rust_2048::*;

#[derive(Properties, PartialEq)]
struct Props {
    pub board: [[Option<Tile>; 4]; 4],
}

#[function_component(Game_Board)]
fn game_board() -> Html {
    html! {
        <table class="board">
            { for (0..BOARD_DIMENSION).map(|i| {
                 html! {
                     <tr class="row">
                         { for (0..BOARD_DIMENSION).map(|j| {
                             html! {
                                 // <td class="cell">{ "\u{00a0}" }</td>
                                 <td class="cell"></td>
                             }
                         })}
                     </tr>
                 }
             })}
        </table>        
    }
}

#[function_component(App)]
fn app() -> Html {
    let game_state = Game::new();
    let board = game_state.board;

    html! {
        <>
            <div class="header">
                <h1>{ "Welcome to 2048!!" }</h1>
            </div>

            <div class="body">
                <div class="body-central">
                    <div class="metadata">
                        <New_Game_Button/>
                    </div>
                    <div class="board-container">
                        <Game_Board/>
                    </div>
                </div>
            </div>

            <div class="footer">
                <h1>
                    { "This project is a hobby imitation of the"}
                    <a href="https://play2048.co/" target="_blank">
                        { " original game " }
                    </a>
                    { "developed by Gabriele Cirulli." }
                </h1>
            </div>
        </>
    }
}

#[function_component(New_Game_Button)]
fn new_game_button() -> Html {
    // let handle_new_game = props.on_new_game.clone();

    html! {
        <button class="new-game" onclick={Callback::from(|_| ())}>{ "New Game" }</button>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
