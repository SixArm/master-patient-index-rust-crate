You are a software programming expert. Create source code for Master Patient Index (MPI). Think deeply. Ask questions.

Create a file "todo.md" with tasks.

Application:
- Programming language: Rust <https://rust-lang.org/>
- Asynchronous runtime: Tokio <https://docs.rs/tokio/latest/tokio/> <https://github.com/tokio-rs/tokio>
- Search engine: Tantivy  <https://docs.rs/tantivy/latest/tantivy/> <https://github.com/quickwit-oss/tantivy>
- Observability: OpenTelemtry logs metrics traces <https://docs.rs/opentelemetry/latest/opentelemetry/> <https://github.com/open-telemetry/opentelemetry-rust>

Data:
- Database: PostgreSQL 18 <https://www.postgresql.org/>
- Database ORM: Diesel <https://docs.rs/diesel/latest/diesel/> <https://github.com/diesel-rs/diesel>
- Data streaming: Fluvio  <https://docs.rs/fluvio/latest/fluvio/> <https://github.com/fluvio-community/fluvio>

API:
- hTTP: Hyper <https://docs.rs/hyper/latest/hyper> <https://github.com/hyperium/hyper>
- RESTful: Axum web aplication framework <https://docs.rs/axum/latest/axum/> <https://github.com/tokio-rs/axum>
- JSON: Serde JSON <https://docs.rs/serde_json/latest/serde_json/index.html> <https://github.com/serde-rs/json>
- OpenAPI v3 <https://docs.rs/openapiv3/latest/openapiv3/> <https://github.com/glademiller/openapiv3>
- OpenAPI doc: Utoipa <https://docs.rs/utoipa/latest/utoipa/> <https://github.com/juhaku/utoipa>
- HL7 FHIR R5
- gRPC: Tonic  <https://docs.rs/tonic/latest/tonic/> <https://github.com/hyperium/tonic>

Testing:
- Unit testing: Assertables <https://docs.rs/assertables/latest/assertables/> <https://github.com/sixarm/assertables-rust-crate>
- Benchmark testing: Criterion  <https://github.com/criterion-rs/criterion.rs> <https://docs.rs/criterion/0.8.1/criterion/>
- Mutation testing: cargo-mutants <https://docs.rs/crate/cargo-mutants/latest> <https://github.com/sourcefrog/cargo-mutants>

Deployment:
- Infrastructure as Code: OpenTofu  <https://opentofu.org>
- Multi-cloud deployments: Google Cloud, Amazon Cloud, Microsoft Cloud

Full-scale production system:
- Millions of patients
- Thousands of clinics
- High availability disaster recovery (HADR)
- Fault tolerance

After each task, write a comprehensive synopsis that describes the task, goal, purpose to file <task-#.md>
