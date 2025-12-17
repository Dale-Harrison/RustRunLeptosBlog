# Rust Cloud Run Blog

A full-stack, serverless-ready blog application built with **Rust**, **Leptos**, and **Actix Web**, designed to run on **Google Cloud Run** with persistent **SQLite** storage backed by Google Cloud Storage (GCS) FUSE.

## 🚀 Features

*   **Full-Stack Rust**: Shared types between backend (Actix) and frontend (Leptos).
*   **Serverless Persistence**: Uses SQLite running on Cloud Run Gen 2 with GCS FUSE mounting, enabling persistent relational data without managing a SQL server.
*   **Authentication**: Google OAuth 2.0 integration for Admin access.
*   **Admin Dashboard**: create, edit, and delete posts; manage users and comments.
*   **Image Uploads**: Direct upload to private Google Cloud Storage bucket with administrative controls.
*   **Markdown Support**: Write posts and comments using Markdown, rendered securely on the frontend.
*   **Secure**:
    *   Dedicated Service Account with least-privilege permissions.
    *   Secrets management via Google Secret Manager.
    *   CSRF protection and secure session management.

## 🛠️ Tech Stack

*   **Frontend**: [Leptos](https://leptos.dev/) (Rust WebAssembly) + [Tailwind CSS](https://tailwindcss.com/)
*   **Backend**: [Actix Web](https://actix.rs/)
*   **Database**: SQLite (via `libsqlite3-sys` / `stoolap` / `sqlx`)
*   **Infrastructure**: Google Cloud Run (Gen 2), Cloud Build, Cloud Storage, Secret Manager.

## 📂 Project Structure

*   `backend/`: The Actix Web server API and static file handling.
*   `leptos_frontend/`: The Leptos WASM frontend application.
*   `load-tests/`: k6 load testing scripts.
*   `deploy.sh`: Deployment automation script.
*   `setup_env.sh`: Infrastructure initialization script.

## ☁️ Deployment Guide

This project is optimized for Google Cloud Platform.

### Prerequisites

1.  [Google Cloud SDK](https://cloud.google.com/sdk/docs/install) installed and authenticated (`gcloud auth login`).
2.  A Google Cloud Project created.

### 1. Initial Setup

Run the setup script to enable required APIs, create the storage bucket, configure secrets (OAuth Client ID/Secret), and provision the Service Account.

```bash
# Export your project variables
export PROJECT_ID="your-project-id"
export REGION="us-central1" # or your preferred region
export BUCKET_NAME="your-unique-bucket-name"

# Run the setup
./setup_env.sh
```

You will be prompted to enter your **Google OAuth Client ID** and **Client Secret**. (Create these in the [Google Cloud Console Credentials page](https://console.cloud.google.com/apis/credentials)).

### 2. Deployment

The deployment script handles building the container (using Cloud Build) and deploying it to Cloud Run with the correct volume mounts for the SQLite database.

```bash
./deploy.sh
```

### 3. OAuth Configuration

After deployment, the script will output your **Service URL** and **Custom Domain** (if configured).

1.  Go back to your Google Cloud Console -> APIs & Services -> Credentials.
2.  Edit your OAuth 2.0 Client.
3.  Add the following **Authorized Redirect URI**:
    *   `https://<your-service-url>/auth/callback`

## 🧪 Local Development

To run locally, you need Rust installed.

1.  **Frontend**:
    ```bash
    cd leptos_frontend
    wasm-pack build --target web --dev
    ```
2.  **Backend**:
    ```bash
    cd backend
    cargo run
    ```
    *Note: You will need to set up local environment variables in `backend/.env` matching those in `deploy.sh` for local execution.*

## 🛡️ Security

*   **Service Account**: The application runs as a dedicated service account (`rust-app-runtime`) with restricted permissions (Storage Object Admin on the specific bucket only, Secret Accessor, Log Writer).
*   **Database**: The SQLite database file sits on a GCS bucket mounted at `/mnt/gcs`. Cloud Run Gen 2 allows this to persist across container restarts.

## 📄 License

This project is open source.
