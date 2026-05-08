import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from 'react';
import './IdlePopUp.css';

type stateConfirmation = {
    timestamp: number,
    streakWindows: number,
    label: string,
    overwrite: boolean,
}

type IdleCheckPayload = {
    timestamp: number,
    streakWindows: number,
    label: string,
}

const LABELS = ["Focused", "At Risk", "Procrastinating", "Idle"];

export const IdlePopUp = () => {
    const [payload, setPayload] = useState<IdleCheckPayload | null>(null);
    const [isVisible, setIsVisible] = useState(false);
    const [selectedLabel, setSelectedLabel] = useState<string>("Idle");

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<IdleCheckPayload>('idle_check', (event) => {
                setPayload(event.payload);
                setSelectedLabel(event.payload.label);
                setIsVisible(true);
            });
            return unlisten;
        };

        const unlistenFn = setupListener();
        return () => { unlistenFn.then(fn => fn()); };
    }, []);

    const handleConfirm = async (overwrite: boolean) => {
        if (!payload) return;
        setIsVisible(false);
        setPayload(null);
        try {
            await invoke('update_label_streak', {
                stateConfirmation: {
                    timestamp: payload.timestamp,
                    streakWindows: payload.streakWindows,
                    label: selectedLabel,
                    overwrite,
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

    const handleDismiss = async () => {
        if (!payload) return;
        setIsVisible(false);
        setPayload(null);
        try {
            await invoke('update_label_streak', {
                stateConfirmation: {
                    timestamp: payload.timestamp,
                    streakWindows: payload.streakWindows,
                    label: "Idle",
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

    if (!isVisible || !payload) return null;

    return (
        <div className="idle-popup-overlay">
            <div className="idle-popup-box">
                <button className="idle-popup-close" onClick={handleDismiss}>✕</button>

                <p className="idle-popup-header">Idle check</p>

                <div className="idle-popup-meta">
                    <span className="idle-popup-badge">Idle</span>
                    <span className="idle-popup-duration">{payload.streakWindows} mins</span>
                </div>

                <p className="idle-popup-message">
                    You seem idle. Still working? Let us know what you're up to.
                </p>

                <div className="idle-popup-labels">
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

                <div className="idle-popup-actions">
                    <button
                        className="btn-idle-confirm"
                        onClick={() => handleConfirm(selectedLabel !== payload.label)}
                    >
                        Confirm
                    </button>
                    <button className="btn-idle-dismiss" onClick={handleDismiss}>Dismiss</button>
                </div>
            </div>
        </div>
    );
};