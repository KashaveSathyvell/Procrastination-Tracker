import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import "./Pages.css";

type FilterKey = "All" | "Focused" | "At Risk" | "Procrastinating" | "Idle";
type RangeKey = "today" | "week" | "month";

type HistoryRowPayload = {
  prediction_id: number;
  timestamp: number;
  predicted_state: string;
  confidence: number;
  was_corrected: boolean;
  user_label: string | null;
};

const getStateClass = (label: string) => {
  if (label === "Focused") return "state-focused";
  if (label === "At Risk") return "state-at-risk";
  if (label === "Procrastinating") return "state-procrastinating";
  return "state-idle";
};

const rangeKeyToDays: Record<RangeKey, number> = {
  today: 1,
  week: 7,
  month: 30,
};

const formatTimestamp = (unixSeconds: number) => {
  const d = new Date(unixSeconds * 1000);
  const dd = String(d.getDate()).padStart(2, "0");
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const yyyy = d.getFullYear();
  const hh = String(d.getHours()).padStart(2, "0");
  const min = String(d.getMinutes()).padStart(2, "0");
  return `${dd}/${mm}/${yyyy} ${hh}:${min}`;
};

export const History = () => {
  const [filter, setFilter] = useState<FilterKey>("All");
  const [range, setRange] = useState<RangeKey>("today");
  const rangeDays = rangeKeyToDays[range];

  const [rows, setRows] = useState<HistoryRowPayload[]>([]);
  const [loading, setLoading] = useState(true);

  const stateFilter = useMemo(() => (filter === "All" ? null : filter), [filter]);

  useEffect(() => {
    let cancelled = false;

    setLoading(true);
    invoke<HistoryRowPayload[]>("get_history", { rangeDays, stateFilter })
      .then((data) => {
        if (!cancelled) setRows(Array.isArray(data) ? data.slice(0, 100) : []);
      })
      .catch(() => {
        if (!cancelled) setRows([]);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [rangeDays, stateFilter]);

  return (
    <div className="page-shell">
      <header>
        <h1 className="page-title">History</h1>
        <p className="page-subtitle">Recent prediction events and correction status.</p>
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
          <span>Correction</span>
        </div>
        {loading ? (
          <div className="history-loading" aria-hidden="true">
            <span className="analytics-loading-dots">Loading</span>
          </div>
        ) : rows.length === 0 ? (
          <p className="history-empty-message status-secondary">No predictions found for this period.</p>
        ) : (
          <div className="history-list">
            {rows.map((row) => (
              <div key={row.prediction_id} className="history-row">
                <span>{formatTimestamp(row.timestamp)}</span>
                <span className={`badge ${getStateClass(row.predicted_state)}`}>{row.predicted_state}</span>
                <span>{Math.round(Math.min(1, Math.max(0, row.confidence)) * 100)}%</span>
                <div className="history-feedback-cell">
                  <span
                    className={`history-correction-badge ${row.was_corrected ? "history-correction-badge--warn" : "history-correction-badge--ok"}`}
                  >
                    {row.was_corrected ? "Corrected" : "Accurate"}
                  </span>
                  {row.user_label != null && row.user_label !== "" && (
                    <span className="history-user-label-pill">→ {row.user_label}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
};
