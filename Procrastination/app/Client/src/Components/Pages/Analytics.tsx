import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import "./Pages.css";

type RangeKey = "today" | "week" | "month";

type AnalyticsStats = {
  focused: number;
  at_risk: number;
  procrastinating: number;
  idle: number;
  focused_count: number;
  at_risk_count: number;
  procrastinating_count: number;
  idle_count: number;
  total: number;
};

type AnalyticsFocusScore = {
  score: number;
  average_confidence: number;
  focused_percentage: number;
};

type RetrainingCheck = {
  correction_rate: number;
  labelled_count: number;
  retraining_needed: boolean;
};

const rangeKeyToDays: Record<RangeKey, number> = {
  today: 1,
  week: 7,
  month: 30,
};

const DISTRIBUTION_ROWS: {
  label: string;
  pctField: keyof Pick<AnalyticsStats, "focused" | "at_risk" | "procrastinating" | "idle">;
  countField: keyof Pick<
    AnalyticsStats,
    "focused_count" | "at_risk_count" | "procrastinating_count" | "idle_count"
  >;
  className: string;
}[] = [
  { label: "Focused", pctField: "focused", countField: "focused_count", className: "state-focused" },
  { label: "At Risk", pctField: "at_risk", countField: "at_risk_count", className: "state-at-risk" },
  {
    label: "Procrastinating",
    pctField: "procrastinating",
    countField: "procrastinating_count",
    className: "state-procrastinating",
  },
  { label: "Idle", pctField: "idle", countField: "idle_count", className: "state-idle" },
];

export const Analytics = () => {
  const [range, setRange] = useState<RangeKey>("today");
  const rangeDays = rangeKeyToDays[range];

  const [stats, setStats] = useState<AnalyticsStats | null>(null);
  const [statsLoading, setStatsLoading] = useState(true);

  const [focusPayload, setFocusPayload] = useState<AnalyticsFocusScore | null>(null);
  const [focusLoading, setFocusLoading] = useState(true);

  const [retraining, setRetraining] = useState<RetrainingCheck | null>(null);
  const [retrainingLoading, setRetrainingLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    setStatsLoading(true);
    invoke<AnalyticsStats>("get_analytics_stats", { rangeDays })
      .then((data) => {
        if (!cancelled) setStats(data);
      })
      .catch(() => {
        if (!cancelled) setStats(null);
      })
      .finally(() => {
        if (!cancelled) setStatsLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [rangeDays]);

  useEffect(() => {
    let cancelled = false;

    setFocusLoading(true);
    invoke<AnalyticsFocusScore>("get_analytics_focus_score", { rangeDays })
      .then((data) => {
        if (!cancelled) setFocusPayload(data);
      })
      .catch(() => {
        if (!cancelled) setFocusPayload(null);
      })
      .finally(() => {
        if (!cancelled) setFocusLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [rangeDays]);

  useEffect(() => {
    let cancelled = false;

    setRetrainingLoading(true);
    invoke<RetrainingCheck>("check_retraining_needed")
      .then((data) => {
        if (!cancelled) setRetraining(data);
      })
      .catch(() => {
        if (!cancelled) setRetraining(null);
      })
      .finally(() => {
        if (!cancelled) setRetrainingLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [rangeDays]);

  const correctionPercent = useMemo(() => {
    if (!retraining) return null;
    return Math.round(Math.min(1, Math.max(0, retraining.correction_rate)) * 100);
  }, [retraining]);

  const focusDisplayScore = useMemo(() => {
    if (!focusPayload) return null;
    return Math.round(Math.min(100, Math.max(0, focusPayload.score)));
  }, [focusPayload]);

  return (
    <div className="page-shell">
      <header>
        <h1 className="page-title">Analytics</h1>
        <p className="page-subtitle">Session trends and quality indicators.</p>
      </header>

      <section className="card range-toggle">
        {[
          { key: "today", label: "Today" },
          { key: "week", label: "This Week" },
          { key: "month", label: "This Month" },
        ].map((item) => (
          <button
            key={item.key}
            className={`pill-toggle ${range === item.key ? "active" : ""}`}
            onClick={() => setRange(item.key as RangeKey)}
          >
            {item.label}
          </button>
        ))}
      </section>

      <section className="card">
        <h3 className="section-title">State Distribution</h3>
        {statsLoading ? (
          <div className="analytics-widget-loading" aria-hidden="true">
            <span className="analytics-loading-dots">Loading</span>
          </div>
        ) : !stats || stats.total === 0 ? (
          <p className="analytics-empty-message status-secondary">
            No data for this period — start a monitoring session to see your analytics.
          </p>
        ) : (
          <div className="bar-chart">
            {DISTRIBUTION_ROWS.map((row) => {
              const pct = Math.min(100, Math.max(0, stats[row.pctField]));
              const count = stats[row.countField];
              return (
                <div key={row.label} className="bar-item">
                  <div className="bar-track">
                    <div className={`bar-fill ${row.className}`} style={{ height: `${pct}%` }} />
                  </div>
                  <span className="bar-item-label">{row.label}</span>
                  <span className="status-secondary bar-item-stats">
                    {Math.round(pct)}% ({count})
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </section>

      <section className="analytics-metrics">
        <div className="metric-card">
          <p className="metric-label">Focus Score</p>
          {focusLoading ? (
            <div className="analytics-widget-loading analytics-widget-loading--inline">
              <span className="analytics-loading-dots">Loading</span>
            </div>
          ) : (
            <>
              <p className="metric-value">{focusDisplayScore !== null ? `${focusDisplayScore} / 100` : "—"}</p>
              <p className="status-secondary">A higher score means more sustained focused windows.</p>
            </>
          )}
        </div>
        <div className="metric-card">
          <p className="metric-label">Corrections Rate</p>
          {retrainingLoading ? (
            <div className="analytics-widget-loading analytics-widget-loading--inline">
              <span className="analytics-loading-dots">Loading</span>
            </div>
          ) : (
            <>
              <p className="metric-value">{correctionPercent !== null ? `${correctionPercent}%` : "—"}</p>
              <p className="status-secondary">Predictions manually corrected by user feedback.</p>
            </>
          )}
        </div>
      </section>
    </div>
  );
};
