import { useState } from "react";
import "./Pages.css";

type RangeKey = "today" | "week" | "month";

export const Analytics = () => {
  const [range, setRange] = useState<RangeKey>("today");

  // TODO: Replace this mock analytics data with aggregated results from a new Tauri command.
  const stateDistribution = [
    { label: "Focused", percentage: 48, className: "state-focused" },
    { label: "At Risk", percentage: 22, className: "state-at-risk" },
    { label: "Procrastinating", percentage: 18, className: "state-procrastinating" },
    { label: "Idle", percentage: 12, className: "state-idle" },
  ];

  // TODO: Replace mock focus score with computed score from backend analytics endpoint.
  const focusScore = 72;
  // TODO: Replace mock corrections rate with real correction metrics from backend.
  const correctionRate = 14;

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
        <div className="bar-chart">
          {stateDistribution.map((item) => (
            <div key={item.label} className="bar-item">
              <div className="bar-track">
                <div
                  className={`bar-fill ${item.className}`}
                  style={{ height: `${item.percentage}%` }}
                />
              </div>
              <span>{item.label}</span>
              <span className="status-secondary">{item.percentage}%</span>
            </div>
          ))}
        </div>
      </section>

      <section className="analytics-metrics">
        <div className="metric-card">
          <p className="metric-label">Focus Score</p>
          <p className="metric-value">{focusScore}/100</p>
          <p className="status-secondary">A higher score means more sustained focused windows.</p>
        </div>
        <div className="metric-card">
          <p className="metric-label">Corrections Rate</p>
          <p className="metric-value">{correctionRate}%</p>
          <p className="status-secondary">Predictions manually corrected by user feedback.</p>
        </div>
      </section>
    </div>
  );
};
