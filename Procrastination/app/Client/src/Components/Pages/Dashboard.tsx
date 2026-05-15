import { useEffect, useMemo, useState } from "react";
import { StartStopButton } from "../StartStopButton";
import type { PredictionItem } from "../../App";
import "./Pages.css";

type DashboardProps = {
  predictions: PredictionItem[];
  totalPredictions: number;
  isMonitoring: boolean;
  onMonitoringChange: (active: boolean) => void;
  showRetrainingBanner: boolean;
  onNavigateToSettings: () => void;
};

const getStateClass = (label: string) => {
  if (label === "Focused") return "state-focused";
  if (label === "At Risk") return "state-at-risk";
  if (label === "Procrastinating") return "state-procrastinating";
  return "state-idle";
};

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp);
  const now = new Date();
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);

  const dateKey = date.toDateString();
  const todayKey = now.toDateString();
  const yesterdayKey = yesterday.toDateString();
  const hh = String(date.getHours()).padStart(2, "0");
  const mm = String(date.getMinutes()).padStart(2, "0");
  const timePart = `${hh}:${mm}`;

  if (dateKey === todayKey) return `Today ${timePart}`;
  if (dateKey === yesterdayKey) return `Yesterday ${timePart}`;

  const month = date.toLocaleString("en-US", { month: "short" });
  const day = String(date.getDate()).padStart(2, "0");
  return `${month} ${day}, ${timePart}`;
};

export const Dashboard = ({ predictions, totalPredictions, isMonitoring, onMonitoringChange, showRetrainingBanner, onNavigateToSettings }: DashboardProps) => {
  const [pulseDot, setPulseDot] = useState(false);
  const [bannerDismissed, setBannerDismissed] = useState(false);

  useEffect(() => {
    if (predictions.length === 0) return;
    setPulseDot(true);
    const timer = setTimeout(() => setPulseDot(false), 300);
    return () => clearTimeout(timer);
  }, [predictions]);

  const currentPrediction = predictions[0];
  const averageConfidence = useMemo(() => {
    if (predictions.length === 0) return 0;
    const total = predictions.reduce((sum, p) => sum + p.confidence, 0);
    return Math.round((total / predictions.length) * 100);
  }, [predictions]);

  const mostFrequentState = useMemo(() => {
    if (predictions.length === 0) return "N/A";
    const counts = predictions.reduce<Record<string, number>>((acc, p) => {
      acc[p.prediction_label] = (acc[p.prediction_label] || 0) + 1;
      return acc;
    }, {});
    return Object.entries(counts).sort((a, b) => b[1] - a[1])[0][0];
  }, [predictions]);

  return (
    <div className="page-shell dashboard-page">
      {showRetrainingBanner && !bannerDismissed && (
        <div className="retrain-banner">
            <div className="retrain-banner-content">
                <span className="retrain-banner-icon">⚠</span>
                <span className="retrain-banner-text">
                    Model accuracy may be declining. Retrain when system is not in use.
                </span>
            </div>
            <div className="retrain-banner-actions">
                <button className="retrain-banner-btn" onClick={onNavigateToSettings}>
                    Go to Settings
                </button>
                <button className="retrain-banner-dismiss" onClick={() => setBannerDismissed(true)}>
                    ✕
                </button>
            </div>
        </div>
      )}
      <header>
        <h1 className="page-title">Dashboard</h1>
        <p className="page-subtitle">Live monitoring status and recent predictions.</p>
      </header>

      <section className="card status-card">
        <div className="status-heading">
          <span className={`status-dot ${currentPrediction ? getStateClass(currentPrediction.prediction_label) : "state-idle"} ${pulseDot ? "pulse" : ""}`} />
          <h2 className={`status-label ${currentPrediction ? getStateClass(currentPrediction.prediction_label) : ""}`}>
            {currentPrediction ? currentPrediction.prediction_label : "Waiting for predictions"}
          </h2>
        </div>
        <div className="confidence-wrap">
          <div className="confidence-track">
            <div
              className={`confidence-fill ${currentPrediction ? getStateClass(currentPrediction.prediction_label) : ""}`}
              style={{ width: `${Math.round((currentPrediction?.confidence || 0) * 100)}%` }}
            />  
          </div>
          <span className="status-secondary">
            Confidence: {Math.round((currentPrediction?.confidence || 0) * 100)}%
          </span>
        </div>
        <span className="status-secondary">
          Last update: {currentPrediction ? formatTime(currentPrediction.timestamp) : "--:--:--"}
        </span>
      </section>

      <section className="card monitoring-card">
        <div className="monitoring-header">
          <h3 className="section-title">Monitoring</h3>
          <span className={`pill ${isMonitoring ? "state-focused" : "state-idle"}`}>
            {isMonitoring ? "Active" : "Inactive"}
          </span>
        </div>
        <StartStopButton isMonitoring={isMonitoring} onMonitoringChange={onMonitoringChange} />
      </section>

      <section className="card">
        <h3 className="section-title">Recent Predictions</h3>
        <div className="prediction-list">
          {predictions.length === 0 ? (
            <p className="status-secondary">No predictions received yet.</p>
          ) : (
            predictions.map((prediction) => (
              <div key={prediction.prediction_id} className={`prediction-row ${getStateClass(prediction.prediction_label)}`}>
                <span>{prediction.prediction_label}</span>
                <span>{Math.round(prediction.confidence * 100)}%</span>
                <span>{formatTime(prediction.timestamp)}</span>
              </div>
            ))
          )}
        </div>
      </section>

      <section className="session-summary">
        <div className="metric-card">
          <p className="metric-label">Most Frequent State</p>
          <p className={`metric-value ${getStateClass(mostFrequentState)}`}>{mostFrequentState}</p>
        </div>
        <div className="metric-card">
          <p className="metric-label">Average Confidence</p>
          <p className="metric-value">{averageConfidence}%</p>
        </div>
        <div className="metric-card">
          <p className="metric-label">Total Predictions</p>
          <p className="metric-value">{totalPredictions}</p>
        </div>
      </section>
    </div>
  );
};
