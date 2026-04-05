import pandas as pd
import numpy as np
import random

RAW_PATH = "ml/data/raw/EVTRACKTRACK.csv"
OUTPUT_PATH = "ml/data/processed/dataset_v1.csv"

WINDOW_SIZE = 10.0  #Should I switch to 60s? Confirm with spv


def load_and_clean(filepath):
    print("Loading raw dataset...")
    df = pd.read_csv(filepath, sep='\t')

    df = df[['timestamp', 'xpos', 'ypos', 'event', 'key', 'session_id']]

    df['timestamp'] = pd.to_datetime(df['timestamp']).astype('int64') / 10**9

    df = df.sort_values(by=['session_id', 'timestamp']).reset_index(drop=True)

    return df


def extract_features(df):
    print("Extracting features...")
    rows = []

    for session_id, session_data in df.groupby('session_id'):
        start = session_data['timestamp'].iloc[0]
        end = session_data['timestamp'].iloc[-1]

        current = start

        while current < end:
            window_end = current + WINDOW_SIZE

            window = session_data[
                (session_data['timestamp'] >= current) &
                (session_data['timestamp'] < window_end)
            ]

            if len(window) == 0:
                current += WINDOW_SIZE
                continue

            # --- Typing Speed + Repetition ---
            keyboard = window[window['event'].str.contains('key', na=False)]
            keys_pressed = len(keyboard)

            typing_speed = keys_pressed / WINDOW_SIZE

            if keys_pressed > 1:
                keys = keyboard['key'].astype(str).tolist()
                repetitive = sum(
                    1 for i in range(1, len(keys))
                    if keys[i] == keys[i - 1] and keys[i] != '0'
                )
                repetitive_ratio = repetitive / keys_pressed
            else:
                repetitive_ratio = 0.0

            # --- Mouse Velocity ---
            mouse = window[window['event'] == 'mousemove']
            if len(mouse) > 1:
                dx = mouse['xpos'].diff().fillna(0)
                dy = mouse['ypos'].diff().fillna(0)
                distance = np.sqrt(dx**2 + dy**2).sum()
                mouse_velocity = distance / WINDOW_SIZE
            else:
                mouse_velocity = 0.0

            # --- Idle Ratio ---
            timestamps = window['timestamp'].tolist()
            idle_time = 0
            idle_threshold = 2

            if timestamps[0] - current > idle_threshold:
                idle_time += timestamps[0] - current

            for i in range(1, len(timestamps)):
                gap = timestamps[i] - timestamps[i - 1]
                if gap > idle_threshold:
                    idle_time += gap

            if window_end - timestamps[-1] > idle_threshold:
                idle_time += window_end - timestamps[-1]

            idle_ratio = min(idle_time / WINDOW_SIZE, 1.0)

            rows.append({
                "typing_speed": typing_speed,
                "repetitive_key_ratio": repetitive_ratio,
                "mouse_velocity": mouse_velocity,
                "idle_ratio": idle_ratio
            })

            current += WINDOW_SIZE

    return pd.DataFrame(rows)


def apply_labels(df):
    print("Applying labels...")

    def label(row):
        if row['idle_ratio'] >= 0.9:
            return "Idle"
        elif row['typing_speed'] > 1.5 and row['repetitive_key_ratio'] < 0.3:
            return "Focused"
        elif row['repetitive_key_ratio'] > 0.5 or row['mouse_velocity'] > 300:
            return "Procrastinating"
        else:
            return "At Risk"

    df['label'] = df.apply(label, axis=1)


    final_columns = [
        'typing_speed', 'repetitive_key_ratio', 'mouse_velocity', 
        'idle_ratio', 'label'
    ]
    return df[final_columns]

    # return df


if __name__ == "__main__":
    df = load_and_clean(RAW_PATH)
    features = extract_features(df)
    final = apply_labels(features)

    final.to_csv(OUTPUT_PATH, index=False)
    print("Dataset created:", OUTPUT_PATH)

