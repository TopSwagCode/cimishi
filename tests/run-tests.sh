#!/bin/bash
set -e

# Integration test runner for RDF Query Pipeline
# Usage: ./run-tests.sh [--cleanup] [--all]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "RDF Query Pipeline - Integration Tests"
echo "=========================================="
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    docker compose -f docker-compose.test.yml down -v --remove-orphans 2>/dev/null || true
}

# Handle Ctrl+C
trap cleanup EXIT

# Check if just cleanup requested
if [ "$1" = "--cleanup" ]; then
    cleanup
    echo "Cleanup complete"
    exit 0
fi

# Clean previous runs
echo "Stopping any previous test containers..."
docker compose -f docker-compose.test.yml down -v --remove-orphans 2>/dev/null || true

# Clean output directories
rm -rf output/local/* output/s3/* output/azure/* output/gcs/* 2>/dev/null || true

echo ""
echo "Building and starting test infrastructure..."
echo "  - MinIO (S3 emulator)"
echo "  - Azurite (Azure Blob emulator)"
echo "  - fake-gcs-server (GCS emulator)"
echo ""

# Run all tests
docker compose -f docker-compose.test.yml up test-validator --build --abort-on-container-exit

echo ""
echo "=========================================="
echo "Integration tests complete!"
echo "=========================================="
