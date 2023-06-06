use yew::prelude::*;
use rust_2048::*;

#[derive(Properties, PartialEq)]
struct GameBoardProps {
    game_state: Game,
}

#[function_component(GameBoard)]
fn game_board(props: &GameBoardProps) -> Html {
    html! {
        <table>
            { for (0..BOARD_DIMENSION).map(|i| {
                 let row = &props.game_state.board[i];
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

#[function_component(Content)]
fn content() -> Html {
    let game_state = Game::new();
    
    // Not actually keeping track of the state of any variable.
    // use_state is being used to trigger a re-render whenever the button is clicked.
    // `true` is merely a placeholder value.
    let new_game = use_state(|| true);
    let onclick = {
        Callback::from(move |_| new_game.set(true))
    };

    html! {
        <div class="content">
            <div class="metadata">
                <button class="new-game" onclick={onclick}>{ "New Game" }</button>
            </div>
            <div class="board-container">
                <GameBoard game_state={game_state}/>
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

