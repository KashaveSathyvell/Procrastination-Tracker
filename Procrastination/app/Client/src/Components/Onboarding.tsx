import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import React from "react";
import "./Onboarding.css";

type OnboardingProps = {
    onComplete: () => void;
};

type Screen = "select" | "success";

const ACTIVITY_ICONS: Record<string, string> = {
    Walking: "🚶",
    "Light Exercise": "🏃",
    Gaming: "🎮",
    Reading: "📖",
    Meditation: "🧘",
    Stretching: "🤸",
};

export const Onboarding = ({ onComplete }: OnboardingProps) => {
    const [activities, setActivities] = useState<string[]>([]);
    const [selected, setSelected] = useState<Set<string>>(new Set());
    const [screen, setScreen] = useState<Screen>("select");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        invoke<string[]>("get_preference")
            .then(setActivities)
            .catch(() => setError("Failed to load activities."));
    }, []);

    const toggle = (activity: string) => {
        setSelected((prev) => {
            const next = new Set(prev);
            next.has(activity) ? next.delete(activity) : next.add(activity);
            return next;
        });
    };

    const handleContinue = async () => {
        if (selected.size === 0) return;
        setLoading(true);
        setError(null);
        try {
            await invoke("save_user_activity", { chosenList: Array.from(selected) });
            setScreen("success");
        } catch (e) {
            setError("Something went wrong. Please try again.");
        } finally {
            setLoading(false);
        }
    };

    if (screen === "success") {
        return (
            <div className="ob-root">
                <div className="ob-card ob-success-card">
                    <div className="ob-success-icon">✓</div>
                    <h1 className="ob-success-title">You're all set</h1>
                    <p className="ob-success-sub">
                        {selected.size} {selected.size === 1 ? "activity" : "activities"} saved.
                        We'll suggest these when you need a break.
                    </p>
                    <div className="ob-success-chips">
                        {Array.from(selected).map((a) => (
                            <span key={a} className="ob-chip">
                                {ACTIVITY_ICONS[a] ?? "•"} {a}
                            </span>
                        ))}
                    </div>
                    <button className="ob-btn-primary" onClick={onComplete}>
                        Start Session
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="ob-root">
            <div className="ob-card">
                <div className="ob-header">
                    <span className="ob-eyebrow">Setup</span>
                    <h1 className="ob-title">Pick your break activities</h1>
                    <p className="ob-sub">
                        We'll suggest these when you need to step away. Choose as many as you like.
                    </p>
                </div>

                {error && <p className="ob-error">{error}</p>}

                <div className="ob-grid">
                    {activities.map((activity) => {
                        const isSelected = selected.has(activity);
                        return (
                            <button
                                key={activity}
                                className={`ob-activity-btn ${isSelected ? "ob-activity-btn--selected" : ""}`}
                                onClick={() => toggle(activity)}
                            >
                                <span className="ob-activity-icon">
                                    {ACTIVITY_ICONS[activity] ?? "•"}
                                </span>
                                <span className="ob-activity-name">{activity}</span>
                                {isSelected && <span className="ob-check">✓</span>}
                            </button>
                        );
                    })}
                </div>

                <div className="ob-footer">
                    <span className="ob-count">
                        {selected.size > 0
                            ? `${selected.size} selected`
                            : "Select at least one"}
                    </span>
                    <button
                        className="ob-btn-primary"
                        onClick={handleContinue}
                        disabled={selected.size === 0 || loading}
                    >
                        {loading ? "Saving..." : "Continue"}
                    </button>
                </div>
            </div>
        </div>
    );
};