import pandas as pd
from sklearn.model_selection import train_test_split

LABEL_MAP = {
    "Focused": 0,
    "At Risk": 1,
    "Procrastinating": 2,
    "Idle": 3
}


def load_dataset(path):
    df = pd.read_csv(path)
    return df


def preprocess(df):
    df = df.copy()

    df['label'] = df['label'].map(LABEL_MAP)

    X = df.drop("label", axis=1)
    y = df["label"]

    return X, y


def split(X, y):
    return train_test_split(X, y, test_size=0.2, random_state=42)