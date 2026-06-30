# FocusGuard

A desktop application that detects procrastination in real time by analyzing behavioural patterns (typing speed, mouse movement, idle time, window switching) and intervenes with personalized micro-breaks before focus fully breaks down.

Built as a final year project exploring whether passive HCI signals alone — no webcam, no self-reporting — can reliably predict when someone is about to disengage from work.

## How it works

FocusGuard monitors five behavioural signals over rolling 60-second windows:

- Typing speed
- Mouse velocity
- Idle ratio
- Window switching frequency
- Repetitive key ratio

These signals feed into a **Random Forest classifier** that sorts the user's current state into one of four categories: **Focused**, **At Risk**, **Procrastinating**, or **Idle**.

When the model detects a user drifting toward "At Risk" or "Procrastinating," a **Just-in-Time Adaptive Intervention (JITAI)** controller decides whether and how to intervene. A **Multi-Armed Bandit** selector picks which type of break or nudge is likely to be most effective for that user, learning over time which interventions actually work for them.

A **Human-in-the-Loop retraining pipeline** lets the model improve from real usage rather than staying static after initial training.

## Results

- **95.82% classification accuracy**
- **MCC (Matthews Correlation Coefficient): 0.9408**
- Validated with **30 participants** in a user acceptance testing (UAT) study
- Benchmarked against existing procrastination-detection research, outperforming comparable approaches (e.g. Altuwairqi et al., 95.23% accuracy)

## Tech stack

- **Rust** — core application logic, event capture, inference pipeline (via Tauri backend)
- **React + TypeScript** — desktop UI (via Tauri frontend)
- **Python** — model training pipeline
- **ONNX** — model export format for fast in-app inference
- **SQLite** — local storage for behavioural event logs and session history

## Architecture
