//NOTE: don't remove
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, ops::Deref, process::Stdio};

use tokio::{
    sync::futures,
    task::JoinSet,
    time::{sleep, Duration},
};

use anyhow::Result;
use lazy_static::lazy_static;
use opening::{Opening, OpeningList};

use pgnparse::parser::PgnInfo;
use pgnparse::parser::*;

use serde::Serialize;

use tokio::{self, io::AsyncWriteExt, process::Command};

pub mod opening;

lazy_static! {
    static ref OPENINGS: OpeningList = OpeningList::new();
}

#[derive(Clone, Debug)]
pub struct CustomPgnInfo {
    pub headers: HashMap<String, String>,
    pub moves: Vec<String>,
}

impl std::fmt::Display for CustomPgnInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut headers = String::new();
        let mut moves = String::new();

        for (key, value) in &self.headers {
            headers.push_str(&format!("[{} \"{}\"]\n", key, value));
        }

        for move_ in &self.moves {
            moves.push_str(&format!("{}\n", move_));
        }

        write!(f, "{}\n\n{}", headers, moves)
    }
}

#[derive(Debug, Serialize)]
pub struct GameInfo {
    pub players: (String, String),
    pub result: String,
    pub link: String,
    pub eval: Vec<(String, i32)>,
}

#[derive(Serialize, Debug)]
pub struct GamesAnalytics {
    pub won: u32,
    pub drawn: u32,
    pub played: u32,
    pub openings_played: HashMap<String, u32>,
    pub games: Vec<GameInfo>,
}

pub struct AnalysedGames {
    pub games: Vec<Vec<(String, i32)>>,
}

pub async fn get_eval(fen: String) -> (String, i32) {
    let mut stockfish = Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Stockfish.");

    println!("Stockfish started");

    let mut stdin = stockfish.stdin.take().unwrap();

    stdin
        .write_all(format!("position fen {}\n", fen).as_bytes())
        .await
        .unwrap();
    stdin.write_all(b"go depth 15\n").await.unwrap();

    sleep(Duration::from_millis(300)).await;

    stdin.write_all(b"quit\n").await.unwrap();

    let res = stockfish.wait_with_output().await.unwrap();

    let output = String::from_utf8_lossy(&res.stdout)
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<String>>();

    //second to last line
    let line_with_cp = &output[&output.len() - 2];

    // find word cp the next word is the eval
    let mut eval = 0;

    let words = line_with_cp.split_whitespace();

    for (index, word) in words.clone().enumerate() {
        if word == "cp" {
            eval = words
                .clone()
                .nth(index + 1)
                .unwrap_or("0")
                .parse::<i32>()
                .unwrap_or(0);
        }
    }

    (fen, eval)
}

pub async fn game_analysis(game: CustomPgnInfo) -> Vec<(String, i32)> {
    let mut analysed_game = Vec::new();
    let mut handles = JoinSet::new();

    for move_ in &game.moves {
        let fen = move_;

        if let Some(index) = fen.find('-') {
            let fen = fen[..=index].to_string();
            handles.spawn(get_eval(fen));
        }
    }

    while let Some(result) = handles.join_next().await {
        analysed_game.push(result.unwrap());
    }

    analysed_game
}

pub async fn get_games_data(games: Vec<&CustomPgnInfo>, white: bool) -> GamesAnalytics {
    let mut analytics = GamesAnalytics {
        won: 0,
        drawn: 0,
        played: 0,
        openings_played: HashMap::new(),
        games: Vec::new(),
    };

    for game in games {
        let mut opening_played = String::new();
        let mut won = false;
        let mut drawn = false;

        let mut players = (String::new(), String::new());
        let mut result = String::new();
        let mut link = String::new();

        game.headers.iter().for_each(|(key, value)| {
            if key == "Result" {
                result = value.to_string();
            } else if key == "White" {
                players.0 = value.to_string();
            } else if key == "Black" {
                players.1 = value.to_string();
            } else if key == "Site" {
                link = value.to_string();
            }
        });

        let opening = Some(Opening {
            name: String::from("testing"),
            eco: String::from("testing"),
        });

        let opening_name = &opening.as_ref().unwrap().name;

        let base_opening = opening_name.split(':').next().unwrap();

        opening_played = base_opening.to_string();

        if result == "1-0" {
            if white {
                won = true;
            }
        } else if result == "0-1" {
            if !white {
                won = true;
            }
        } else if result == "1/2-1/2" {
            drawn = true;
        }

        if won {
            analytics.won += 1;
        } else if drawn {
            analytics.drawn += 1;
        }

        analytics.played += 1;

        if let Some(opening) = analytics.openings_played.get_mut(&opening_played) {
            *opening += 1;
        } else {
            analytics.openings_played.insert(opening_played, 1);
        }

        let game_evals = tokio::task::spawn(game_analysis(game.clone()));

        let game_info = GameInfo {
            players,
            result,
            link,
            eval: game_evals.await.unwrap(),
        };

        if game_info.players.0.is_empty()
            || game_info.players.1.is_empty()
            || game_info.result.is_empty()
            || game_info.link.is_empty()
        {
            break;
        }
        println!("{:?}", &game_info);

        analytics.games.push(game_info);
    }

    analytics
}

pub fn get_opening(moves: &PgnInfo) -> Option<&Opening> {
    let moves_iter = moves.moves.iter();
    let mut used_opening = None;

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

    used_opening
}

#[tauri::command]
async fn parse_pgn_data(multiple_pgn_string: &str, username: &str) -> Result<String, ()> {
    let mut pgn_vec = multiple_pgn_string.split("\n\n");

    let mut games: Vec<CustomPgnInfo> = Vec::new();

    while pgn_vec.clone().count() > 0 {
        let pgn = pgn_vec.next().unwrap();
        let game = pgn_vec.next().unwrap();

        let whole_game = format!("{}\n\n{}", pgn, game);

        let data = parse_pgn_to_rust_struct(whole_game);

        let customPgnInfo = CustomPgnInfo {
            headers: data.headers,
            moves: data
                .moves
                .iter()
                .map(|move_| move_.fen_after.clone())
                .collect(),
        };

        games.push(customPgnInfo);
    }

    let mut games_as_black = Vec::new();
    let mut games_as_white = Vec::new();

    games.iter().for_each(|game| {
        let binding = String::from("ERROR");
        let data = game.headers.get("Black").unwrap_or(&binding).to_lowercase();
        if data == username {
            games_as_black.push(game);
        } else if data != "ERROR" {
            games_as_white.push(game);
        }
    });

    let games_as_black_analytics = get_games_data(games_as_black, false).await;
    let games_as_white_analytics = get_games_data(games_as_white, true).await;

    let all_analytics = AllAnalytics {
        games_as_black_analytics,
        games_as_white_analytics,
    };

    println!("{:?}", serde_json::to_string(&all_analytics).unwrap());

    Ok(serde_json::to_string(&all_analytics).unwrap())
}

#[derive(Serialize, Debug)]
pub struct AllAnalytics {
    pub games_as_black_analytics: GamesAnalytics,
    pub games_as_white_analytics: GamesAnalytics,
}

fn main() -> Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_pgn_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
