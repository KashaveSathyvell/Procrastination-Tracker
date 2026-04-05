import os
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import classification_report, confusion_matrix, matthews_corrcoef
from imblearn.over_sampling import SMOTE

from preprocessing import load_dataset, preprocess, split

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

DATA_PATH = os.path.join(BASE_DIR, "data", "processed", "dataset_v1.csv")
MODEL_PATH = os.path.join(BASE_DIR, "models", "baseline_model.onnx")

from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType


def train(X_train, y_train):
    print("Applying SMOTE...")
    smote = SMOTE(random_state=42)
    X_bal, y_bal = smote.fit_resample(X_train, y_train)

    model = RandomForestClassifier(
        n_estimators=100,
        max_depth=10,
        random_state=42
    )

    model.fit(X_bal, y_bal)
    return model


def evaluate(model, X_test, y_test):
    preds = model.predict(X_test)

    print("Confusion Matrix:")
    print(confusion_matrix(y_test, preds))

    print("\nClassification Report:")
    print(classification_report(y_test, preds))

    mcc = matthews_corrcoef(y_test, preds)
    print("\nMCC:", mcc)


def export_onnx(model, num_features):
    initial_type = [('float_input', FloatTensorType([None, num_features]))]
    onnx_model = convert_sklearn(model, initial_types=initial_type)

    with open(MODEL_PATH, "wb") as f:
        f.write(onnx_model.SerializeToString())

    print("ONNX model saved.")


if __name__ == "__main__":
    df = load_dataset(DATA_PATH)
    X, y = preprocess(df)

    X_train, X_test, y_train, y_test = split(X, y)

    model = train(X_train, y_train)
    evaluate(model, X_test, y_test)

    export_onnx(model, X_train.shape[1])