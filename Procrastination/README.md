## Prerequisites
To compile and run this application locally, please ensure the following environments are installed on your machine:
* Node.js (v18+ recommended)
* Rust & Cargo (Latest stable toolchain)
* Python (v3.10+ for the ML Retraining Sidecar)

## 🛠️ Step-by-Step Installation & Setup

Install Frontend Dependencies
Navigate to the React Client directory and install the required Node modules.

cd app/Client
npm install


Run the Application (Development Mode)
Tauri will automatically compile the Rust backend and launch the React frontend. Note: The first Rust build may take a few minutes as it downloads the crates.

npm run tauri dev