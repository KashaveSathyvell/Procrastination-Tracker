## Prerequisites
To compile and run this application locally, please ensure the following environments are installed on your machine:
* Node.js (v18+ recommended)
* Rust & Cargo (Latest stable toolchain)
* Python (v3.10+ for the ML Retraining Sidecar)

## Step-by-Step Installation & Setup

Install Frontend Dependencies
Navigate to the React Client directory and install the required Node modules.

cd app/Client
npm install


Run the Application (from same folder - app/Client)
Tauri will automatically compile the Rust backend and launch the React frontend. Note: The first Rust build may take a few minutes as it downloads the crates.

npm run tauri dev


If users would like to install the app as an application run the following commands. 

cd app/Client
npm run tauri build


Once the compilation build is complete, users can find the executable installer file at the following path:
app/Client/src-tauri/target/release/bundle/nsis/FocusGuard_0.1.0_x64-setup.exe

double click to install the system natively onto you device. 

NOTE: application will only work on Windows operating system.