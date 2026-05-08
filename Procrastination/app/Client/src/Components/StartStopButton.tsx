import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'
import "./StartStopButton.css"

type StartStopButtonProps = {
    isMonitoring: boolean,
    onMonitoringChange: (active: boolean) => void,
}

export const StartStopButton = ({ isMonitoring, onMonitoringChange }: StartStopButtonProps) => {
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const isStateNotManagedError = (err: unknown) => {
        const msg =
            typeof err === 'string'
                ? err
                : (err as any)?.message
                    ? String((err as any).message)
                    : String(err);

        return (
            msg.includes('state not managed') ||
            msg.includes('must call') && msg.includes('.manage()') ||
            msg.includes('.manage()')
        );
    };

    const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

    const handleStart = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const maxAttempts = 4;
            const delayMs = 400;

            for (let attempt = 0; attempt < maxAttempts; attempt++) {
                try {
                    await invoke('start_collect');
                    onMonitoringChange(true);
                    return;
                } catch (err) {
                    const shouldRetry =
                        isStateNotManagedError(err) && attempt < maxAttempts - 1;

                    if (!shouldRetry) throw err;
                    await sleep(delayMs);
                }
            }
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
            onMonitoringChange(false);
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
                disabled={isMonitoring || isLoading}
            >
                {isLoading && !isMonitoring ? 'Starting...' : 'Start'}
            </button>
            <button
                onClick={handleStop}
                disabled={!isMonitoring || isLoading}
            >
                {isLoading && isMonitoring ? 'Stopping...' : 'Stop'}
            </button>
            {error && <p className="start-stop-error">{error}</p>}
        </div>
    )
}