import { invoke } from "@tauri-apps/api/core";
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

export const Settings = ({ theme, setTheme }: SettingsProps) => {
  const [activities, setActivities] = useState<string[]>([]);
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

  useEffect(() => {
    // get_saved_activities returns only activities the user saved in onboarding
    // (from user_preferences), not the full catalog from get_preference.
    invoke<string[]>("get_saved_activities")
      .then(setActivities)
      .catch(() => setActivities([]));
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
            <p className="metric-label">Correction rate</p>
            <p className="metric-value">
              {retrainingStatsLoading ? "…" : correctionPercent !== null ? `${correctionPercent}%` : "—"}
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
              }`}
            >
              {retrainingStatsLoading
                ? "…"
                : retrainingStats
                  ? retrainingStats.retraining_needed
                    ? "Retraining recommended"
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
                    : "Not enough data"}
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
        <p className="status-secondary">Version 0.1.0</p>
        <p className="status-secondary">Procrastination detection and intervention system</p>
      </section>
    </div>
  );
};
