import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from 'react';
import './FocusPopUp.css';

type stateConfirmation = {
    timestamp: number,
    streakWindows: number,
    label: string,
    overwrite: boolean,
}

type FocusStreakPayload = {
    timestamp: number,
    streakWindows: number,
    label: string,
}

const LABELS = ["Focused", "At Risk", "Procrastinating", "Idle"];

export const FocusPopUp = () => {
    const [payload, setPayload] = useState<FocusStreakPayload | null>(null);
    const [isVisible, setIsVisible] = useState(false);
    const [showCorrection, setShowCorrection] = useState(false);
    const [selectedLabel, setSelectedLabel] = useState<string>("Focused");

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<FocusStreakPayload>('focus_check', (event) => {
                setPayload(event.payload);
                setSelectedLabel(event.payload.label);
                setShowCorrection(false);
                setIsVisible(true);
            });
            return unlisten;
        };

        const unlistenFn = setupListener();
        return () => { unlistenFn.then(fn => fn()); };
    }, []);

    const handleConfirm = async () => {
        if (!payload) return;
        setIsVisible(false);
        setPayload(null);
        try {
            await invoke('update_label_streak', {
                stateConfirmation: {
                    timestamp: payload.timestamp,
                    streakWindows: payload.streakWindows,
                    label: payload.label,
                    overwrite: false,
                } as stateConfirmation
            });
        } catch (e) {
            console.error('update_label_streak failed:', e);
        }
        try {
            const { getCurrentWindow } = await import("@tauri-apps/api/window");
            const win = getCurrentWindow();
            if (win.label !== "main") await win.hide();
        } catch (e) {
            console.error("Failed to hide popup window:", e);
        }
    };

    // const handleCorrection = async () => {
    //     if (!payload) return;
    //     setIsVisible(false);
    //     setPayload(null);
    //     try {
    //         await invoke('update_label_streak', {
    //             stateConfirmation: {
    //                 timestamp: payload.timestamp,
    //                 streakWindows: payload.streakWindows,
    //                 label: selectedLabel,
    //                 overwrite: true,
    //             } as stateConfirmation
    //         });
    //     } catch (e) {
    //         console.error('update_label_streak failed:', e);
    //     }
    //     try {
    //         const { getCurrentWindow } = await import("@tauri-apps/api/window");
    //         const win = getCurrentWindow();
    //         if (win.label !== "main") await win.hide();
    //     } catch (e) {
    //         console.error("Failed to hide popup window:", e);
    //     }
    // };

    const handleCorrection = async () => {
        if (!payload) return;
        setIsVisible(false);
        setPayload(null);

        try {
            await invoke('update_label_streak', {
                stateConfirmation: {
                    timestamp: payload.timestamp,
                    streakWindows: payload.streakWindows,
                    label: selectedLabel,
                    overwrite: true,
                } as stateConfirmation
            });
        } catch (e) {
            console.error('update_label_streak failed:', e);
        }

        // If user corrects to a bad state, surface the intervention popup
        if (selectedLabel === 'Procrastinating' || selectedLabel === 'At Risk') {
            try {
                await invoke('trigger_manual_intervention', {
                    label: selectedLabel,
                    timestamp: payload.timestamp,
                });
            } catch (e) {
                console.error('Failed to trigger intervention after correction:', e);
            }
        }

        try {
            const { getCurrentWindow } = await import("@tauri-apps/api/window");
            const win = getCurrentWindow();
            if (win.label !== "main") await win.hide();
        } catch (e) {
            console.error("Failed to hide popup window:", e);
        }
    };

    

    if (!isVisible || !payload) return null;

    return (
        <div className="focus-popup-overlay">
            <div className="focus-popup-box">
                <p className="focus-popup-header">Focus streak</p>

                <div className="focus-popup-meta">
                    <span className="focus-popup-badge">Focused</span>
                    <span className="focus-popup-duration">{payload.streakWindows} mins</span>
                </div>

                <p className="focus-popup-message">
                    Great work! You've been focused for {payload.streakWindows} minutes straight. Keep it up!
                </p>

                {!showCorrection ? (
                    <div className="focus-popup-actions">
                        <button className="btn-thanks" onClick={handleConfirm}>Thanks!</button>
                        <button className="btn-actually" onClick={() => setShowCorrection(true)}>Actually...</button>
                    </div>
                ) : (
                    <>
                        <p className="focus-popup-correction-prompt">What were you actually doing?</p>
                        <div className="focus-popup-labels">
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
                        <div className="focus-popup-actions">
                            <button className="btn-thanks" onClick={handleCorrection}>Confirm</button>
                            <button className="btn-actually" onClick={() => setShowCorrection(false)}>Back</button>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
};