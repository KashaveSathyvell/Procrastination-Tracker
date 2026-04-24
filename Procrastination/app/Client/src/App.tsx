import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import {StartStopButton} from "./Components/StartStopButton"
import { PopUp, BreakData } from "./Components/PopUp";
import { BreakPopUp } from "./Components/BreakPopUp";
import { Onboarding } from "./Components/Onboarding";

function App() {
  const [breakData, setBreakData] = useState<BreakData | null>(null);
  const [hasPreferences, setHasPreferences] = useState<boolean | null>(null);

    useEffect(() => {
        invoke<boolean>("preference_exist")
            .then(setHasPreferences)
            .catch(() => setHasPreferences(false));
    }, []);

    if (hasPreferences === null) return null; // loading state

    if (!hasPreferences) {
        return <Onboarding onComplete={() => setHasPreferences(true)} />;
    }

  return (
    <main className="container">
      <StartStopButton></StartStopButton>
      <PopUp onBreakStart={(data) => setBreakData(data)} />

      {breakData && (
          <BreakPopUp
              breakData={breakData}
              onBreakEnd={() => setBreakData(null)}
          />
      )}
    </main>
  );
}

export default App;
