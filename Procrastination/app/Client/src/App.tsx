import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Sidebar } from "./Components/Sidebar";
import { PopUp, BreakData } from "./Components/PopUp";
import { BreakPopUp } from "./Components/BreakPopUp";
import { Onboarding } from "./Components/Onboarding";
import { FocusPopUp } from "./Components/FocusPopUp";
import { IdlePopUp } from "./Components/IdlePopUp";
import { Dashboard } from "./Components/Pages/Dashboard";
import { Analytics } from "./Components/Pages/Analytics";
import { History } from "./Components/Pages/History";
import { Settings } from "./Components/Pages/Settings";

type PageKey = "dashboard" | "analytics" | "history" | "settings";
type ThemeMode = "dark" | "light";

function App() {
  const [breakData, setBreakData] = useState<BreakData | null>(null);
  const [hasPreferences, setHasPreferences] = useState<boolean | null>(null);
  const [currentPage, setCurrentPage] = useState<PageKey>("dashboard");
  const [theme, setTheme] = useState<ThemeMode>(() => {
    const savedTheme = localStorage.getItem("theme");
    return savedTheme === "light" ? "light" : "dark";
  });

  useEffect(() => {
    localStorage.setItem("theme", theme);
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  useEffect(() => {
    invoke<boolean>("preference_exist")
      .then(setHasPreferences)
      .catch(() => setHasPreferences(false));
  }, []);

  const toggleTheme = () => {
    setTheme((prevTheme) => (prevTheme === "dark" ? "light" : "dark"));
  };

  const renderPage = () => {
    if (currentPage === "dashboard") {
      return <Dashboard />;
    }
    if (currentPage === "analytics") {
      return <Analytics />;
    }
    if (currentPage === "history") {
      return <History />;
    }
    return <Settings theme={theme} setTheme={setTheme} />;
  };

  if (hasPreferences === null) return null; // loading state

  return (
    <div id="app-root" data-theme={theme}>
      {/* TODO: Call window.setAlwaysOnTop(true) for intervention windows in a future Tauri update. */}
      {!hasPreferences ? (
        <Onboarding onComplete={() => setHasPreferences(true)} />
      ) : (
        <main className="app-layout">
          <Sidebar
            currentPage={currentPage}
            onNavigate={(page) => setCurrentPage(page as PageKey)}
            theme={theme}
            onThemeToggle={toggleTheme}
          />
          <section className="app-content">{renderPage()}</section>
        </main>
      )}

      <PopUp onBreakStart={(data) => setBreakData(data)} />

      {breakData && (
        <BreakPopUp
          breakData={breakData}
          onBreakEnd={() => setBreakData(null)}
        />
      )}

      <FocusPopUp />
      <IdlePopUp />
    </div>
  );
}

export default App;
