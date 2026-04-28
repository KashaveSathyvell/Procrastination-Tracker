import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import React from "react";
import "./Pages.css";

type ThemeMode = "dark" | "light";

type SettingsProps = {
  theme: ThemeMode;
  setTheme: React.Dispatch<React.SetStateAction<ThemeMode>>;
};

export const Settings = ({ theme, setTheme }: SettingsProps) => {
  const [activities, setActivities] = useState<string[]>([]);
  const isDark = theme === "dark";

  useEffect(() => {
    invoke<string[]>("get_preference")
      .then(setActivities)
      .catch(() => setActivities([]));
  }, []);

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  return (
    <div className="page-shell">
      <header>
        <h1 className="page-title">Settings</h1>
        <p className="page-subtitle">Personalization and monitoring preferences.</p>
      </header>

      <section className="card settings-section">
        <h3 className="section-title">Theme</h3>
        <div className="theme-switch-row">
          <span>{isDark ? "Dark mode" : "Light mode"}</span>
          <button className={`theme-switch ${isDark ? "active" : ""}`} onClick={toggleTheme}>
            <span className="theme-switch-thumb" />
          </button>
        </div>
      </section>

      <section className="card settings-section">
        <h3 className="section-title">Monitoring Preferences</h3>
        <div className="settings-stat">
          <span className="status-secondary">Detection threshold</span>
          <span>75%</span>
        </div>
        {/* TODO: Make detection threshold configurable via Tauri settings command. */}
        <div className="settings-stat">
          <span className="status-secondary">Streak window</span>
          <span>15 minutes</span>
        </div>
      </section>

      <section className="card settings-section">
        <h3 className="section-title">Break Activities</h3>
        <div className="settings-pills">
          {activities.length > 0 ? (
            activities.map((activity) => (
              <span key={activity} className="pill">
                {activity}
              </span>
            ))
          ) : (
            <span className="status-secondary">No activities found.</span>
          )}
        </div>
        <p className="status-secondary">To change activities, restart onboarding.</p>
      </section>

      <section className="card settings-section">
        <h3 className="section-title">About</h3>
        <p>FocusGuard</p>
        <p className="status-secondary">Version 0.1.0</p>
        <p className="status-secondary">Procrastination detection and intervention system</p>
      </section>
    </div>
  );
};
