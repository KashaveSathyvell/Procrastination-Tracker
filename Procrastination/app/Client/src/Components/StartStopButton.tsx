import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'
import "./StartStopButton.css"

export const StartStopButton = () => {
    const [isRunning, setIsRunning] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleStart = async () => {
        setIsLoading(true);
        setError(null);
        try {
            await invoke('start_collect');
            setIsRunning(true);
        } catch (e) {
            setError(`Failed to start: ${e}`);
        } finally {
            setIsLoading(false);
        }
    };

    const handleStop = async () => {
        setIsLoading(true);
        setError(null);
        try {
            await invoke('stop_collect');
            setIsRunning(false);
        } catch (e) {
            setError(`Failed to stop: ${e}`);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="button_container">
            <button
                onClick={handleStart}
                disabled={isRunning || isLoading}
            >
                {isLoading && !isRunning ? 'Starting...' : 'Start'}
            </button>
            <button
                onClick={handleStop}
                disabled={!isRunning || isLoading}
            >
                {isLoading && isRunning ? 'Stopping...' : 'Stop'}
            </button>
            {error && <p className="start-stop-error">{error}</p>}
        </div>
    )
}