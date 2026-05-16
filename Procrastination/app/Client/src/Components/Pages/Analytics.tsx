import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import {
    LineChart, Line, XAxis, YAxis, Tooltip,
    ResponsiveContainer, CartesianGrid
} from "recharts";
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

const STATE_COLOR: Record<string, string> = {
    "Focused": "var(--success)",
    "At Risk": "var(--warning)",
    "Procrastinating": "var(--danger)",
    "Idle": "var(--text-muted)",
};


const rangeKeyToDays: Record<RangeKey, number> = {
    today: 1,
    week: 7,
    month: 30,
};

const DISTRIBUTION_ROWS: {
    label: string;
    pctField: keyof Pick<AnalyticsStats, "focused" | "at_risk" | "procrastinating" | "idle">;
    countField: keyof Pick<AnalyticsStats, "focused_count" | "at_risk_count" | "procrastinating_count" | "idle_count">;
    className: string;
}[] = [
    { label: "Focused", pctField: "focused", countField: "focused_count", className: "state-focused" },
    { label: "At Risk", pctField: "at_risk", countField: "at_risk_count", className: "state-at-risk" },
    { label: "Procrastinating", pctField: "procrastinating", countField: "procrastinating_count", className: "state-procrastinating" },
    { label: "Idle", pctField: "idle", countField: "idle_count", className: "state-idle" },
];

