# Healthcheck Endpoint Proposal

## Overview

Endpoint: `GET /api/public/hc`  
Response Content-Type: `application/json`

## Services to Monitor

### External Dependencies

- PostgreSQL database connection
- Redis cache instance
- Loki logging system
- Prometheus metrics server

### Internal Systems

- Background worker queues
- Rate limiting subsystem
- WebSocket connections
- Database schema version

## Additional Functionality

1. **Standard Fields**

   - App version from Cargo.toml
   - Server timestamp (UTC)
   - Uptime duration
   - Git commit SHA (if available)
   - DB scheme version

2. **Status Details**

   - Success boolean for overall status
   - Individual service status with latency metrics
   - Optional degraded state detection

3. **Security**
   - Rate limiting for the endpoint
   - CORS configuration
   - Cache-Control headers

## Implementation Steps

1. [x] Add new `healthcheck` module in src/routes/
2. [x] Create dummy (Ok()) hc function and add endpoint to misc routes
3. [x] Create service check traits for unified interface
4. Create each service hc impl one by one
   1. [x] create basic healthcheck response (HTTP 200 response only)
   2. [] create postgres hc impl and add to route
   3. [] create redis hc implementation and add to route
5. [] Implement parallel checks using tokio::join!
6. [] Update OpenAPI documentation
7. [] Integration tests with mocked dependencies
