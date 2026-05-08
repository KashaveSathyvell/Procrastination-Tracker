import "./popup-reset.css";
import React from "react";
import ReactDOM from "react-dom/client";
import "./global.css";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { PopUp } from "./Components/PopUp";
import { FocusPopUp } from "./Components/FocusPopUp";
import { IdlePopUp } from "./Components/IdlePopUp";

const savedTheme = localStorage.getItem("theme") ?? "dark";
document.documentElement.setAttribute("data-theme", savedTheme);

const PopupHost = () => {
  useEffect(() => {
    const theme = localStorage.getItem("theme") ?? "dark";
    document.documentElement.setAttribute("data-theme", theme);
  }, []);

  useEffect(() => {
    let unlistenFn: (() => void) | null = null;
    const setup = async () => {
      const unlisten = await listen("break_window_closed", async () => {
        console.log("Break window closed event received");
        try {
          const win = getCurrentWindow();
          if (win.label !== "main") {
            await win.setAlwaysOnTop(false);
            await win.hide();
          }
        } catch (e) {
          console.error("Failed to hide popup after break close:", e);
        }
      });
      unlistenFn = unlisten;
    };

    setup().catch((e) => {
      console.error("Failed to register break_window_closed listener:", e);
    });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  return (
    <>
      <PopUp />
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
