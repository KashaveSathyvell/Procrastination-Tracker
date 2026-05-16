"""
FocusGuard Model Evaluation Script
Evaluates the retrained ONNX model against real labelled user data from SQLite.
Generates confusion matrix, feature importance chart, and classification metrics.

Usage:
    python evaluate_model.py <db_path> <model_path>

Example:
    python evaluate_model.py "C:/Users/Kash/AppData/Roaming/com.kashave.client/ProcrastinationAI/behavior.db" "C:/Users/Kash/AppData/Roaming/com.kashave.client/ProcrastinationAI/model/current_model.onnx"
"""

import sys
import os
import sqlite3
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
import seaborn as sns
from sklearn.metrics import (
    confusion_matrix,
    classification_report,
    matthews_corrcoef,
    accuracy_score
)
import onnxruntime as rt

# ── constants ──────────────────────────────────────────────────────────────────

FEATURE_COLUMNS = [
    'typing_speed',
    'repetitive_key_ratio',
    'mouse_velocity',
    'idle_ratio',
    'window_switch_frequency',
    'scroll_velocity',
]

LABEL_MAP = {
    0: "Focused",
    1: "At Risk",
    2: "Procrastinating",
    3: "Idle"
}

LABEL_TO_INT = {v: k for k, v in LABEL_MAP.items()}

CLASS_NAMES = ["Focused", "At Risk", "Procrastinating", "Idle"]

# colour palette — matches the app's CSS variables as closely as matplotlib allows
PALETTE = {
    "Focused":        "#4ade80",
    "At Risk":        "#fbbf24",
    "Procrastinating":"#f87171",
    "Idle":           "#9898b0",
}

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "evaluation_outputs")

# ── helpers ────────────────────────────────────────────────────────────────────

def load_labelled_data(db_path: str) -> pd.DataFrame:
    print(f"\nConnecting to database: {db_path}")
    if not os.path.exists(db_path):
        raise FileNotFoundError(f"Database not found: {db_path}")

    conn = sqlite3.connect(db_path)
    query = """
        SELECT
            typing_speed,
            repetitive_key_ratio,
            mouse_velocity,
            idle_ratio,
            window_switch_frequency,
            COALESCE(scroll_velocity, 0.0) AS scroll_velocity,
            truth_label
        FROM feature_vectors
        WHERE truth_label IS NOT NULL
          AND truth_label NOT IN ('Break')
    """
    df = pd.read_sql_query(query, conn)
    conn.close()

    print(f"Loaded {len(df)} labelled rows")
    print(f"\nLabel distribution:\n{df['truth_label'].value_counts().to_string()}")
    return df


def prepare_data(df: pd.DataFrame):
    df = df.copy()
    df = df.dropna(subset=FEATURE_COLUMNS + ['truth_label'])
    df['label_int'] = df['truth_label'].map(LABEL_TO_INT)

    unknown = df['label_int'].isna().sum()
    if unknown > 0:
        print(f"Warning: dropping {unknown} rows with unrecognised labels")
        df = df.dropna(subset=['label_int'])

    df['label_int'] = df['label_int'].astype(int)

    X = df[FEATURE_COLUMNS].values.astype(np.float32)
    y = df['label_int'].values
    labels = df['truth_label'].values
    return X, y, labels


def run_inference(session: rt.InferenceSession, X: np.ndarray):
    input_name = session.get_inputs()[0].name
    predictions = []
    probabilities = []

    for row in X:
        inp = row.reshape(1, -1)
        outputs = session.run(["label", "probabilities"], {input_name: inp})
        predictions.append(int(outputs[0][0]))
        probabilities.append(outputs[1][0])

    return np.array(predictions), np.array(probabilities)


