//NOTE: don't remove
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use lazy_static::lazy_static;
use opening::OpeningList;
use pgnparse::parser::*;

pub mod opening;

lazy_static! {
    static ref OPENINGS: OpeningList = OpeningList::new();
}

#[tauri::command]
fn parse_pgn(multiple_pgn_string: &str) -> String {
    let mut pgn_vec = multiple_pgn_string.split("\n\n");

    let mut games = Vec::new();

    while pgn_vec.clone().count() > 0 {
        let pgn = pgn_vec.next().unwrap();
        let game = pgn_vec.next().unwrap();

        let whole_game = format!("{}\n\n{}", pgn, game);

        games.push(parse_pgn_to_rust_struct(whole_game));
    }

    let mut list = String::new();

    games.iter().for_each(|game| {
        let result = game.headers.get("Result");
        let white = game.headers.get("White");
        let black = game.headers.get("Black");

        let mut used_opening = None;
        let moves_iter = game.moves.iter();

        for move_ in moves_iter {
            let move_fen = &move_.fen_after;
            if let Some(index) = move_fen.find('-') {
                let parsed_move = &move_fen[..=index]; // Include the '-' character
                if let Some(opening_struct) = OPENINGS.openings.get(parsed_move) {
                    used_opening = Some(opening_struct);
                } else {
                    break;
                }
            } else {
                continue;
            }
        }

        if result.is_none() || white.is_none() || black.is_none() {
            list.push_str(&format!("{:?} {:?} {:?}\n", white, result, black));
        } else {
            if used_opening.is_some() {
                list.push_str(&format!("{}:", used_opening.unwrap().name))
            }
            list.push_str(&format!(
                "{} {} {} moves: {}\n",
                white.unwrap(),
                result.unwrap(),
                black.unwrap(),
                game.moves.len()
            ));
        }
    });

    format!("{}\n\n{}", games.len(), list)
}

fn main() -> Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_pgn])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
