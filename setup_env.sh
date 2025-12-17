#!/bin/bash
set -e

# --- Configuration ---
# Replace these with your actual values or set them as environment variables
PROJECT_ID="${PROJECT_ID:-your-project-id}"
REGION="${REGION:-us-central1}"
BUCKET_NAME="${BUCKET_NAME:-your-bucket-name}"

# Secrets
# You can export these before running or the script will prompt
GOOGLE_CLIENT_ID="${GOOGLE_CLIENT_ID}"
GOOGLE_CLIENT_SECRET="${GOOGLE_CLIENT_SECRET}"

if [ "$PROJECT_ID" == "your-project-id" ]; then
    echo "Error: Please set PROJECT_ID environment variable or edit the script."
    exit 1
fi

echo "========================================================"
echo "Setting up environment for Project: $PROJECT_ID"
echo "Region: $REGION"
echo "Bucket: $BUCKET_NAME"
echo "========================================================"

# 1. Enable APIs
echo "Enabling necessary APIs (run, cloudbuild, secretmanager, storage)..."
gcloud services enable \
    run.googleapis.com \
    cloudbuild.googleapis.com \
    secretmanager.googleapis.com \
    storage.googleapis.com \
    --project "$PROJECT_ID"

# 2. Create Storage Bucket
echo "Checking Cloud Storage bucket: $BUCKET_NAME..."
if ! gcloud storage buckets describe "gs://$BUCKET_NAME" --project "$PROJECT_ID" &>/dev/null; then
    echo "Creating bucket..."
    gcloud storage buckets create "gs://$BUCKET_NAME" --project "$PROJECT_ID" --location "$REGION"
else
    echo "Bucket $BUCKET_NAME already exists."
fi

# 3. Create Secrets
create_secret() {
    local name=$1
    local value=$2
    
    echo "Configuring secret: $name"
    
    # Create secret if it doesn't exist
    if ! gcloud secrets describe "$name" --project "$PROJECT_ID" &>/dev/null; then
        gcloud secrets create "$name" --replication-policy="automatic" --project "$PROJECT_ID"
    fi
    
    # Add a new version
    echo -n "$value" | gcloud secrets versions add "$name" --data-file=- --project "$PROJECT_ID"
}

if [ -z "$GOOGLE_CLIENT_ID" ]; then
    read -p "Enter GOOGLE_CLIENT_ID: " GOOGLE_CLIENT_ID
fi
create_secret "GOOGLE_CLIENT_ID" "$GOOGLE_CLIENT_ID"

if [ -z "$GOOGLE_CLIENT_SECRET" ]; then
    read -s -p "Enter GOOGLE_CLIENT_SECRET: " GOOGLE_CLIENT_SECRET
    echo ""
fi
create_secret "GOOGLE_CLIENT_SECRET" "$GOOGLE_CLIENT_SECRET"

# 4. Create and Configure Service Account
echo "Configuring Service Account..."
SERVICE_ACCOUNT_NAME="rust-app-runtime"
SERVICE_ACCOUNT_EMAIL="${SERVICE_ACCOUNT_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

# Create Service Account if it doesn't exist
if ! gcloud iam service-accounts describe "$SERVICE_ACCOUNT_EMAIL" --project "$PROJECT_ID" &>/dev/null; then
    echo "Creating service account: $SERVICE_ACCOUNT_NAME..."
    gcloud iam service-accounts create "$SERVICE_ACCOUNT_NAME" \
        --display-name="Runtime SA for Rust App" \
        --project "$PROJECT_ID"
else
    echo "Service account $SERVICE_ACCOUNT_NAME already exists."
fi

# Grant Project-Level Permissions
echo "Granting roles to $SERVICE_ACCOUNT_EMAIL..."

# Secret Accessor (to read GOOGLE_CLIENT_ID/SECRET)
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
    --member="serviceAccount:$SERVICE_ACCOUNT_EMAIL" \
    --role="roles/secretmanager.secretAccessor" \
    --condition=None

# Log Writer (to write app logs)
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
    --member="serviceAccount:$SERVICE_ACCOUNT_EMAIL" \
    --role="roles/logging.logWriter" \
    --condition=None

# Grant Bucket-Level Permissions (Least Privilege)
echo "Granting bucket access..."
gcloud storage buckets add-iam-policy-binding "gs://$BUCKET_NAME" \
    --member="serviceAccount:$SERVICE_ACCOUNT_EMAIL" \
    --role="roles/storage.objectAdmin"

echo "========================================================"
echo "Setup complete!"
echo "You can now run ./deploy.sh"
echo "========================================================"