def plot_confusion_matrix(y_true, y_pred, output_dir: str):
    cm = confusion_matrix(y_true, y_pred, labels=[0, 1, 2, 3])
    cm_pct = cm.astype(float) / cm.sum(axis=1, keepdims=True) * 100

    fig, axes = plt.subplots(1, 2, figsize=(14, 5))
    fig.patch.set_facecolor("#16161f")

    for ax in axes:
        ax.set_facecolor("#1e1e2e")

    # raw counts
    sns.heatmap(
        cm, annot=True, fmt='d', cmap='Purples',
        xticklabels=CLASS_NAMES, yticklabels=CLASS_NAMES,
        ax=axes[0], linewidths=0.5, linecolor='#2a2a3e',
        cbar_kws={'shrink': 0.8}
    )
    axes[0].set_title("Confusion Matrix (Counts)", color="white", pad=12, fontsize=13)
    axes[0].set_xlabel("Predicted", color="#9898b0", labelpad=8)
    axes[0].set_ylabel("Actual", color="#9898b0", labelpad=8)
    axes[0].tick_params(colors="white")

    # percentages
    annot = np.array([[f"{v:.1f}%" for v in row] for row in cm_pct])
    sns.heatmap(
        cm_pct, annot=annot, fmt='', cmap='Purples',
        xticklabels=CLASS_NAMES, yticklabels=CLASS_NAMES,
        ax=axes[1], linewidths=0.5, linecolor='#2a2a3e',
        vmin=0, vmax=100,
        cbar_kws={'shrink': 0.8}
    )
    axes[1].set_title("Confusion Matrix (Row %)", color="white", pad=12, fontsize=13)
    axes[1].set_xlabel("Predicted", color="#9898b0", labelpad=8)
    axes[1].set_ylabel("Actual", color="#9898b0", labelpad=8)
    axes[1].tick_params(colors="white")

    plt.suptitle("FocusGuard — Model Evaluation", color="white", fontsize=15, y=1.02)
    plt.tight_layout()

    path = os.path.join(output_dir, "confusion_matrix.png")
    plt.savefig(path, dpi=150, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close()
    print(f"Saved: {path}")
    return cm


def plot_feature_importance(session: rt.InferenceSession, output_dir: str, model_path: str):
    import json
    importances = None

    # Try to load from the JSON file we created during training
    json_path = os.path.join(os.path.dirname(model_path), "feature_importances.json")
    if os.path.exists(json_path):
        try:
            with open(json_path, "r") as f:
                importances = np.array(json.load(f))
        except Exception as e:
            print(f"Could not read JSON importances: {e}")
            
    if importances is None or len(importances) != len(FEATURE_COLUMNS):
        print("Note: feature importances not found. Showing placeholder equal-weight chart.")
        importances = np.ones(len(FEATURE_COLUMNS)) / len(FEATURE_COLUMNS)

    sorted_idx = np.argsort(importances)[::-1]
    sorted_features = [FEATURE_COLUMNS[i] for i in sorted_idx]
    sorted_importance = importances[sorted_idx]

    feature_labels = {
        'typing_speed':            'Typing Speed',
        'repetitive_key_ratio':    'Repetitive Key Ratio',
        'mouse_velocity':          'Mouse Velocity',
        'idle_ratio':              'Idle Ratio',
        'window_switch_frequency': 'Window Switch Freq.',
        'scroll_velocity':         'Scroll Velocity',
    }

    colors = ["#7c6af7"] * len(sorted_features)

    fig, ax = plt.subplots(figsize=(9, 5))
    fig.patch.set_facecolor("#16161f")
    ax.set_facecolor("#1e1e2e")

    bars = ax.barh(
        [feature_labels.get(f, f) for f in sorted_features],
        sorted_importance,
        color=colors, edgecolor='none'
    )

    for bar, val in zip(bars, sorted_importance):
        ax.text(
            bar.get_width() + 0.002, bar.get_y() + bar.get_height() / 2,
            f"{val:.3f}", va='center', ha='left', color='white', fontsize=10
        )

    ax.set_xlabel("Importance", color="#9898b0", labelpad=8)
    ax.set_title("Feature Importance (Random Forest)", color="white", fontsize=13, pad=12)
    ax.tick_params(colors="white")
    ax.spines[:].set_color("#2a2a3e")
    ax.invert_yaxis()
    ax.set_xlim(0, sorted_importance.max() * 1.2)

    plt.tight_layout()
    path = os.path.join(output_dir, "feature_importance.png")
    plt.savefig(path, dpi=150, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close()
    print(f"Saved: {path}")

def plot_per_class_metrics(y_true, y_pred, output_dir: str):
    report = classification_report(
        y_true, y_pred,
        labels=[0, 1, 2, 3],
        target_names=CLASS_NAMES,
        output_dict=True,
        zero_division=0
    )

    metrics_df = pd.DataFrame({
        cls: {
            'Precision': report[cls]['precision'],
            'Recall':    report[cls]['recall'],
            'F1-Score':  report[cls]['f1-score'],
        }
        for cls in CLASS_NAMES
    }).T

    x = np.arange(len(CLASS_NAMES))
    width = 0.25
    metric_colors = ["#7c6af7", "#4ade80", "#fbbf24"]

    fig, ax = plt.subplots(figsize=(10, 5))
    fig.patch.set_facecolor("#16161f")
    ax.set_facecolor("#1e1e2e")

    for i, (metric, color) in enumerate(zip(['Precision', 'Recall', 'F1-Score'], metric_colors)):
        vals = metrics_df[metric].values
        rects = ax.bar(x + i * width, vals, width, label=metric, color=color, alpha=0.9)
        for rect, v in zip(rects, vals):
            ax.text(
                rect.get_x() + rect.get_width() / 2, rect.get_height() + 0.01,
                f"{v:.2f}", ha='center', va='bottom', color='white', fontsize=9
            )

    ax.set_xticks(x + width)
    ax.set_xticklabels(CLASS_NAMES, color='white')
    ax.set_ylim(0, 1.15)
    ax.set_ylabel("Score", color="#9898b0")
    ax.set_title("Per-Class Classification Metrics", color="white", fontsize=13, pad=12)
    ax.legend(facecolor='#2a2a3e', edgecolor='#2a2a3e', labelcolor='white')
    ax.tick_params(colors='white')
    ax.spines[:].set_color("#2a2a3e")

    plt.tight_layout()
    path = os.path.join(output_dir, "per_class_metrics.png")
    plt.savefig(path, dpi=150, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close()
    print(f"Saved: {path}")


def plot_confidence_distribution(y_true, y_pred, probabilities, output_dir: str):
    fig, axes = plt.subplots(1, 2, figsize=(12, 4))
    fig.patch.set_facecolor("#16161f")

    # confidence of correct vs incorrect predictions
    correct_mask = y_true == y_pred
    correct_conf = probabilities[correct_mask].max(axis=1)
    incorrect_conf = probabilities[~correct_mask].max(axis=1)

    for ax in axes:
        ax.set_facecolor("#1e1e2e")
        ax.tick_params(colors='white')
        ax.spines[:].set_color("#2a2a3e")

    axes[0].hist(correct_conf, bins=20, color="#4ade80", alpha=0.8, label=f"Correct (n={len(correct_conf)})", edgecolor='none')
    axes[0].hist(incorrect_conf, bins=20, color="#f87171", alpha=0.8, label=f"Incorrect (n={len(incorrect_conf)})", edgecolor='none')
    axes[0].set_xlabel("Confidence", color="#9898b0")
    axes[0].set_ylabel("Count", color="#9898b0")
    axes[0].set_title("Confidence: Correct vs Incorrect", color="white", fontsize=12)
    axes[0].legend(facecolor='#2a2a3e', edgecolor='#2a2a3e', labelcolor='white')

    # average confidence per class
    class_conf = {CLASS_NAMES[i]: [] for i in range(4)}
    for true_label, prob_row in zip(y_true, probabilities):
        class_conf[CLASS_NAMES[true_label]].append(prob_row.max())

    means = [np.mean(class_conf[c]) if class_conf[c] else 0 for c in CLASS_NAMES]
    colors = [PALETTE[c] for c in CLASS_NAMES]
    bars = axes[1].bar(CLASS_NAMES, means, color=colors, edgecolor='none')
    for bar, val in zip(bars, means):
        axes[1].text(
            bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.01,
            f"{val:.2f}", ha='center', va='bottom', color='white', fontsize=10
        )
    axes[1].set_ylim(0, 1.15)
    axes[1].set_ylabel("Avg Confidence", color="#9898b0")
    axes[1].set_title("Average Confidence per Class", color="white", fontsize=12)
    axes[1].tick_params(colors='white')

    plt.tight_layout()
    path = os.path.join(output_dir, "confidence_distribution.png")
    plt.savefig(path, dpi=150, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close()
    print(f"Saved: {path}")


def print_summary(y_true, y_pred, probabilities):
    acc = accuracy_score(y_true, y_pred)
    mcc = matthews_corrcoef(y_true, y_pred)

    print("\n" + "=" * 60)
    print("  FOCUSGUARD MODEL EVALUATION SUMMARY")
    print("=" * 60)
    print(f"  Total samples evaluated : {len(y_true)}")
    print(f"  Overall accuracy        : {acc * 100:.2f}%")
    print(f"  Matthews Corr. Coeff.   : {mcc:.4f}  (range -1 to +1, higher is better)")
    print()
    print("  Classification Report:")
    print()
    print(classification_report(
        y_true, y_pred,
        labels=[0, 1, 2, 3],
        target_names=CLASS_NAMES,
        zero_division=0
    ))
    print("=" * 60)
    print()
    print("  Interpretation:")
    if mcc >= 0.7:
        print("  MCC >= 0.70 — Strong model performance.")
    elif mcc >= 0.5:
        print("  MCC 0.50–0.69 — Moderate performance. Retraining with more")
        print("  labelled data will improve minority class detection.")
    elif mcc >= 0.3:
        print("  MCC 0.30–0.49 — Weak performance. More labelled corrections needed.")
    else:
        print("  MCC < 0.30 — Model needs significantly more training data.")
    print("=" * 60)


# ── main ───────────────────────────────────────────────────────────────────────

def main():
    if len(sys.argv) != 3:
        print("Usage: python evaluate_model.py <db_path> <model_path>")
        print()
        print("Example:")
        print('  python evaluate_model.py "C:/Users/.../behavior.db" "C:/Users/.../current_model.onnx"')
        sys.exit(1)

    db_path = sys.argv[1]
    model_path = sys.argv[2]

    if not os.path.exists(model_path):
        print(f"Model not found: {model_path}")
        sys.exit(1)

    os.makedirs(OUTPUT_DIR, exist_ok=True)
    print(f"Output directory: {OUTPUT_DIR}")

    # load data
    df = load_labelled_data(db_path)
    if len(df) < 10:
        print(f"Not enough labelled data for evaluation (have {len(df)}, need at least 10).")
        sys.exit(1)

    X, y_true, true_labels = prepare_data(df)
    print(f"\nEvaluating on {len(X)} samples...")

    # run model
    print(f"Loading model: {model_path}")
    session = rt.InferenceSession(model_path)
    y_pred, probabilities = run_inference(session, X)

    # generate all outputs
    print("\nGenerating evaluation charts...")
    plot_confusion_matrix(y_true, y_pred, OUTPUT_DIR)
    plot_feature_importance(session, OUTPUT_DIR, model_path)
    plot_per_class_metrics(y_true, y_pred, OUTPUT_DIR)
    plot_confidence_distribution(y_true, y_pred, probabilities, OUTPUT_DIR)

    # print summary to console
    print_summary(y_true, y_pred, probabilities)

    print(f"\nAll charts saved to: {OUTPUT_DIR}")
    print("Use these PNG files in Chapter 4 (Implementation) and Chapter 5 (Results).")


if __name__ == "__main__":
    main()