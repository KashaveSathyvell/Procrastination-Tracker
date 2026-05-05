// PopUp.tsx
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from 'react';
import './PopUp.css';

import { getCurrentWindow } from "@tauri-apps/api/window";

type InterventionPackage = {
    intervention_id: number,
    timestamp: number,
    intervention_type: string,
    prediction_label: string,
    confidence: number,
    suggested_activity: string | null,
    suggested_duration: number | null,
    preference_id: number | null,
}

export type BreakData = {
    activity: string | null,
    duration: number | null,
    preference_id: number | null,
    intervention_id: number,
    break_session_id: number,
}

const LABELS = ["Focused", "At Risk", "Procrastinating", "Idle"];

type PopUpProps = {
    onBreakStart: (data: BreakData) => void,
}

export const PopUp = ({ onBreakStart }: PopUpProps) => {
    const [intervention, setIntervention] = useState<InterventionPackage | null>(null);
    const [isVisible, setIsVisible] = useState(false);
    const [showCorrection, setShowCorrection] = useState(false);
    const [selectedLabel, setSelectedLabel] = useState<string>("");

    // const appWindow = getCurrentWindow();

    // await appWindow.setAlwaysOnTop(true);
    // await appWindow.setFocus();

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<InterventionPackage>('new_intervention', async (event) => {
                setIntervention(event.payload);
                setSelectedLabel(event.payload.prediction_label);
                setShowCorrection(false);
                setIsVisible(true);

                try {
                    const appWindow = getCurrentWindow();
                    await appWindow.setFocus();
                } catch (e) {
                    console.error("Failed to set window to top:", e);
                }
            });
            return unlisten;
        };

        const unlistenFn = setupListener();
        return () => { unlistenFn.then(fn => fn()); };
    }, []);

    const close = async () => {
        setIsVisible(false);
        setIntervention(null);
        setShowCorrection(false);

        try {
            const { getCurrentWindow } = await import("@tauri-apps/api/window");
            const win = getCurrentWindow();
            await win.hide();
        } catch (e) {
            console.error("Failed to hide popup window:", e);
        }
    };

    const handleTakeBreak = async () => {
        if (!intervention) return;
        if (!intervention.suggested_activity || 
            !intervention.suggested_duration || 
            !intervention.preference_id) {
            // No activity available, just dismiss
            setIsVisible(false);
            return;
        }

        setIsVisible(false);
        setIntervention(null);
        setShowCorrection(false);

        try {
            await invoke('intervention_update', {
                updatedIntervention: {
                    timestamp: intervention.timestamp,
                    interventionId: intervention.intervention_id,
                    userLabel: intervention.prediction_label,
                    dismissed: false,
                    predictedLabel: intervention.prediction_label,
                },
            });
            const sessionId = await invoke<number>('break_start', {
                interventionId: intervention.intervention_id,
                activity: intervention.suggested_activity,
                plannedDurationMins: intervention.suggested_duration,
                preferenceId: intervention.preference_id,
            });
            onBreakStart({
                activity: intervention.suggested_activity,
                duration: intervention.suggested_duration,
                preference_id: intervention.preference_id,
                intervention_id: intervention.intervention_id,
                break_session_id: sessionId,
            });
        } catch (e) {
            console.error('start_break failed:', e);
        }
    };

    const handleDismiss = async () => {
        if (!intervention) return;
        close();
        try {
            await invoke('intervention_update', {
                updatedIntervention: {
                    timestamp: intervention.timestamp,
                    interventionId: intervention.intervention_id,
                    userLabel: intervention.prediction_label,
                    dismissed: true,
                    predictedLabel: intervention.prediction_label,
                },
            });
        } catch (e) {
            console.error('intervention_update failed:', e);
        }
    };

    const handleConfirmCorrection = async () => {
        if (!intervention) return;
        if (!intervention.suggested_activity || 
            !intervention.suggested_duration || 
            !intervention.preference_id) {
            // No activity available, just dismiss
            setIsVisible(false);
            return;
        }
        // If user corrects to a still-bad state, we still suggest a break
        const isStillBadState = selectedLabel === "At Risk" || selectedLabel === "Procrastinating";
        close();
        try {
            await invoke('intervention_update', {
                updatedIntervention: {
                    timestamp: intervention.timestamp,
                    interventionId: intervention.intervention_id,
                    userLabel: selectedLabel,
                    dismissed: false,
                    predictedLabel: intervention.prediction_label,
                },
            });
            if (isStillBadState) {
                const sessionId = await invoke<number>('break_start', {
                    interventionId: intervention.intervention_id,
                    activity: intervention.suggested_activity,
                    plannedDurationMins: intervention.suggested_duration,
                    preferenceId: intervention.preference_id,
                });
                onBreakStart({
                    activity: intervention.suggested_activity,
                    duration: intervention.suggested_duration,
                    preference_id: intervention.preference_id,
                    intervention_id: intervention.intervention_id,
                    break_session_id: sessionId,
                });
            }
        } catch (e) {
            console.error('correction failed:', e);
        }
    };

    if (!isVisible || !intervention) return null;

    return (
        <div className="popup-overlay">
            <div className="popup-box">
                <button className="popup-close" onClick={handleDismiss}>✕</button>

                {!showCorrection ? (
                    <>
                        <p className="popup-header">Time for a break?</p>

                        <div className="popup-meta">
                            <span className="popup-badge">{intervention.prediction_label}</span>
                            <span className="popup-confidence">{(intervention.confidence * 100).toFixed(0)}% confidence</span>
                        </div>

                        <div className="break-suggestion">
                            <p className="break-activity">{intervention.suggested_activity}</p>
                            <p className="break-duration">{intervention.suggested_duration} min</p>
                        </div>

                        <div className="popup-actions-col">
                            <button className="btn-take-break" onClick={handleTakeBreak}>
                                I'll take the break
                            </button>
                            <button className="btn-wrong-state" onClick={() => setShowCorrection(true)}>
                                Wrong state
                            </button>
                            <button className="btn-dismiss" onClick={handleDismiss}>
                                Dismiss
                            </button>
                        </div>
                    </>
                ) : (
                    <>
                        <button className="popup-back" onClick={() => setShowCorrection(false)}>← Back</button>

                        <p className="popup-header">Correct your state</p>

                        <div className="popup-labels">
                            {LABELS.map(label => (
                                <button
                                    key={label}
                                    className={`label-btn ${selectedLabel === label ? 'selected' : ''}`}
                                    onClick={() => setSelectedLabel(label)}
                                >
                                    {label}
                                </button>
                            ))}
                        </div>

                        <div className="popup-actions">
                            <button className="btn-confirm" onClick={handleConfirmCorrection}>Confirm</button>
                            <button className="btn-dismiss" onClick={handleDismiss}>Dismiss</button>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
};