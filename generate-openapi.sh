#!/bin/bash
set -e

echo "Fetching OpenAPI spec from running server..."
curl -s http://127.0.0.1:3000/api-docs/openapi.json -o openapi.json

echo "Generating TypeScript client..."
cd ui/src && npx @hey-api/openapi-ts

echo "Done!"