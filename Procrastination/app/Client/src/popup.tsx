import "./popup-reset.css";
import React from "react";
import ReactDOM from "react-dom/client";
import "./global.css";
import { useEffect, useState } from "react";
import { PopUp, BreakData } from "./Components/PopUp";
import { BreakPopUp } from "./Components/BreakPopUp";
import { FocusPopUp } from "./Components/FocusPopUp";
import { IdlePopUp } from "./Components/IdlePopUp";

const PopupHost = () => {
  const [breakData, setBreakData] = useState<BreakData | null>(null);

  useEffect(() => {
    const theme = localStorage.getItem("theme") ?? "dark";
    document.documentElement.setAttribute("data-theme", theme);
  }, []);

  const handleBreakEnd = async () => {
    setBreakData(null);
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      await win.hide();
    } catch (e) {
      console.error("Failed to hide popup window after break:", e);
    }
  };

  return (
    <>
      <PopUp onBreakStart={(data) => setBreakData(data)} />
      {breakData && (
        <BreakPopUp breakData={breakData} onBreakEnd={handleBreakEnd} />
      )}
      <FocusPopUp />
      <IdlePopUp />
    </>
  );
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <PopupHost />
  </React.StrictMode>,
);
