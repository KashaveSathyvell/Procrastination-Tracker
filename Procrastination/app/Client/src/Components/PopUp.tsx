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

export const PopUp = () => {
    const [intervention, setIntervention] = useState<InterventionPackage | null>(null);
    const [isVisible, setIsVisible] = useState(false);
    const [showCorrection, setShowCorrection] = useState(false);
    const [selectedLabel, setSelectedLabel] = useState<string>("");

    useEffect(() => {
        // Apply saved theme on mount
        const savedTheme = localStorage.getItem("theme") ?? "dark";
        document.documentElement.setAttribute("data-theme", savedTheme);

        // Listen for theme changes from main window
        const setupThemeListener = async () => {
            const unlisten = await listen<{ theme: string }>("theme-changed", (event) => {
                document.documentElement.setAttribute("data-theme", event.payload.theme);
                localStorage.setItem("theme", event.payload.theme);
            });
            return unlisten;
        };

        const unlistenFn = setupThemeListener();
        return () => { unlistenFn.then(fn => fn()); };
    }, []);

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<InterventionPackage>('new_intervention', async (event) => {
                setIntervention(event.payload);
                setSelectedLabel(event.payload.prediction_label);
                setShowCorrection(false);
                setIsVisible(true);

                try {
                    const appWindow = getCurrentWindow();
                    await appWindow.setAlwaysOnTop(true);
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
            if (win.label === "main") return;
            await win.hide();
        } catch (e) {
            console.error("Failed to hide popup window:", e);
        }
    };

    const handleTakeBreak = async () => {
        if (!intervention) return;
        const interventionData = intervention;
        if (!interventionData.suggested_activity || 
            !interventionData.suggested_duration || 
            !interventionData.preference_id) {
            // No activity available, just dismiss
            await close();
            return;
        }

        await close();

        try {
            await invoke('intervention_update', {
                updatedIntervention: {
                    timestamp: interventionData.timestamp,
                    interventionId: interventionData.intervention_id,
                    userLabel: interventionData.prediction_label,
                    dismissed: false,
                    predictedLabel: interventionData.prediction_label,
                },
            });
            const sessionId = await invoke<number>('break_start', {
                interventionId: interventionData.intervention_id,
                activity: interventionData.suggested_activity,
                plannedDurationMins: interventionData.suggested_duration,
                preferenceId: interventionData.preference_id,
            });
            await invoke('open_break_window', {
                activity: interventionData.suggested_activity,
                duration: interventionData.suggested_duration,
                breakSessionId: sessionId,
                interventionId: interventionData.intervention_id,
            });
            try {
                const appWindow = getCurrentWindow();
                await appWindow.setAlwaysOnTop(false);
            } catch (e) {
                console.error("Failed to disable always-on-top after break start:", e);
            }
        } catch (e) {
            console.error('start_break failed:', e);
            setIntervention(interventionData);
            setSelectedLabel(interventionData.prediction_label);
            setShowCorrection(false);
            setIsVisible(true);
            try {
                const appWindow = getCurrentWindow();
                await appWindow.show();
                await appWindow.setAlwaysOnTop(true);
                await appWindow.setFocus();
            } catch (showError) {
                console.error("Failed to restore popup window after start break failure:", showError);
            }
        }
    };

    const handleDismiss = async () => {
        if (!intervention) return;
        const interventionData = intervention;
        close();
        try {
            await invoke('intervention_update', {
                updatedIntervention: {
                    timestamp: interventionData.timestamp,
                    interventionId: interventionData.intervention_id,
                    userLabel: interventionData.prediction_label,
                    dismissed: true,
                    predictedLabel: interventionData.prediction_label,
                },
            });
            try {
                const appWindow = getCurrentWindow();
                await appWindow.setAlwaysOnTop(false);
            } catch (e) {
                console.error("Failed to disable always-on-top after dismiss:", e);
            }
        } catch (e) {
            console.error('intervention_update failed:', e);
        }
    };

    const handleConfirmCorrection = async () => {
        if (!intervention) return;
        const interventionData = intervention;
        if (!interventionData.suggested_activity || 
            !interventionData.suggested_duration || 
            !interventionData.preference_id) {
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
                    timestamp: interventionData.timestamp,
                    interventionId: interventionData.intervention_id,
                    userLabel: selectedLabel,
                    dismissed: false,
                    predictedLabel: interventionData.prediction_label,
                },
            });
            if (isStillBadState) {
                const sessionId = await invoke<number>('break_start', {
                    interventionId: interventionData.intervention_id,
                    activity: interventionData.suggested_activity,
                    plannedDurationMins: interventionData.suggested_duration,
                    preferenceId: interventionData.preference_id,
                });
                await invoke('open_break_window', {
                    activity: interventionData.suggested_activity,
                    duration: interventionData.suggested_duration,
                    breakSessionId: sessionId,
                    interventionId: interventionData.intervention_id,
                });
                try {
                    const appWindow = getCurrentWindow();
                    await appWindow.setAlwaysOnTop(false);
                    if (appWindow.label !== "main") {
                        await appWindow.hide();
                    }
                } catch (e) {
                    console.error("Failed to disable always-on-top after break start:", e);
                }
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