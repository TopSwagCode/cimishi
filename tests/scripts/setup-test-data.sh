#!/bin/sh
set -e

echo "=== Setting up test data for all backends ==="

# Install required tools
apk add --no-cache curl > /dev/null 2>&1

# Wait for services to be ready
echo "Waiting for services..."
sleep 5

# MinIO (S3) Setup
echo "Setting up MinIO (S3)..."
curl -s -X POST "http://minio:9000/minio/admin/v3/add-bucket?accessKey=minioadmin&secretKey=minioadmin&name=test-bucket" || true

for f in /test-data/*.xml /test-data/*.sparql; do
  [ -f "$f" ] || continue
  filename=$(basename "$f")
  curl -s -X PUT "http://minio:9000/test-bucket/$filename" \
    -H "Content-Type: application/xml" \
    --data-binary "@$f"
  echo "  Uploaded to MinIO: $filename"
done

# Azurite (Azure) Setup
echo "Setting up Azurite (Azure Blob)..."
# Create container
curl -s -X PUT "http://azurite:10000/devstoreaccount1/test-container?restype=container" \
  -H "x-ms-version: 2019-12-12" \
  -H "Authorization: SharedKey devstoreaccount1:Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==" || true

# Upload files
for f in /test-data/*.xml /test-data/*.sparql; do
  [ -f "$f" ] || continue
  filename=$(basename "$f")
  curl -s -X PUT "http://azurite:10000/devstoreaccount1/test-container/$filename" \
    -H "x-ms-version: 2019-12-12" \
    -H "x-ms-blob-type: BlockBlob" \
    -H "Authorization: SharedKey devstoreaccount1:Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==" \
    --data-binary "@$f"
  echo "  Uploaded to Azurite: $filename"
done

# fake-gcs-server (GCS) Setup
echo "Setting up fake-gcs-server (GCS)..."

# Create bucket via API
curl -s -X POST "http://fake-gcs:4443/storage/v1/b?project=test" \
  -H "Content-Type: application/json" \
  -d '{"name": "test-bucket"}' > /dev/null || true

# Upload files
for f in /test-data/*.xml /test-data/*.sparql; do
  [ -f "$f" ] || continue
  filename=$(basename "$f")
  curl -s -X POST "http://fake-gcs:4443/upload/storage/v1/b/test-bucket/o?uploadType=media&name=$filename" \
    -H "Content-Type: application/xml" \
    --data-binary "@$f"
  echo "  Uploaded to GCS: $filename"
done

echo ""
echo "=== All backends ready for testing ==="
