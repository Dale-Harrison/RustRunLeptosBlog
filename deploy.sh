#!/bin/bash
set -e

# --- Configuration ---
# Replace these with your actual values or set them as environment variables
PROJECT_ID="${PROJECT_ID:-websitehosting-403318}"
BUCKET_NAME="${BUCKET_NAME:-run-bucket-sqlite}"
REGION="${REGION:-us-central1}"
SERVICE_NAME="rust-next-hello"
IMAGE_NAME="us-central1-docker.pkg.dev/$PROJECT_ID/cloud-run-rust-blog/$SERVICE_NAME"
SERVICE_ACCOUNT_EMAIL="rust-app-runtime@$PROJECT_ID.iam.gserviceaccount.com"


# Parse arguments
RUN_TESTS="false"
for arg in "$@"; do
    if [ "$arg" == "--run-tests" ]; then
        RUN_TESTS="true"
    fi
done

echo "========================================================"
echo "Deploying $SERVICE_NAME to Project: $PROJECT_ID"
echo "Using GCS Bucket: $BUCKET_NAME for database storage"
echo "Service Account: $SERVICE_ACCOUNT_EMAIL"
echo "Run Tests: $RUN_TESTS"
echo "========================================================"

# 1. Build the container image using Cloud Build
echo "Building container image with caching..."
gcloud builds submit --config cloudbuild.yaml --substitutions=_IMAGE_NAME="$IMAGE_NAME",_RUN_TESTS="$RUN_TESTS" .

# 2. Deploy to Cloud Run
# We use Gen 2 execution environment to support GCS volume mounts
# We limit max-instances to 1 to avoid SQLite write conflicts
echo "Deploying to Cloud Run..."
gcloud run deploy "$SERVICE_NAME" \
    --image "$IMAGE_NAME" \
    --platform managed \
    --region "$REGION" \
    --allow-unauthenticated \
    --service-account "$SERVICE_ACCOUNT_EMAIL" \
    --max-instances 1 \
    --execution-environment gen2 \
    --add-volume=name=db-storage,type=cloud-storage,bucket="$BUCKET_NAME" \
    --add-volume-mount=volume=db-storage,mount-path=/mnt/gcs \
    --set-env-vars "DATABASE_URL=file:///mnt/gcs/stoolap_v2.db" \
    --set-secrets "GOOGLE_CLIENT_ID=GOOGLE_CLIENT_ID:latest" \
    --set-secrets "GOOGLE_CLIENT_SECRET=GOOGLE_CLIENT_SECRET:latest" \
    --set-env-vars "RUST_LOG=info" \
    --set-env-vars "RUST_BACKTRACE=1" \
    --set-env-vars "BUCKET_NAME=${BUCKET_NAME}"

# Get the Service URL
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" --platform managed --region "$REGION" --format 'value(status.url)')
echo "Service URL: $SERVICE_URL"

# Generate or read session key
if [ ! -f .session_key ]; then
    echo "Generating new session key..."
    openssl rand -hex 64 > .session_key
fi
SESSION_KEY=$(cat .session_key)

# 3. Update the service with the REDIRECT_URL and SESSION_KEY
echo "Updating service with REDIRECT_URL and SESSION_KEY..."
CUSTOM_DOMAIN="https://inthedustyclocklesshours.balquidderocklabs.com"
gcloud run services update "$SERVICE_NAME" \
    --platform managed \
    --region "$REGION" \
    --set-env-vars "REDIRECT_URL=${CUSTOM_DOMAIN}/auth/callback" \
    --set-env-vars "SESSION_KEY=${SESSION_KEY}" \
    --set-env-vars "DATABASE_URL=file:///mnt/gcs/stoolap_v2.db" \
    --set-env-vars "BUCKET_NAME=${BUCKET_NAME}"

echo "Deployment complete!"
echo "Service URL (Default): $SERVICE_URL"
echo "Service URL (Custom): $CUSTOM_DOMAIN"
echo "IMPORTANT: Ensure both URLs are added to your OAuth Client Authorized Redirect URIs in Google Cloud Console."
echo " - ${SERVICE_URL}/auth/callback"
echo " - ${CUSTOM_DOMAIN}/auth/callback"
