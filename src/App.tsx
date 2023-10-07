import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "preact/hooks";
import "./App.css";

function App() {
  async function fetch_user_data(username: string) {
    let png_to_parse = "";
    await fetch(
      "https://api.chess.com/pub/player/" + username + "/games/2023/7/pgn",
    )
      .then((response) => response.text())
      .then((data) => {
        png_to_parse = data;
      });

    await fetch(
      "https://api.chess.com/pub/player/" + username + "/games/2023/8/pgn",
    )
      .then((response) => response.text())
      .then((data) => {
        png_to_parse = png_to_parse + data;
      });

    await fetch(
      "https://api.chess.com/pub/player/" + username + "/games/2023/9/pgn",
    )
      .then((response) => response.text())
      .then((data) => {
        png_to_parse = png_to_parse + data;
      });

    parse_pgn_to_console(png_to_parse, username)
      .then(() => {
        console.log("done");
      })
      .catch((err) => {
        console.log("error: " + err);
      });
  }

  const [parsed_pgn, setParsedPgn] = useState<string>("");

  const [totalGames, setTotalGames] = useState<string>("");
  const [totalWins, setTotalWins] = useState<string>("");
  const [totalDraws, setTotalDraws] = useState<string>("");
  const [totalLosses, setTotalLosses] = useState<string>("");
  const [totalWinrate, setTotalWinrate] = useState<string>("");
  const [totalDrawrate, setTotalDrawrate] = useState<string>("");
  const [totalLossrate, setTotalLossrate] = useState<string>("");

  const [winrateAsBlack, setWinrateAsBlack] = useState<string>("");
  const [drawrateAsBlack, setDrawrateAsBlack] = useState<string>("");
  const [lossrateAsBlack, setLossrateAsBlack] = useState<string>("");
  const [totalBlackWins, setTotalBlackWins] = useState<string>("");
  const [totalBlackDraws, setTotalBlackDraws] = useState<string>("");
  const [totalBlackLosses, setTotalBlackLosses] = useState<string>("");

  const [winrateAsWhite, setWinrateAsWhite] = useState<string>("");
  const [drawrateAsWhite, setDrawrateAsWhite] = useState<string>("");
  const [lossrateAsWhite, setLossrateAsWhite] = useState<string>("");
  const [totalWhiteWins, setTotalWhiteWins] = useState<string>("");
  const [totalWhiteDraws, setTotalWhiteDraws] = useState<string>("");
  const [totalWhiteLosses, setTotalWhiteLosses] = useState<string>("");

  async function parse_pgn_to_console(pgnData: string, username: string) {
    await invoke("parse_pgn_data", { multiplePgnString: pgnData, username })
      .then((from_rust) => {
        if (typeof from_rust === "string") {
          let parsed: {
            games_as_black_analytics: {
              won: number;
              drawn: number;
              played: number;
              games: {
                eval: any[];
              };
            };
            games_as_white_analytics: {
              won: number;
              drawn: number;
              played: number;
              games: {
                eval: any[];
              };
            };
          } = JSON.parse(from_rust);

          // do the math with data
          let total_games =
            parsed.games_as_black_analytics.played +
            parsed.games_as_white_analytics.played;
          let total_wins =
            parsed.games_as_black_analytics.won +
            parsed.games_as_white_analytics.won;
          let total_draws =
            parsed.games_as_black_analytics.drawn +
            parsed.games_as_white_analytics.drawn;
          let total_losses = total_games - total_wins - total_draws;
          let total_winrate = (total_wins / total_games) * 100;
          let total_drawrate = (total_draws / total_games) * 100;
          let total_lossrate = (total_losses / total_games) * 100;

          setTotalGames(total_games.toString());
          setTotalWins(total_wins.toString());
          setTotalDraws(total_draws.toString());
          setTotalLosses(total_losses.toString());
          setTotalWinrate(total_winrate.toFixed(2));
          setTotalDrawrate(total_drawrate.toFixed(2));
          setTotalLossrate(total_lossrate.toFixed(2));
        } else {
          setParsedPgn("Error: " + from_rust);
        }
      })
      .catch((err) => {
        setParsedPgn("Error: " + err);
      });
  }

  return (
    <div class="container">
      <h1>Chess.com analysis</h1>
      <input
        type="text"
        id="username"
        name="username"
        placeholder="Enter username"
      />
      <button
        onClick={() => {
          fetch_user_data(
            (document.getElementById("username") as HTMLInputElement).value,
          );
        }}
      >
        Submit
      </button>
      <table>
        <thead>
          <tr>
            <th>Category</th>
            <th>Value</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Total Games</td>
            <td>{totalGames}</td>
          </tr>
          <tr>
            <td>Total Wins</td>
            <td>{totalWins}</td>
          </tr>
          <tr>
            <td>Total Draws</td>
            <td>{totalDraws}</td>
          </tr>
          <tr>
            <td>Total Losses</td>
            <td>{totalLosses}</td>
          </tr>
          <tr>
            <td>Total Winrate</td>
            <td>{totalWinrate}</td>
          </tr>
          <tr>
            <td>Total Drawrate</td>
            <td>{totalDrawrate}</td>
          </tr>
          <tr>
            <td>Total Lossrate</td>
            <td>{totalLossrate}</td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}

export default App;
