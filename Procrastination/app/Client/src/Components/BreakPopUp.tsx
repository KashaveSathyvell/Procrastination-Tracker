// BreakPopUp.tsx
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from 'react';
import { BreakData } from './PopUp';
import './BreakPopUp.css';



type BreakState = 'active' | 'expired';

type BreakPopUpProps = {
    breakData: BreakData,
    onBreakEnd: () => void,
}

export const BreakPopUp = ({ breakData, onBreakEnd }: BreakPopUpProps) => {
    const [breakState, setBreakState] = useState<BreakState>('active');
    const [secondsLeft, setSecondsLeft] = useState((breakData.duration ?? 5) * 60);

    useEffect(() => {
        setSecondsLeft((breakData.duration ?? 5) * 60);
        setBreakState('active');
    }, [breakData]);

    useEffect(() => {
        if (secondsLeft <= 0) {
            setBreakState('expired');
            return;
        }
        const tick = setInterval(() => {
            setSecondsLeft(s => s - 1);
        }, 1000);
        return () => clearInterval(tick);
    }, [secondsLeft]);

    const formatTime = (secs: number) => {
        const m = Math.floor(secs / 60).toString().padStart(2, '0');
        const s = (secs % 60).toString().padStart(2, '0');
        return `${m}:${s}`;
    };

    const handleImBack = async () => {
        try {
            await invoke('break_end', {
                endBreak: {
                    breakSessionId: breakData.break_session_id,
                    returnedOnTime: breakState === 'active',
                }
            });
        } catch (e) {
            console.error('end_break failed:', e);
        }
        onBreakEnd();
    };

    const handleExtend = () => {
        setSecondsLeft(5 * 60);
        setBreakState('active');
    };

    return (
        <div className="break-overlay">
            <div className="break-box">
                {breakState === 'active' ? (
                    <>
                        <p className="break-header">On a break</p>
                        <p className="break-activity-name">{breakData.activity}</p>
                        <p className="break-timer">{formatTime(secondsLeft)}</p>
                        <p className="break-hint">Come back when you're ready</p>
                        <button className="btn-im-back" onClick={handleImBack}>
                            I'm back
                        </button>
                    </>
                ) : (
                    <>
                        <p className="break-header">Break time's up</p>
                        <p className="break-activity-name">{breakData.activity}</p>
                        <p className="break-expired-msg">Ready to get back into it?</p>
                        <div className="break-expired-actions">
                            <button className="btn-im-back" onClick={handleImBack}>
                                Let's go
                            </button>
                            <button className="btn-extend" onClick={handleExtend}>
                                5 more mins
                            </button>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
};