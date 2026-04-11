import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import {StartStopButton} from "./Components/StartStopButton"

function App() {


  return (
    <main className="container">
      <StartStopButton></StartStopButton>
    </main>
  );
}

export default App;
