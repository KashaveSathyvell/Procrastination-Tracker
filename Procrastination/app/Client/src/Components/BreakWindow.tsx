import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState, useCallback } from "react";
import "./BreakWindow.css";

type BreakInitPayload = {
  activity: string;
  duration: number;
  breakSessionId: number;
  interventionId: number;
};

type BreakState = "active" | "expired";

const formatTime = (seconds: number) => {
  const mins = Math.floor(seconds / 60)
    .toString()
    .padStart(2, "0");
  const secs = (seconds % 60).toString().padStart(2, "0");
  return `${mins}:${secs}`;
};

export const BreakWindow = () => {
  const [breakData, setBreakData] = useState<BreakInitPayload | null>(null);
  const [secondsLeft, setSecondsLeft] = useState(0);
  const [breakState, setBreakState] = useState<BreakState>("active");
  const hasInitializedRef = useRef(false);

  useEffect(() => {
    let cancelled = false;

    const loadInit = async () => {
      try {
        let data = await invoke<BreakInitPayload | null>("get_break_init_data");
        if ((data === null || data === undefined) && !cancelled) {
          await new Promise((resolve) => setTimeout(resolve, 200));
          data = await invoke<BreakInitPayload | null>("get_break_init_data");
        }
        if (cancelled || data === null || data === undefined || hasInitializedRef.current) return;

        hasInitializedRef.current = true;
        setBreakData(data);
        setSecondsLeft(Math.max(0, data.duration) * 60);
        setBreakState("active");
      } catch (e) {
        console.error("get_break_init_data failed:", e);
      }
    };

    loadInit();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let unlistenFn: (() => void) | null = null;

    const setup = async () => {
      const unlisten = await listen<BreakInitPayload>("break_init_data", (event) => {
        const data = event.payload;
        hasInitializedRef.current = true;
        setBreakData(data);
        setSecondsLeft(Math.max(0, data.duration) * 60);
        setBreakState("active");
      });
      unlistenFn = unlisten;
    };

    setup().catch((e) => {
      console.error("Failed to listen for break_init_data:", e);
    });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  useEffect(() => {
    if (!breakData || breakState === "expired") return;
    if (secondsLeft <= 0) {
      setBreakState("expired");
      return;
    }

    const timer = setInterval(() => {
      setSecondsLeft((prev) => {
        const next = prev - 1;
        return next < 0 ? 0 : next;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [breakData, breakState, secondsLeft]);

  const closeBreakWindow = useCallback(async () => {
    await emit("break_window_closed");
    await getCurrentWindow().close();
  }, []);

  const handleImBack = useCallback(async () => {
    if (!breakData) return;

    try {
      await invoke("break_end", {
        endBreak: {
          breakSessionId: breakData.breakSessionId,
          returnedOnTime: breakState === "active",
        },
      });
      await closeBreakWindow();
    } catch (e) {
      console.error("end_break failed:", e);
    }
  }, [breakData, breakState, closeBreakWindow]);

  const handleNeedMoreTime = () => {
    if (!breakData) return;
    setSecondsLeft((prev) => prev + 5 * 60);
    setBreakState("active");
    invoke("extend_break", { breakSessionId: breakData.breakSessionId, extraMinutes: 5 }).catch((e) => {
      console.error("extend_break failed:", e);
    });
  };

  //bring window to front when countdown = 0 & self-destruct if 5 mins no response
  useEffect(() => {
    if (breakState === "expired") {
      const bringToFront = async () => {
        try {
          const win = getCurrentWindow();
          await win.unminimize();
          await win.setAlwaysOnTop(true);
          await win.setFocus();
          await win.setAlwaysOnTop(false); 
        } catch (e) {
          console.error("Failed to bring window to front:", e);
        }
      };
      bringToFront();

      // Start 5-minute self-destruct timer
      const timeoutId = setTimeout(() => {
        handleImBack();
      }, 300000);

      return () => clearTimeout(timeoutId);
    }
  }, [breakState, handleImBack]);

  if (!breakData) {
    return (
      <div className="break-window-shell">
        <div className="break-window-card">
          <p className="break-window-title">Starting your break...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="break-window-shell">
      <div className="break-window-card">
        <p className="break-window-title">Break Time</p>
        <p className="break-window-activity">{breakData.activity}</p>

        {breakState === "expired" ? (
          <p className="break-window-expired">Time&apos;s up! Ready to get back?</p>
        ) : (
          <p className="break-window-timer">{formatTime(secondsLeft)}</p>
        )}

        <div className="break-window-actions">
          <button className="break-btn-primary" onClick={handleImBack}>
            I&apos;m back
          </button>
          <button className="break-btn-secondary" onClick={handleNeedMoreTime}>
            Need more time
          </button>
        </div>
      </div>
    </div>
  );
};
