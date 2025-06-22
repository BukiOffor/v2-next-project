import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { check } from '@tauri-apps/plugin-updater';
import { ask, message } from '@tauri-apps/plugin-dialog';
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }



async function checkForAppUpdates(onUserClick: false) {
  const update = await check();
  if (update === null) {
			 await message('You are on the latest version. Stay awesome!', { 
      title: 'No Update Available',
      kind: 'info',
      okLabel: 'OK'
    });
			return;
		} else if (update?.available) {
    const yes = await ask(`Update to ${update.version} is available!\n\nRelease notes: ${update.body}`, { 
      title: 'Update Available',
      kind: 'info',
      okLabel: 'Update',
      cancelLabel: 'Cancel'
    });
    if (yes) {
      await update.downloadAndInstall();
      // Restart the app after the update is installed by calling the Tauri command that handles restart for your app
      // It is good practice to shut down any background processes gracefully before restarting
      // As an alternative, you could ask the user to restart the app manually
      await invoke("graceful_restart");
    }
  } else if (onUserClick) {
   
  }
}










  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <button onClick={ async () => await checkForAppUpdates(false)}>Check for Updates</button>
    </main>
  );
}

export default App;