export const Analytics = () => {
    const [range, setRange] = useState<RangeKey>("today");
    const rangeDays = rangeKeyToDays[range];

    const [stats, setStats] = useState<AnalyticsStats | null>(null);
    const [statsLoading, setStatsLoading] = useState(true);
    const [statsError, setStatsError] = useState<string | null>(null);

    const [focusPayload, setFocusPayload] = useState<AnalyticsFocusScore | null>(null);
    const [focusLoading, setFocusLoading] = useState(true);
    const [focusError, setFocusError] = useState<string | null>(null);

    const [retraining, setRetraining] = useState<RetrainingCheck | null>(null);
    const [retrainingLoading, setRetrainingLoading] = useState(true);

    const [timeline, setTimeline] = useState<[number, string, number][]>([]);
    const [timelineLoading, setTimelineLoading] = useState(true);

  
    const [yesterdayFocus, setYesterdayFocus] = useState<AnalyticsFocusScore | null>(null);
    

    useEffect(() => {
        let cancelled = false;
        setStatsLoading(true);
        setStatsError(null);
        invoke<AnalyticsStats>("get_analytics_stats", { rangeDays })
            .then((data) => { if (!cancelled) setStats(data); })
            .catch((err) => { if (!cancelled) { setStats(null); setStatsError(String(err)); } })
            .finally(() => { if (!cancelled) setStatsLoading(false); });
        return () => { cancelled = true; };
    }, [rangeDays]);

    useEffect(() => {
        let cancelled = false;
        setFocusLoading(true);
        setFocusError(null);
        invoke<AnalyticsFocusScore>("get_analytics_focus_score", { rangeDays })
            .then((data) => { if (!cancelled) setFocusPayload(data); })
            .catch((err) => { if (!cancelled) { setFocusPayload(null); setFocusError(String(err)); } })
            .finally(() => { if (!cancelled) setFocusLoading(false); });
        return () => { cancelled = true; };
    }, [rangeDays]);

    useEffect(() => {
        let cancelled = false;
        setRetrainingLoading(true);
        invoke<RetrainingCheck>("check_retraining_needed")
            .then((data) => { if (!cancelled) setRetraining(data); })
            .catch(() => { if (!cancelled) setRetraining(null); })
            .finally(() => { if (!cancelled) setRetrainingLoading(false); });
        return () => { cancelled = true; };
    }, [rangeDays]);

    useEffect(() => {
      let cancelled = false;
      setTimelineLoading(true);
      invoke<[number, string, number][]>("get_state_timeline", { rangeDays })
          .then((data) => { if (!cancelled) setTimeline(data); })
          .catch(() => { if (!cancelled) setTimeline([]); })
          .finally(() => { if (!cancelled) setTimelineLoading(false); });
      return () => { cancelled = true; };
    }, [rangeDays]);

    // fetch yesterday's focus for comparison message — only relevant for "today" view
    useEffect(() => {
        if (range !== "today") { setYesterdayFocus(null); return; }
        invoke<AnalyticsFocusScore>("get_analytics_focus_score", { rangeDays: 2 })
            .then((data) => setYesterdayFocus(data))
            .catch(() => setYesterdayFocus(null));
    }, [range]);

    const correctionPercent = useMemo(() => {
        if (!retraining) return null;
        return Math.round(Math.min(1, Math.max(0, retraining.correction_rate)) * 100);
    }, [retraining]);

    const focusDisplayScore = useMemo(() => {
        if (!focusPayload) return null;
        return Math.round(Math.min(100, Math.max(0, focusPayload.score)));
    }, [focusPayload]);
    
    const chartData = useMemo(() => {
      const byBucket: Record<number, Record<string, number>> = {};
      for (const [bucket, state, pct] of timeline) {
          if (!byBucket[bucket]) byBucket[bucket] = {};
          byBucket[bucket][state] = pct;
      }
      return Object.entries(byBucket)
          .sort(([a], [b]) => Number(a) - Number(b))
          .map(([bucket, states]) => ({
              bucket: Number(bucket),
              Focused: states["Focused"] ?? 0,
              "At Risk": states["At Risk"] ?? 0,
              Procrastinating: states["Procrastinating"] ?? 0,
              Idle: states["Idle"] ?? 0,
          }));
    }, [timeline]);

    // comparison message between today and yesterday
    const comparisonMessage = useMemo(() => {
        if (range !== "today") return null;
        if (!focusPayload || !yesterdayFocus) return null;
        const todayPct = Math.round(focusPayload.focused_percentage);
        const yesterdayPct = Math.round(yesterdayFocus.focused_percentage);
        const diff = todayPct - yesterdayPct;
        if (Math.abs(diff) < 3) return null;
        if (diff > 0) {
            return {
                text: `You've been ${diff}% more focused today than yesterday. Keep it going!`,
                positive: true,
            };
        } else {
            return {
                text: `Focus is down ${Math.abs(diff)}% compared to yesterday — you've got this, every session is a fresh start.`,
                positive: false,
            };
        }
    }, [focusPayload, yesterdayFocus, range]);

    const weekComparisonMessage = useMemo(() => {
      if (range === "today") return null;
      if (chartData.length < 2) return null;

      const mid = Math.floor(chartData.length / 2);
      const firstHalf = chartData.slice(0, mid);
      const secondHalf = chartData.slice(mid);

      const avgFocused = (data: typeof chartData) =>
          data.reduce((sum, d) => sum + d.Focused, 0) / data.length;

      const firstAvg = avgFocused(firstHalf);
      const secondAvg = avgFocused(secondHalf);
      const diff = Math.round(secondAvg - firstAvg);

      if (Math.abs(diff) < 3) return null;

      const periodLabel = range === "week" ? "this week" : "this month";

      if (diff > 0) {
          return {
              text: `Your focus improved by ${diff}% in the second half of ${periodLabel}. You're building momentum!`,
              positive: true,
          };
      } else {
          return {
              text: `Focus dipped ${Math.abs(diff)}% toward the end of ${periodLabel} — rest up and come back strong.`,
              positive: false,
          };
      }
    }, [chartData, range]);

    

    // format X axis label based on range
    const formatXAxis = (bucket: number) => {
        const date = new Date(bucket * 1000);
        if (rangeDays <= 1) {
            return `${String(date.getHours()).padStart(2, "0")}:00`;
        }
        return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
    };

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

            {/* comparison banner */}
            {(comparisonMessage ?? weekComparisonMessage) && (
              <div className={`comparison-banner ${(comparisonMessage ?? weekComparisonMessage)!.positive ? "comparison-banner--positive" : "comparison-banner--neutral"}`}>
                  <span>{(comparisonMessage ?? weekComparisonMessage)!.positive ? "🎯" : "💪"}</span>
                  <p>{(comparisonMessage ?? weekComparisonMessage)!.text}</p>
              </div>
            )}

            <section className="card">
              <h3 className="section-title">Focus State Timeline</h3>
              {timelineLoading ? (
                  <div className="analytics-widget-loading">
                      <span className="analytics-loading-dots">Loading</span>
                  </div>
              ) : chartData.length === 0 ? (
                  <p className="analytics-empty-message status-secondary">
                      No data for this period — start a monitoring session to see your timeline.
                  </p>
              ) : (
                  <>
                      <div className="chart-legend">
                          {Object.entries(STATE_COLOR).map(([state, color]) => (
                              <span key={state} className="chart-legend-item">
                                  <span className="chart-legend-dot" style={{ background: color }} />
                                  {state}
                              </span>
                          ))}
                      </div>
                      <ResponsiveContainer width="100%" height={220}>
                          <LineChart data={chartData} margin={{ top: 8, right: 8, left: -10, bottom: 0 }}>
                              <CartesianGrid stroke="var(--border)" strokeDasharray="3 3" vertical={false} />
                              <XAxis
                                  dataKey="bucket"
                                  tickFormatter={formatXAxis}
                                  tick={{ fontSize: 11, fill: "var(--text-secondary)" }}
                                  axisLine={false}
                                  tickLine={false}
                              />
                              <YAxis
                                  domain={[0, 100]}
                                  tickFormatter={(v) => `${v}%`}
                                  tick={{ fontSize: 10, fill: "var(--text-secondary)" }}
                                  axisLine={false}
                                  tickLine={false}
                                  width={40}
                              />
                              <Tooltip
                                  formatter={(value: any, name: any) => [`${Math.round(Number(value) || 0)}%`, name]}
                                  contentStyle={{
                                      background: "var(--bg-surface)",
                                      border: "1px solid var(--border-card)",
                                      borderRadius: "8px",
                                      fontSize: "12px",
                                      color: "var(--text-primary)"
                                  }}
                                  labelFormatter={(bucket) => formatXAxis(bucket)}
                              />
                              <Line type="monotone" dataKey="Focused" stroke="var(--success)" strokeWidth={2} dot={false} isAnimationActive={false} />
                              <Line type="monotone" dataKey="At Risk" stroke="var(--warning)" strokeWidth={2} dot={false} isAnimationActive={false} />
                              <Line type="monotone" dataKey="Procrastinating" stroke="var(--danger)" strokeWidth={2} dot={false} isAnimationActive={false} />
                              <Line type="monotone" dataKey="Idle" stroke="var(--text-muted)" strokeWidth={2} dot={false} isAnimationActive={false} />
                          </LineChart>
                      </ResponsiveContainer>
                  </>
              )}
            </section>

            <section className="card">
                <h3 className="section-title">State Distribution</h3>
                {statsLoading ? (
                    <div className="analytics-widget-loading" aria-hidden="true">
                        <span className="analytics-loading-dots">Loading</span>
                    </div>
                ) : statsError ? (
                    <p className="analytics-empty-message" style={{ color: "var(--danger)" }}>
                        Failed to load: {statsError}
                    </p>
                ) : stats && stats.total === 0 ? (
                    <p className="analytics-empty-message status-secondary">
                        No data for this period.
                    </p>
                ) : (
                    <div className="bar-chart">
                        {DISTRIBUTION_ROWS.map((row) => {
                            const pct = Math.min(100, Math.max(0, stats![row.pctField]));
                            const count = stats![row.countField];
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
                    ) : focusError ? (
                        <p style={{ color: "var(--danger)" }}>Failed to load</p>
                    ) : (
                        <>
                            <p className="metric-value">{focusDisplayScore !== null ? `${focusDisplayScore} / 100` : "—"}</p>
                            <p className="status-secondary">A higher score means more sustained focused windows.</p>
                        </>
                    )}
                </div>
                <div className="metric-card">
                    <p className="metric-label">Corrections Since Last Training</p>
                    {retrainingLoading ? (
                        <div className="analytics-widget-loading analytics-widget-loading--inline">
                            <span className="analytics-loading-dots">Loading</span>
                        </div>
                    ) : (
                        <>
                            <p className="metric-value">{correctionPercent !== null ? `${correctionPercent}%` : "—"}</p>
                            <p className="status-secondary">Based on predictions since last model update</p>
                        </>
                    )}
                </div>
            </section>
        </div>
    );
};