import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { Sidebar } from "./Components/Sidebar";
import { Onboarding } from "./Components/Onboarding";
import { Dashboard } from "./Components/Pages/Dashboard";
import { Analytics } from "./Components/Pages/Analytics";
import { History } from "./Components/Pages/History";
import { Settings } from "./Components/Pages/Settings";

type PageKey = "dashboard" | "analytics" | "history" | "settings";
type ThemeMode = "dark" | "light";

export type PredictionItem = {
  prediction_id: number;
  prediction_label: string;
  confidence: number;
  timestamp: number;
};

type RecentPredictionRow = {
  prediction_id: number;
  timestamp: number;
  predicted_state: string;
  confidence: number;
  was_corrected: boolean;
  user_label: string | null;
};

function App() {
  const [hasPreferences, setHasPreferences] = useState<boolean | null>(null);
  const [currentPage, setCurrentPage] = useState<PageKey>("dashboard");
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [predictions, setPredictions] = useState<PredictionItem[]>([]);
  const [totalPredictions, setTotalPredictions] = useState(0);
  const [theme, setTheme] = useState<ThemeMode>(() => {
    const savedTheme = localStorage.getItem("theme");
    return savedTheme === "light" ? "light" : "dark";
  });

  const isStateNotManagedError = (err: unknown) => {
    const msg =
      typeof err === "string"
        ? err
        : (err as any)?.message
          ? String((err as any).message)
          : String(err);
    return (
      msg.includes("state not managed") ||
      msg.includes("must call") && msg.includes(".manage()") ||
      msg.includes(".manage()")
    );
  };

  useEffect(() => {
    localStorage.setItem("theme", theme);
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  useEffect(() => {
    let cancelled = false;

    const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

    const run = async () => {
      // On first launch after a restart, Rust managed state may not yet be registered.
      // Retry only for that specific "state not managed / must call .manage()" failure.
      const maxAttempts = 10;
      const delayMs = 250;

      for (let attempt = 0; attempt < maxAttempts; attempt++) {
        try {
          const result = await invoke<boolean>("preference_exist");
          if (cancelled) return;
          setHasPreferences(result);
          return;
        } catch (err) {
          if (cancelled) return;
          const shouldRetry = isStateNotManagedError(err) && attempt < maxAttempts - 1;

          if (!shouldRetry) {
            console.error("preference_exist failed:", err);
            setHasPreferences(true); // existing fallback behavior
            return;
          }

          await sleep(delayMs);
        }
      }
    };

    run().catch((err) => {
      if (cancelled) return;
      console.error("preference_exist failed (unexpected):", err);
      setHasPreferences(true); // existing fallback behavior
    });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!hasPreferences) return;

    let mounted = true;
    invoke<RecentPredictionRow[]>("get_recent_predictions")
      .then((rows) => {
        if (!mounted || !Array.isArray(rows) || rows.length === 0) return;
        const hydrated = rows
          .slice(0, 10)
          .map((row) => ({
            prediction_id: row.prediction_id,
            prediction_label: row.predicted_state,
            confidence: row.confidence,
            timestamp: row.timestamp * 1000,
          }));
        setPredictions(hydrated);
      })
      .catch(() => {
        // Keep predictions empty on hydration failure by design.
      });

    invoke<number>("get_total_predictions_today")
      .then((count) => {
        if (!mounted) return;
        setTotalPredictions(Number.isFinite(count) ? count : 0);
      })
      .catch(() => {
        if (!mounted) return;
        setTotalPredictions(0);
      });

    return () => {
      mounted = false;
    };
  }, [hasPreferences]);

  useEffect(() => {
    if (!hasPreferences) return;

    let unlistenFn: (() => void) | null = null;
    const setup = async () => {
      const unlisten = await listen<PredictionItem>("new_prediction", (event) => {
        setPredictions((prev) => [event.payload, ...prev].slice(0, 10));
        setTotalPredictions((prev) => prev + 1);
      });
      unlistenFn = unlisten;
    };

    setup().catch(() => {
      // If listener setup fails, Dashboard simply remains on hydrated/empty state.
    });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, [hasPreferences]);

  useEffect(() => {
    let unlistenFn: (() => void) | null = null;
    const setup = async () => {
      const unlisten = await listen("break_window_closed", () => {
        console.log("Break window closed");
      });
      unlistenFn = unlisten;
    };

    setup().catch((e) => {
      console.error("Failed to listen for break_window_closed:", e);
    });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  const toggleTheme = () => {
    setTheme((prevTheme) => (prevTheme === "dark" ? "light" : "dark"));
  };

  const renderPage = () => {
    if (currentPage === "dashboard") {
      return (
        <Dashboard
          predictions={predictions}
          totalPredictions={totalPredictions}
          isMonitoring={isMonitoring}
          onMonitoringChange={setIsMonitoring}
        />
      );
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
        <>
          <main className="app-layout">
            <Sidebar
              currentPage={currentPage}
              onNavigate={(page) => setCurrentPage(page as PageKey)}
              theme={theme}
              onThemeToggle={toggleTheme}
            />
            <section className="app-content">{renderPage()}</section>
          </main>
        </>
      )}
    </div>
  );
}

export default App;
