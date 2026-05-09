import './LoadingScreen.css';

export const LoadingScreen = () => {
    return (
        <div className="loading-screen">
            <svg className="loading-ring" width="52" height="52" viewBox="0 0 52 52">
                <circle className="ring-track" cx="26" cy="26" r="22" />
                <circle className="ring-glow" cx="26" cy="26" r="22" transform="rotate(-90 26 26)" />
            </svg>
            <p className="loading-text">Starting up</p>
        </div>
    );
};