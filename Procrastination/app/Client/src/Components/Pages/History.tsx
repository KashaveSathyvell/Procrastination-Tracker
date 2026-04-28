import { useState } from "react";
import "./Pages.css";

type FilterKey = "All" | "Focused" | "At Risk" | "Procrastinating" | "Idle";

const getStateClass = (label: string) => {
  if (label === "Focused") return "state-focused";
  if (label === "At Risk") return "state-at-risk";
  if (label === "Procrastinating") return "state-procrastinating";
  return "state-idle";
};

export const History = () => {
  const [filter, setFilter] = useState<FilterKey>("All");

  // TODO: Replace mock history entries with paginated prediction history from a future Tauri command.
  const historyRows = [
    { timestamp: "14:01:22", state: "Focused", confidence: 91, corrected: "No" },
    { timestamp: "13:58:10", state: "At Risk", confidence: 74, corrected: "Yes" },
    { timestamp: "13:51:43", state: "Procrastinating", confidence: 82, corrected: "No" },
    { timestamp: "13:43:07", state: "Idle", confidence: 69, corrected: "No" },
    { timestamp: "13:39:55", state: "Focused", confidence: 88, corrected: "No" },
    { timestamp: "13:31:01", state: "At Risk", confidence: 70, corrected: "Yes" },
    { timestamp: "13:25:16", state: "Focused", confidence: 86, corrected: "No" },
  ];

  return (
    <div className="page-shell">
      <header>
        <h1 className="page-title">History</h1>
        <p className="page-subtitle">Recent prediction events and correction status.</p>
      </header>

      <section className="card filter-row">
        {(["All", "Focused", "At Risk", "Procrastinating", "Idle"] as FilterKey[]).map((item) => (
          <button
            key={item}
            className={`pill-toggle ${filter === item ? "active" : ""}`}
            onClick={() => setFilter(item)}
          >
            {item}
          </button>
        ))}
      </section>

      <section className="card history-card">
        <div className="history-header-row">
          <span>Timestamp</span>
          <span>Predicted State</span>
          <span>Confidence</span>
          <span>Corrected</span>
        </div>
        <div className="history-list">
          {historyRows.map((row) => (
            <div key={`${row.timestamp}-${row.state}`} className="history-row">
              <span>{row.timestamp}</span>
              <span className={`badge ${getStateClass(row.state)}`}>{row.state}</span>
              <span>{row.confidence}%</span>
              <span className={`badge ${row.corrected === "Yes" ? "state-at-risk" : "state-idle"}`}>{row.corrected}</span>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
};
