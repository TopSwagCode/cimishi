# Integration Tests

Integration tests that verify each storage backend works end-to-end. They spin up emulators in Docker, upload test data, run the pipeline, and check the output.

## Backend Status

| Backend | Emulator | Status |
|---------|----------|--------|
| Local | N/A | Passing |
| S3 | MinIO | Passing |
| Azure Blob | Azurite | Passing |
| GCS | — | Not tested (see below) |

### GCS

The `object_store` crate's GCS adapter requires real Google Cloud credentials even with emulators like `fake-gcs-server` — `STORAGE_EMULATOR_HOST` isn't fully supported. To test GCS, you'll need a real bucket and service account credentials.

## Running

```bash
cd tests
./run-tests.sh
```

Or manually:

```bash
docker compose -f docker-compose.test.yml up --build --abort-on-container-exit
```

### Cleanup

```bash
./run-tests.sh --cleanup
# or
docker compose -f docker-compose.test.yml down -v --remove-orphans
```

## How It Works

1. Start MinIO (S3) on port 9000 and Azurite (Azure) on port 10000
2. Upload test RDF/XML data to each backend
3. Run the pipeline against each backend (local, S3, Azure)
4. Check that the output CSV contains the expected substations

The test data has 3 substations (Alpha, Beta, Gamma). The SPARQL query selects all substation names, so a passing test finds all 3.

## Test Credentials

### MinIO (S3)
```
AWS_ACCESS_KEY_ID=minioadmin
AWS_SECRET_ACCESS_KEY=minioadmin
AWS_REGION=us-east-1
AWS_ENDPOINT_URL=http://minio:9000
```

### Azurite (Azure)
```
AZURE_STORAGE_ACCOUNT_NAME=devstoreaccount1
AZURE_STORAGE_ACCOUNT_KEY=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==
```

## Troubleshooting

**Tests fail at setup** — Make sure Docker is running and has enough resources.

**Can't connect to emulator** — Check if ports 9000 or 10000 are already in use.

**Stuck containers** — `docker compose -f docker-compose.test.yml down -v --remove-orphans && docker system prune -f`
