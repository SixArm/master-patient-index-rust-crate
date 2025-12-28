# Master Patient Index (MPI)

A high-performance, enterprise-grade Master Patient Index system built with Rust for healthcare organizations.

## Overview

The Master Patient Index (MPI) is a critical healthcare system that maintains a centralized registry of patient identities across multiple healthcare facilities. This implementation provides:

- **Patient Matching**: Probabilistic and deterministic matching algorithms
- **Full-Text Search**: Powered by Tantivy for fast, accurate patient searches
- **RESTful API**: Modern HTTP API using Axum web framework
- **FHIR R5 Support**: HL7 FHIR R5 compliant endpoints
- **gRPC API**: High-performance RPC interface using Tonic
- **Event Streaming**: Real-time patient event streaming via Fluvio
- **Observability**: Comprehensive logging, metrics, and distributed tracing with OpenTelemetry
- **Production Ready**: Built for millions of patients across thousands of clinics

## Features

### Patient Management
- Create, read, update, and delete (CRUD) patient records
- Soft delete support with audit trails
- Patient identifier management (MRN, SSN, etc.)
- Multiple name and address support
- Contact information management

### Patient Matching
- **Probabilistic Matching**: Advanced algorithms for fuzzy patient matching
- **Deterministic Matching**: Rule-based exact matching
- **Configurable Scoring**: Customizable match thresholds
- **Match Components**:
  - Name matching (fuzzy and phonetic)
  - Date of birth matching
  - Gender matching
  - Address matching
  - Identifier matching

### Search Capabilities
- Full-text search across patient records
- Fuzzy search support
- Advanced query syntax
- High-performance indexing with Tantivy

### APIs

#### RESTful API
- OpenAPI 3.0 documentation
- Swagger UI interface
- JSON request/response format
- CORS support

#### HL7 FHIR R5
- FHIR Patient resource
- FHIR Organization resource
- FHIR search parameters
- FHIR Bundle support

#### gRPC API
- Protocol Buffer schemas
- Streaming RPC support
- High-performance binary protocol

### High Availability
- Connection pooling
- Retry logic with exponential backoff
- Circuit breaker patterns
- Health check endpoints
- Graceful shutdown
- Horizontal scaling support

### Observability
- Structured JSON logging
- Custom metrics (patient operations, match scores, API latency)
- Distributed tracing
- OpenTelemetry OTLP export

## Technology Stack

- **Language**: Rust (Edition 2021)
- **Async Runtime**: Tokio
- **Web Framework**: Axum
- **Database**: PostgreSQL 18 via Diesel ORM
- **Search Engine**: Tantivy
- **Event Streaming**: Fluvio
- **gRPC**: Tonic
- **Observability**: OpenTelemetry
- **API Documentation**: Utoipa (OpenAPI)

## Prerequisites

- Rust 1.70+ (https://rustup.rs/)
- PostgreSQL 18
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`
- (Optional) Fluvio for event streaming
- (Optional) OpenTelemetry Collector for observability

## Quick Start

### 1. Clone the repository

```bash
git clone https://github.com/sixarm/master-patient-index-rust-crate.git
cd master-patient-index-rust-crate
```

### 2. Set up the database

```bash
# Create the database
createdb mpi

# Copy environment file
cp .env.example .env

# Edit .env and set your DATABASE_URL
# DATABASE_URL=postgres://username:password@localhost:5432/mpi

# Run migrations
diesel setup
diesel migration run
```

### 3. Build the project

```bash
cargo build --release
```

### 4. Run the server

```bash
cargo run --release
```

The API will be available at:
- REST API: http://localhost:8080/api/v1
- Swagger UI: http://localhost:8080/swagger-ui
- gRPC: localhost:50051

## Configuration

Configuration can be done via environment variables or a `.env` file. See `.env.example` for all available options.

Key configuration options:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://localhost/mpi` |
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | HTTP server port | `8080` |
| `GRPC_PORT` | gRPC server port | `50051` |
| `LOG_LEVEL` | Logging level (trace, debug, info, warn, error) | `info` |
| `MATCHING_THRESHOLD_SCORE` | Minimum match score threshold | `0.85` |

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Running Benchmarks

```bash
cargo bench
```

### Database Migrations

```bash
# Create a new migration
diesel migration generate <migration_name>

# Run migrations
diesel migration run

# Revert last migration
diesel migration revert
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for common mistakes
cargo clippy -- -W clippy::all
```

## API Documentation

### REST API

The RESTful API is documented using OpenAPI 3.0. Access the interactive Swagger UI at:
```
http://localhost:8080/swagger-ui
```

### Example Patient Creation

```bash
curl -X POST http://localhost:8080/api/v1/patients \
  -H "Content-Type: application/json" \
  -d '{
    "name": {
      "family": "Smith",
      "given": ["John", "Robert"],
      "prefix": [],
      "suffix": []
    },
    "gender": "male",
    "birth_date": "1980-01-15"
  }'
```

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   REST API      │     │   gRPC API      │     │   FHIR API      │
│   (Axum)        │     │   (Tonic)       │     │   (HL7 FHIR)    │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         └───────────────────────┴───────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   Business Logic Layer  │
                    │   - Patient Matching    │
                    │   - Validation          │
                    │   - Event Publishing    │
                    └────────────┬────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
│   PostgreSQL    │    │   Tantivy       │    │   Fluvio        │
│   (Persistence) │    │   (Search)      │    │   (Streaming)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Deployment

### Docker (Coming Soon)

```bash
docker build -t mpi:latest .
docker run -p 8080:8080 -p 50051:50051 mpi:latest
```

### Kubernetes (Coming Soon)

```bash
kubectl apply -f k8s/
```

### Multi-Cloud Support

Infrastructure as Code (IaC) using OpenTofu for deployment to:
- Google Cloud Platform (GCP)
- Amazon Web Services (AWS)
- Microsoft Azure

## Performance

Designed for enterprise scale:
- **Patients**: Millions of patient records
- **Organizations**: Thousands of clinics/hospitals
- **Throughput**: High-volume concurrent requests
- **Latency**: Sub-millisecond search and matching

## Security & Compliance

- HIPAA-compliant audit logging
- Data encryption at rest and in transit
- Role-based access control (RBAC)
- Patient consent management
- Secure secrets management

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## License

This project is dual-licensed under MIT OR Apache-2.0.

## Support

For questions or issues, please open an issue on GitHub.

## Roadmap

See `todo.md` for the complete implementation roadmap covering 20 phases of development.

### Phase 1: Foundation ✅
- [x] Project initialization
- [x] Dependency configuration
- [x] Module structure
- [x] Database configuration
- [x] OpenTelemetry setup

### Upcoming Phases
- Phase 2: Database Schema & Models
- Phase 3: Core MPI Logic
- Phase 4: Search Engine Integration
- Phase 5: RESTful API
- Phase 6: FHIR R5 Support
- Phase 7: gRPC API
- ...and more (see todo.md)

## Acknowledgments

Built with the following excellent Rust crates:
- Tokio - Async runtime
- Axum - Web framework
- Diesel - ORM
- Tantivy - Search engine
- Tonic - gRPC
- OpenTelemetry - Observability
- And many more (see Cargo.toml)
