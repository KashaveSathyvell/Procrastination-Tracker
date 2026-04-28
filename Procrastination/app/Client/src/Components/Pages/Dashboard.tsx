import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import { StartStopButton } from "../StartStopButton";
import "./Pages.css";

type PredictionPayload = {
  prediction_id: number;
  feature_vector_id: number;
  prediction_label: "Focused" | "At Risk" | "Procrastinating" | "Idle";
  confidence: number;
  timestamp: number;
};

const getStateClass = (label: string) => {
  if (label === "Focused") return "state-focused";
  if (label === "At Risk") return "state-at-risk";
  if (label === "Procrastinating") return "state-procrastinating";
  return "state-idle";
};

const formatTime = (timestamp: number) =>
  new Date(timestamp).toLocaleTimeString("en-GB", { hour12: false });

export const Dashboard = () => {
  const [predictions, setPredictions] = useState<PredictionPayload[]>([]);
  const [pulseDot, setPulseDot] = useState(false);

  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen<PredictionPayload>("new_prediction", (event) => {
        setPredictions((prev) => [event.payload, ...prev].slice(0, 10));
        setPulseDot(true);
        setTimeout(() => setPulseDot(false), 300);
      });
      return unlisten;
    };

    const unlistenFn = setupListener();
    return () => {
      unlistenFn.then((fn) => fn());
    };
  }, []);

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
              className="confidence-fill"
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
          <span className={`pill ${predictions.length > 0 ? "state-focused" : "state-idle"}`}>
            {predictions.length > 0 ? "Active" : "Inactive"}
          </span>
        </div>
        <StartStopButton />
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
          <p className="metric-value">{predictions.length}</p>
        </div>
      </section>
    </div>
  );
};
