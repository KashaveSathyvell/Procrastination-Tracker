import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import React from "react";
import "./Pages.css";

type ThemeMode = "dark" | "light";

type SettingsProps = {
  theme: ThemeMode;
  setTheme: React.Dispatch<React.SetStateAction<ThemeMode>>;
};

type RetrainingStatsPayload = {
  correction_rate: number;
  labelled_count: number;
  retraining_needed: boolean;
};

type RetrainingResultPayload = {
  success: boolean;
  message: string;
};

type ActivityScorePayload = {
  activity_name: string;
  average_focus_score: number;
  times_completed: number;
  times_suggested: number;
};

type StreakSettingsPayload = {
    focusedStreakWindow: number;
    idleStreakWindow: number;
};

const FOCUS_STREAK_OPTIONS = [5, 8, 10, 12, 15, 20];
const IDLE_STREAK_OPTIONS = [5, 8, 10, 15, 20, 30];

export const Settings = ({ theme, setTheme }: SettingsProps) => {
  const [savedActivities, setSavedActivities] = useState<string[]>([]);
  const [allActivities, setAllActivities] = useState<string[]>([]);
  const [activitiesLoading, setActivitiesLoading] = useState(false);
  const [activitiesError, setActivitiesError] = useState<string | null>(null);

  const [focusedStreakWindow, setFocusedStreakWindow] = useState<number>(15);
  const [idleStreakWindow, setIdleStreakWindow] = useState<number>(10);
  const [streakSaveSuccess, setStreakSaveSuccess] = useState(false);
  const [streakSaveError, setStreakSaveError] = useState<string | null>(null);

  const [activityScores, setActivityScores] = useState<ActivityScorePayload[]>([]);
  const [activityScoresLoading, setActivityScoresLoading] = useState(true);
  const [retrainingStats, setRetrainingStats] = useState<RetrainingStatsPayload | null>(null);
  const [retrainingStatsLoading, setRetrainingStatsLoading] = useState(true);
  const [retrainingInProgress, setRetrainingInProgress] = useState(false);
  const [retrainFeedback, setRetrainFeedback] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const isDark = theme === "dark";

  const loadActivityScores = useCallback(() => {
    setActivityScoresLoading(true);
    invoke<ActivityScorePayload[]>("get_activity_scores")
      .then((scores) => {
        const sorted = [...scores].sort((a, b) => {
          const aCompleted = a.times_completed > 0;
          const bCompleted = b.times_completed > 0;
          if (aCompleted !== bCompleted) {
            return aCompleted ? -1 : 1;
          }
          if (!aCompleted && !bCompleted) {
            return a.activity_name.localeCompare(b.activity_name);
          }
          return b.average_focus_score - a.average_focus_score;
        });
        setActivityScores(sorted);
      })
      .catch(() => setActivityScores([]))
      .finally(() => setActivityScoresLoading(false));
  }, []);

  const loadRetrainingStats = useCallback(() => {
    setRetrainingStatsLoading(true);
    invoke<RetrainingStatsPayload>("check_retraining_needed")
      .then((stats) => {
        setRetrainingStats(stats);
      })
      .catch(() => {
        setRetrainingStats(null);
      })
      .finally(() => setRetrainingStatsLoading(false));
  }, []);

  const loadSavedActivities = useCallback(() => {
    return invoke<string[]>("get_saved_activities")
      .then((items) => {
        setSavedActivities(Array.isArray(items) ? items : []);
      })
      .catch(() => {
        setSavedActivities([]);
      });
  }, []);

  useEffect(() => {
    loadSavedActivities();
    invoke<string[]>("get_preference")
      .then((items) => {
        setAllActivities(Array.isArray(items) ? items : []);
      })
      .catch(() => {
        setAllActivities([]);
      });
  }, [loadSavedActivities]);

  useEffect(() => {
    invoke<StreakSettingsPayload>("get_streak_settings")
        .then((settings) => {
            const focusedVal = Number(settings?.focusedStreakWindow);
            setFocusedStreakWindow(FOCUS_STREAK_OPTIONS.includes(focusedVal) ? focusedVal : 15);
            const idleVal = Number(settings?.idleStreakWindow);
            setIdleStreakWindow(IDLE_STREAK_OPTIONS.includes(idleVal) ? idleVal : 10);
        })
        .catch(() => {
            setFocusedStreakWindow(15);
            setIdleStreakWindow(10);
        });
  }, []);

  useEffect(() => {
    loadRetrainingStats();
  }, [loadRetrainingStats]);

  useEffect(() => {
    loadActivityScores();
  }, [loadActivityScores]);

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  const correctionPercent =
    retrainingStats != null ? Math.round(Math.min(1, Math.max(0, retrainingStats.correction_rate)) * 100) : null;
  const accuracyPercent = correctionPercent !== null ? 100 - correctionPercent : null;

  const availableActivities = allActivities.filter((activity) => !savedActivities.includes(activity));

  const handleAddActivity = async (activity: string) => {
    setActivitiesLoading(true);
    setActivitiesError(null);
    try {
      await invoke("save_user_activity", { chosenList: [activity] });
      await loadSavedActivities();
    } catch (_e) {
      setActivitiesError("Could not add activity. Please try again.");
    } finally {
      setActivitiesLoading(false);
    }
  };

  const handleRemoveActivity = async (activity: string) => {
    if (savedActivities.length <= 1) {
        setActivitiesError("At least one activity must remain. Add another before removing this one.");
        return;
    }
    setActivitiesLoading(true);
    setActivitiesError(null);
    try {
        await invoke("delete_activity", { activityName: activity });
        await loadSavedActivities();
    } catch (_e) {
        setActivitiesError("Could not remove activity. Please try again.");
    } finally {
        setActivitiesLoading(false);
    }
  };

  const handleFocusedStreakChange = async (nextValue: number) => {
    const prevValue = focusedStreakWindow;
    setFocusedStreakWindow(nextValue);
    setStreakSaveError(null);
    setStreakSaveSuccess(false);
    try {
      await invoke("save_streak_settings", {
        settings: { focusedStreakWindow: nextValue, idleStreakWindow: idleStreakWindow },
      });
      setStreakSaveSuccess(true);
      setTimeout(() => setStreakSaveSuccess(false), 2000);
    } catch (_e) {
      setFocusedStreakWindow(prevValue);
      setStreakSaveError("Could not save setting. Reverted to previous value.");
    }
  };

  const handleIdleStreakChange = async (nextValue: number) => {
    const prevValue = idleStreakWindow;
    setIdleStreakWindow(nextValue);
    setStreakSaveError(null);
    setStreakSaveSuccess(false);
    try {
        await invoke("save_streak_settings", {
            settings: { focusedStreakWindow: focusedStreakWindow, idleStreakWindow: nextValue },
        });
        setStreakSaveSuccess(true);
        setTimeout(() => setStreakSaveSuccess(false), 2000);
    } catch (_e) {
        setIdleStreakWindow(prevValue);
        setStreakSaveError("Could not save setting. Reverted to previous value.");
    }
  };

  const handleRetrain = async () => {
    if (!retrainingStats?.retraining_needed) return;
    setRetrainFeedback(null);
    setRetrainingInProgress(true);
    try {
      const result = await invoke<RetrainingResultPayload>("trigger_retraining");
      setRetrainFeedback({
          kind: result.success ? "success" : "error",
          text: result.message,
      });
      if (result.success) {
          try {
              await emit("model-retrained");
          } catch (e) {
              console.error("Failed to emit model-retrained:", e);
          }
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setRetrainFeedback({ kind: "error", text: msg });
    } finally {
      setRetrainingInProgress(false);
    }
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
      </section>

      <section className="card settings-section">
        <h3 className="section-title">Detection Settings</h3>
        <div className="detection-settings-row">
          <label htmlFor="focused-streak-window">Focus check-in after:</label>
          <select
            id="focused-streak-window"
            value={focusedStreakWindow}
            onChange={(e) => handleFocusedStreakChange(Number(e.target.value))}
          >
            {FOCUS_STREAK_OPTIONS.map((minutes) => (
              <option key={minutes} value={minutes}>
                {minutes} minutes
              </option>
            ))}
          </select>
        </div>
        <div className="detection-settings-row">
          <label htmlFor="idle-streak-window">Idle check-in after:</label>
          <select
              id="idle-streak-window"
              value={idleStreakWindow}
              onChange={(e) => handleIdleStreakChange(Number(e.target.value))}
          >
              {IDLE_STREAK_OPTIONS.map((minutes) => (
                  <option key={minutes} value={minutes}>
                      {minutes} minutes
                  </option>
              ))}
          </select>
        </div>
        <p className="status-secondary">
            The system will check in if you appear idle for this long.
        </p>
        <p className="status-secondary">
          The system will check in if you appear focused for this long. Lower this if you tend to lose focus before 15
          minutes.
        </p>
        {streakSaveSuccess && <p className="settings-success">Saved</p>}
        {streakSaveError && <p className="settings-error">{streakSaveError}</p>}
      </section>

      <section className="card settings-section">
        <h3 className="section-title">Break Activities</h3>
        <div className="settings-pills">
          {savedActivities.length > 0 ? (
            savedActivities.map((activity) => (
              <span key={activity} className="pill settings-activity-pill">
                <span>{activity}</span>
                <button
                  type="button"
                  className="settings-activity-remove"
                  onClick={() => handleRemoveActivity(activity)}
                  disabled={activitiesLoading}
                  aria-label={`Remove ${activity}`}
                >
                  ✕
                </button>
              </span>
            ))
          ) : (
            <span className="settings-warning">Add at least one activity so the system can suggest breaks.</span>
          )}
        </div>

        {activitiesLoading && <p className="status-secondary">Updating activities...</p>}

        {availableActivities.length > 0 ? (
          <div className="settings-add-activities">
            <p className="status-secondary">Add activity</p>
            <div className="settings-pills">
              {availableActivities.map((activity) => (
                <button
                  key={activity}
                  type="button"
                  className="settings-add-activity-btn"
                  onClick={() => handleAddActivity(activity)}
                  disabled={activitiesLoading}
                >
                  {activity}
                </button>
              ))}
            </div>
          </div>
        ) : (
          <p className="status-secondary">All available activities added.</p>
        )}

        {activitiesError && <p className="settings-error">{activitiesError}</p>}
      </section>

      <section className="card settings-section activity-effectiveness-section">
        <div className="activity-effectiveness-header">
          <h3 className="section-title activity-effectiveness-title">Break Activity Effectiveness</h3>
          <button
            type="button"
            className="activity-effectiveness-refresh"
            onClick={loadActivityScores}
            disabled={activityScoresLoading}
          >
            Refresh
          </button>
        </div>

        {activityScoresLoading ? (
          <p className="status-secondary activity-effectiveness-loading">Loading activity scores...</p>
        ) : activityScores.length === 0 ? (
          <p className="status-secondary">
            No activity data yet. Complete some break sessions to see effectiveness scores.
          </p>
        ) : (
          <div className="activity-effectiveness-grid">
            {activityScores.map((activity) => {
              const scorePercent = Math.round(Math.min(1, Math.max(0, activity.average_focus_score)) * 100);
              const hasCompletions = activity.times_completed > 0;

              return (
                <article key={activity.activity_name} className="activity-effectiveness-card">
                  <h4 className="activity-effectiveness-name">{activity.activity_name}</h4>
                  {hasCompletions ? (
                    <>
                      <p className="activity-effectiveness-score-label">Focus score: {scorePercent}%</p>
                      <div className="activity-effectiveness-score-track" aria-hidden="true">
                        <div
                          className="activity-effectiveness-score-fill"
                          style={{ width: `${scorePercent}%` }}
                        />
                      </div>
                      <p className="status-secondary">Completed: {activity.times_completed} breaks</p>
                      <p className="status-secondary">Suggested: {activity.times_suggested} times</p>
                    </>
                  ) : (
                    <p className="status-secondary">Not yet tried</p>
                  )}
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="card settings-section model-performance-section">
        <div className="model-performance-header">
          <h3 className="section-title model-performance-title">Model Performance</h3>
          <button
            type="button"
            className="model-performance-refresh"
            onClick={() => {
              setRetrainFeedback(null);
              loadRetrainingStats();
            }}
            disabled={retrainingStatsLoading || retrainingInProgress}
          >
            Refresh
          </button>
        </div>

        <div className="model-performance-metrics">
          <div className="metric-card">
              <p className="metric-label">Model accuracy</p>
              <p className="metric-value">
                  {retrainingStatsLoading ? "…" : accuracyPercent !== null ? `${accuracyPercent}%` : "—"}
              </p>
          </div>
          <div className="metric-card">
              <p className="metric-label">Labelled training rows</p>
              <p className="metric-value">
                  {retrainingStatsLoading
                      ? "…"
                      : retrainingStats != null
                          ? `${retrainingStats.labelled_count} rows`
                          : "—"}
              </p>
              {!retrainingStatsLoading && correctionPercent !== null && (
                  <p className="status-secondary" style={{ marginTop: "4px", fontSize: "12px" }}>
                      {correctionPercent}% correction rate
                  </p>
              )}
          </div>
          <div className="metric-card">
            <p className="metric-label">Status</p>
            <p 
              className={`metric-value ${
                retrainingStatsLoading
                    ? ""
                    : retrainingStats?.retraining_needed
                        ? "model-performance-status-warn"
                        : "model-performance-status-ok"
            }`}>
                {retrainingStatsLoading
                    ? "…"
                    : retrainingStats
                        ? retrainingStats.retraining_needed
                            ? "Retraining recommended"
                            : retrainingStats.labelled_count >= 50
                                ? "Recently retrained"
                                : "Model performing well"
                        : "Could not load"}
            </p>
          </div>
        </div>

        <div className="model-performance-actions">
          <button
            type="button"
            className="model-performance-retrain-btn"
            disabled={
              retrainingInProgress ||
              retrainingStatsLoading ||
              !retrainingStats ||
              !retrainingStats.retraining_needed
            }
            onClick={handleRetrain}
          >
            {retrainingInProgress
              ? "Retraining in progress..."
              : retrainingStatsLoading
                ? "…"
                : !retrainingStats
                  ? "Unavailable"
                  : retrainingStats.retraining_needed
                    ? "Retrain Model"
                    : retrainingStats.labelled_count < 50
                    ? "Not enough data yet"
                    : "Model accuracy is good"}
          </button>
          {retrainingInProgress && (
            <p className="model-performance-duration-hint status-secondary">This may take up to a minute.</p>
          )}
          {retrainFeedback && (
            <p
              className={
                retrainFeedback.kind === "success" ? "model-performance-feedback-success" : "model-performance-feedback-error"
              }
            >
              {retrainFeedback.text}
            </p>
          )}
        </div>
      </section>

      <section className="card settings-section">
        <h3 className="section-title">About</h3>
        <p>FocusGuard</p>
        <p className="status-secondary">Version 0.1.5</p>
        <p className="status-secondary">Procrastination detection and intervention system</p>
      </section>
    </div>
  );
};
