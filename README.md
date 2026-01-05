# Rust Cybersecurity Asset Management Platform

This is a full-stack Rust application for managing network assets, performing port scans, and monitoring security compliance.

## Structure

- **frontend**: Yew (Rust + WASM) application.
- **backend**: Axum (Rust) REST API server.
- **shared**: Shared Rust types (Asset, NetworkZone, etc.) between frontend and backend.

## Features

- **Asset Management**: View and manually add assets with Name, IP, and Network Zone.
- **Scanning**:
    - **Manual Scan**: Trigger a scan for a specific asset.
    - **Periodic Scan**: Backend background task simulates scanning assets every 30 seconds.
- **Security Alerts**:
    - Detects open ports.
    - **Unbound Port Warning**: Highlights ports that are open but not manually bound/approved (marked with ⚠️).
- **Network Zones**: Classify assets into Intranet, DMZ, or Internet.

## Prerequisites

- Rust (cargo)
- `trunk` (for building the frontend): `cargo install trunk`

## How to Run

1. **Start the Backend**:
   ```bash
   cargo run -p backend
   ```
   The server will start on `http://127.0.0.1:3000`.

2. **Start the Frontend**:
   ```bash
   cd frontend
   trunk serve --open
   ```
   The application will open in your browser at `http://127.0.0.1:8080`.

## Architecture

- **Frontend**: Yew calls backend APIs using `gloo-net`. State is managed via `use_state` and updated via API responses.
- **Backend**: Axum handles HTTP requests. State is kept in-memory (`Arc<Mutex<Vec<Asset>>>`) for this demo, simulating a database.
- **Scanner**: A Tokio background task runs periodically to simulate network scanning and finding new/rogue ports.
