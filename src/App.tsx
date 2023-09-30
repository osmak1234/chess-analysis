import { useEffect, useState } from "preact/hooks";
// import preactLogo from "./assets/preact.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  // example
  // async function greet() {
  //   // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  //   setGreetMsg(await invoke("greet", { name }));
  // }

  function fetch_user_data(username: string) {
    fetch("https://api.chess.com/pub/player/" + username + "/games/2023/9/pgn")
      .then((response) => response.text())
      .then((data) => {
        parse_pgn_to_console(data);
      });
  }

  const [parsed_pgn, setParsedPgn] = useState<string>("");

  async function parse_pgn_to_console(pgnData: string) {
    await invoke("parse_pgn", { multiplePgnString: pgnData })
      .then((from_rust) => {
        if (typeof from_rust === "string") {
          setParsedPgn(from_rust);
        } else {
          setParsedPgn("Error: " + from_rust);
        }
      })
      .catch((err) => {
        setParsedPgn("Error: " + err);
      });
  }

  useEffect(() => {
    fetch_user_data("kupec_samo");
  }, []);

  return (
    <div class="container">
      {/* paragraph that linebreaks */}
      <p>
        {parsed_pgn.split("\n").map((i) => {
          return <p>{i}</p>;
        })}
      </p>
    </div>
  );
}

export default App;
