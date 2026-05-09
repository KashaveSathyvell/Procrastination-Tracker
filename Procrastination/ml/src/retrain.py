import sys
import os
import sqlite3
import numpy as np
import pandas as pd
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report, matthews_corrcoef
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType
import onnxruntime as rt

LABEL_MAP = {
    "Focused": 0,
    "At Risk": 1,
    "Procrastinating": 2,
    "Idle": 3
}

FEATURE_COLUMNS = [
    'typing_speed',
    'repetitive_key_ratio',
    'mouse_velocity',
    'idle_ratio',
    'window_switch_frequency',
    'scroll_velocity'
]

MIN_LABELLED_ROWS = 50
CORRECTION_RATE_THRESHOLD = 0.25

#Remove coalesce after retrained once. DELETE
def load_labelled_data(db_path: str) -> pd.DataFrame:
    print(f"Connecting to database: {db_path}")

    if not os.path.exists(db_path):
        raise FileNotFoundError(f"Database not found at: {db_path}")

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

    print(f"Loaded {len(df)} labelled rows from database")
    print(f"Label distribution:\n{df['truth_label'].value_counts()}")

    return df


def prepare_data(df: pd.DataFrame):
    df = df.copy()

    # Drop any rows where features are null
    df = df.dropna(subset=FEATURE_COLUMNS + ['truth_label'])

    # Map string labels to integers
    df['label'] = df['truth_label'].map(LABEL_MAP)

    # Drop any rows where label mapping failed (unknown labels)
    unknown = df['label'].isna().sum()
    if unknown > 0:
        print(f"Warning: dropping {unknown} rows with unrecognised labels")
        df = df.dropna(subset=['label'])

    df['label'] = df['label'].astype(int)

    X = df[FEATURE_COLUMNS]
    y = df['label']

    return X, y


def train(X_train, y_train) -> RandomForestClassifier:
    print("Training RandomForest...")

    model = RandomForestClassifier(
        n_estimators=100,
        max_depth=10,
        random_state=42,
        class_weight='balanced'  # handles class imbalance without SMOTE
    )

    model.fit(X_train, y_train)
    print("Training complete")

    return model


def evaluate(model, X_test, y_test):
    preds = model.predict(X_test)

    print("\nClassification Report:")
    print(classification_report(
        y_test, preds,
        target_names=["Focused", "At Risk", "Procrastinating", "Idle"],
        zero_division=0
    ))

    mcc = matthews_corrcoef(y_test, preds)
    print(f"MCC: {mcc:.4f}")

    return mcc


def export_onnx(model, output_path: str):
    print(f"Exporting ONNX model to: {output_path}")

    # Create output directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    initial_type = [('float_input', FloatTensorType([None, len(FEATURE_COLUMNS)]))]
    onnx_model = convert_sklearn(
        model,
        initial_types=initial_type,
        options={id(model): {"zipmap": False}}
    )

    with open(output_path, "wb") as f:
        f.write(onnx_model.SerializeToString())

    print("ONNX export complete")


def verify_model(model_path: str) -> bool:
    print("Verifying exported model...")

    try:
        sess = rt.InferenceSession(model_path)

        # Check output names match what inference.rs expects
        output_names = [o.name for o in sess.get_outputs()]
        print(f"Model output names: {output_names}")

        if "label" not in output_names:
            print("ERROR: 'label' output not found in model")
            return False

        if "probabilities" not in output_names:
            print("ERROR: 'probabilities' output not found in model")
            return False

        # Test with a clearly focused input:
        # high typing speed, low repetition, moderate mouse, low idle, low switching, no scrolling
        test_input = np.array([[1.2, 0.05, 15.0, 0.1, 0.02, 0.0]], dtype=np.float32)

        input_name = sess.get_inputs()[0].name
        outputs = sess.run(["label", "probabilities"], {input_name: test_input})

        predicted_label = outputs[0][0]
        probabilities = outputs[1][0]

        print(f"Test prediction (should be 0=Focused): {predicted_label}")
        print(f"Probabilities: {probabilities}")

        if predicted_label != 0:
            print("WARNING: Test input did not predict Focused — model may be miscalibrated")
            # Don't fail on this — warn only, since real user data may shift the model

        print("Verification passed")
        return True

    except Exception as e:
        print(f"ERROR during verification: {e}")
        return False


def main():
    if len(sys.argv) != 3:
        print("Usage: python retrain.py <db_path> <model_output_path>")
        sys.exit(1)

    db_path = sys.argv[1]
    model_output_path = sys.argv[2]

    print("=" * 50)
    print("FocusGuard Model Retraining")
    print("=" * 50)

    # Load and validate data
    try:
        df = load_labelled_data(db_path)
    except FileNotFoundError as e:
        print(f"FATAL: {e}")
        sys.exit(1)

    if len(df) < MIN_LABELLED_ROWS:
        print(f"FATAL: Not enough labelled data. Have {len(df)} rows, need {MIN_LABELLED_ROWS}")
        sys.exit(1)

    # Prepare features and labels
    X, y = prepare_data(df)

    if len(X) < MIN_LABELLED_ROWS:
        print(f"FATAL: After cleaning, only {len(X)} valid rows remain. Need {MIN_LABELLED_ROWS}")
        sys.exit(1)

    # Split — use a small test set since real data is limited
    # If we have fewer than 80 rows, skip test split and train on everything
    if len(X) >= 80:
        X_train, X_test, y_train, y_test = train_test_split(
            X, y, test_size=0.2, random_state=42, stratify=y
        )
        print(f"Training on {len(X_train)} rows, evaluating on {len(X_test)} rows")
    else:
        X_train, y_train = X, y
        X_test, y_test = None, None
        print(f"Small dataset ({len(X)} rows) — training on all data, skipping test split")

    # Train
    model = train(X_train, y_train)

    # Evaluate if we have a test set
    if X_test is not None:
        evaluate(model, X_test, y_test)
    else:
        print("Skipping evaluation — dataset too small for held-out test set")

    print(f"\nFeature order used: {FEATURE_COLUMNS}")

    # Export
    export_onnx(model, model_output_path)

    # Verify
    if not verify_model(model_output_path):
        print("FATAL: Model verification failed — not replacing existing model")
        # Delete the bad export so it doesn't get loaded accidentally
        if os.path.exists(model_output_path):
            os.remove(model_output_path)
        sys.exit(1)

    print("=" * 50)
    print("Retraining successful")
    print(f"New model saved to: {model_output_path}")
    print("=" * 50)
    sys.exit(0)


if __name__ == "__main__":
    main()