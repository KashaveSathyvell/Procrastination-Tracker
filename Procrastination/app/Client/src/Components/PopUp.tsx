import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from 'react';
import React from 'react';
import './PopUp.css';

type InterventionPackage = {
    intervention_id: number,
    timestamp: number,
    intervention_type: string,
    prediction_label: string,
    confidence: number,
}

const LABELS = ["Focused", "At Risk", "Procrastinating", "Idle"];

export const PopUp = () => {
    const [intervention, setIntervention] = useState<InterventionPackage | null>(null);
    const [isVisible, setIsVisible] = useState(false);
    const [selectedLabel, setSelectedLabel] = useState<string>("");

    useEffect(() => {
        const setupListener = async () => {
            const unlisten = await listen<InterventionPackage>('new_intervention', (event) => {
                setIntervention(event.payload);
                setSelectedLabel(event.payload.prediction_label);
                setIsVisible(true);
            });
            return unlisten;
        };

        const unlistenFn = setupListener();
        return () => { unlistenFn.then(fn => fn()); };
    }, []);

    const handleConfirm = async () => {
        if (!intervention) return;
        await invoke('respond_to_intervention', {
            interventionId: intervention.intervention_id,
            userLabel: selectedLabel,
            dismissed: false,
        });
        setIsVisible(false);
        setIntervention(null);
    };

    const handleDismiss = async () => {
        if (!intervention) return;
        await invoke('respond_to_intervention', {
            interventionId: intervention.intervention_id,
            userLabel: null,
            dismissed: true,
        });
        setIsVisible(false);
        setIntervention(null);
    };

    if (!isVisible || !intervention) return null;

    return (
        <div className="popup-overlay">
            <div className="popup-box">
                <button className="popup-close" onClick={handleDismiss}>✕</button>

                <p className="popup-header">Procrastination alert</p>

                <div className="popup-meta">
                    <span className="popup-badge">{intervention.prediction_label}</span>
                    <span className="popup-confidence">{(intervention.confidence * 100).toFixed(0)}% confidence</span>
                </div>

                <p className="popup-prompt">Is this accurate? Pick your current state or dismiss.</p>

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
                    <button className="btn-confirm" onClick={handleConfirm}>Confirm</button>
                    <button className="btn-dismiss" onClick={handleDismiss}>Dismiss</button>
                </div>
            </div>
        </div>
    );
};