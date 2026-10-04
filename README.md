# TomTom Orbis Map Enhancement

## Overview
This repository focuses on enhancements and applications built around **TomTom Orbis Maps (Next-Gen Mapping)**. Orbis is TomTom’s newest unified mapping standard that combines open-source data (such as Overture Maps) with TomTom’s proprietary data to deliver rich, 3D maps ranging from the road down to the lane level.

## How It Works
TomTom Orbis Maps creates a unified, highly detailed mapping experience by:
- Integrating **open-source data** (e.g., Overture Maps).
- Merging it with **TomTom's proprietary mapping data**.
- Generating comprehensive **3D maps** that provide road-to-lane level accuracy.

## The Computer Vision (CV) Application
Orbis heavily relies on advanced Computer Vision techniques to process petabytes of probe data and imagery. The system uses automated algorithms to directly extract critical mapping features from raw visual data, including:
- Fine-grained road geometries
- Lane dividers
- Turn restrictions
- Complex intersection topologies

---
*Built for next-generation mapping technologies and computer vision applications.*

I audited the repository and checked the current TomTom Orbis and Overture documentation. The important finding is that this repository is presently a **green-field foundation**, not an existing implementation: `main` contains only `README.md` and `LICENSE`, with two commits. [Open the repository](https://github.com/KumarKumar89/TomTom-Orbis-Map-Enhancement?utm_source=chatgpt.com)

That is actually a very good position for this project: we can establish the architecture correctly before technical debt appears.

TomTom currently describes Orbis as a continuously updated map platform combining Overture, OSM, TomTom proprietary content, partner/sensor observations, and customer data. TomTom also explicitly warns that Orbis has a different data model/API surface from older TomTom Maps APIs, so I would make the Orbis boundary a first-class architectural component. ([TomTom Documentation][1]) Overture's transportation model is especially relevant because it represents **segments + connectors**, with support for access restrictions, destinations, and other road semantics. ([Overture Maps][2])

# 1. Recommended complete architecture

## Target system

```text
                    ┌─────────────────────────────────────────┐
                    │           EXTERNAL DATA SOURCES         │
                    ├─────────────────────────────────────────┤
                    │ TomTom Orbis APIs / PBF / FGDB           │
                    │ Overture Maps                            │
                    │ OpenStreetMap                            │
                    │ Probe / GPS / sensor observations        │
                    │ Street imagery / video / LiDAR           │
                    │ Customer / proprietary map layers        │
                    └───────────────────┬─────────────────────┘
                                        │
                                        ▼
              ┌───────────────────────────────────────────────────┐
              │             DATA ACQUISITION LAYER                 │
              │                                                    │
              │ Python: API clients / ETL / scheduling             │
              │ Rust: high-throughput download / parsing / IO      │
              │ checksum + provenance + source licensing metadata  │
              └──────────────────────┬────────────────────────────┘
                                     │
                                     ▼
              ┌───────────────────────────────────────────────────┐
              │              RAW / LANDING DATA LAKE              │
              │                                                   │
              │ Object Storage: S3 / Azure Blob / MinIO           │
              │ PBF / GeoParquet / images / metadata / manifests  │
              └──────────────────────┬────────────────────────────┘
                                     │
                                     ▼
        ┌────────────────────────────────────────────────────────────────┐
        │                CANONICAL GEO-DATA MODEL                         │
        │                                                                  │
        │ Overture-aligned feature contracts                               │
        │ Segment / Connector / Road / Restriction / Destination / etc.    │
        │ Pydantic v2 + JSON Schema + versioned data contracts             │
        └──────────────────────┬───────────────────────────────────────────┘
                               │
             ┌─────────────────┴───────────────────┐
             │                                     │
             ▼                                     ▼
┌──────────────────────────────┐       ┌─────────────────────────────────┐
│    COMPUTER VISION PLANE     │       │       GEO ANALYTICS PLANE       │
├──────────────────────────────┤       ├─────────────────────────────────┤
│ Python / PyTorch             │       │ Rust                            │
│ OpenCV                       │       │ geometry                        │
│ object detection             │       │ topology                        │
│ segmentation                 │       │ projections                     │
│ lane detection               │       │ routing graph                   │
│ road boundary detection      │       │ map matching                    │
│ sign / marking detection     │       │ spatial indexing                │
│ change detection             │       │ conflation                      │
└──────────────┬───────────────┘       └────────────────┬────────────────┘
               │                                        │
               └────────────────┬───────────────────────┘
                                ▼
                  ┌─────────────────────────────────┐
                  │     FEATURE EXTRACTION LAYER    │
                  │                                 │
                  │ lanes                           │
                  │ road geometries                 │
                  │ dividers                        │
                  │ signs / markings                │
                  │ intersections                   │
                  │ restrictions                    │
                  │ topology candidates             │
                  │ confidence / provenance         │
                  └────────────────┬────────────────┘
                                   │
                                   ▼
             ┌──────────────────────────────────────────────┐
             │       MAP FUSION / CONFLATION ENGINE        │
             ├──────────────────────────────────────────────┤
             │ Orbis ↔ Overture ↔ OSM ↔ CV observations     │
             │ geometry matching                            │
             │ entity resolution                            │
             │ duplicate detection                          │
             │ temporal reconciliation                      │
             │ confidence weighting                         │
             │ provenance tracking                          │
             └─────────────────────┬────────────────────────┘
                                   │
                                   ▼
             ┌──────────────────────────────────────────────┐
             │          TOPOLOGY / QUALITY ENGINE           │
             ├──────────────────────────────────────────────┤
             │ geometry validity                            │
             │ segment↔connector consistency                │
             │ intersection correctness                      │
             │ lane continuity                              │
             │ connectivity                                │
             │ restriction consistency                      │
             │ spatial anomalies                             │
             │ schema validation                            │
             │ regression detection                         │
             └─────────────────────┬────────────────────────┘
                                   │
                      ┌────────────┴────────────┐
                      │                         │
                      ▼                         ▼
            ┌─────────────────┐       ┌─────────────────────┐
            │ HUMAN QA /      │       │ AUTOMATED RELEASE   │
            │ EXPERT REVIEW   │       │ QUALITY GATE        │
            └────────┬────────┘       └──────────┬──────────┘
                     │                           │
                     └────────────┬──────────────┘
                                  ▼
             ┌─────────────────────────────────────────────┐
             │              ENHANCED MAP STORE              │
             │                                             │
             │ PostGIS                                    │
             │ GeoParquet                                 │
             │ Object storage                             │
             │ versioned map artifacts                    │
             │ feature lineage / provenance               │
             └──────────────────────┬──────────────────────┘
                                    │
                ┌───────────────────┼────────────────────┐
                │                   │                    │
                ▼                   ▼                    ▼
        REST / API service     Tile generation       Analytics
        Axum / FastAPI         MVT / vector tiles    QA / reporting
                │
                ▼
       ┌─────────────────────┐
       │ MapLibre / Web UI    │
       │ 2D/3D visualization  │
       │ diff viewer          │
       │ confidence overlays  │
       │ before/after map     │
       └─────────────────────┘
```

## Architectural principle

I would **not** build this as dozens of microservices initially.

Build it as a **modular monorepo with two execution planes**:

**Rust = deterministic, performance-critical geospatial engine**

**Python = AI/ML, data engineering, orchestration and experimentation**

Then introduce services only where scaling actually requires them.

That gives you much lower complexity while retaining a path to distributed execution.

---

# 2. Rust + Python responsibility split

This is one of the most important design decisions.

| Component                     |        Rust |      Python |
| ----------------------------- | ----------: | ----------: |
| API ingestion                 |           ✓ |           ✓ |
| PBF parsing                   | **Primary** |             |
| GeoJSON/GeoParquet processing | **Primary** |           ✓ |
| geometry operations           | **Primary** |             |
| spatial indexing              | **Primary** |             |
| topology engine               | **Primary** |             |
| graph algorithms              | **Primary** |             |
| map matching                  | **Primary** |           ✓ |
| conflation                    | **Primary** |           ✓ |
| tile generation               | **Primary** |             |
| deterministic validation      | **Primary** |           ✓ |
| CV training                   |             | **Primary** |
| CV inference experiments      |             | **Primary** |
| PyTorch                       |             | **Primary** |
| OpenCV                        |             | **Primary** |
| data science                  |             | **Primary** |
| model evaluation              |             | **Primary** |
| pipeline orchestration        |             | **Primary** |
| REST API                      |           ✓ |           ✓ |
| CLI                           |           ✓ |           ✓ |
| benchmarking                  | **Primary** | **Primary** |

The bridge between them should be:

```text
Python
   │
   │ PyO3 / maturin
   ▼
Rust high-performance engine
   │
   ▼
geometry / topology / indexing / conflation
```

This is much better than rewriting everything in one language.

---

# 3. Recommended repository structure

I would evolve the repository toward this:

```text
TomTom-Orbis-Map-Enhancement/
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml
│   │   ├── security.yml
│   │   ├── dependency.yml
│   │   ├── benchmark.yml
│   │   ├── data-validation.yml
│   │   ├── build-release.yml
│   │   └── deploy.yml
│   │
│   ├── CODEOWNERS
│   ├── dependabot.yml
│   └── pull_request_template.md
│
├── rust/
│   ├── Cargo.toml
│   │
│   ├── crates/
│   │   ├── geo-core/
│   │   ├── geo-geometry/
│   │   ├── spatial-index/
│   │   ├── topology-engine/
│   │   ├── routing-graph/
│   │   ├── map-matching/
│   │   ├── conflation-engine/
│   │   ├── tile-engine/
│   │   ├── validation-engine/
│   │   ├── orbis-client/
│   │   ├── data-format/
│   │   ├── py-bridge/
│   │   └── cli/
│   │
│   └── benches/
│
├── python/
│   ├── pyproject.toml
│   │
│   ├── orbis/
│   │   ├── ingestion/
│   │   ├── clients/
│   │   ├── schemas/
│   │   ├── normalization/
│   │   ├── pipelines/
│   │   └── provenance/
│   │
│   ├── cv/
│   │   ├── datasets/
│   │   ├── preprocessing/
│   │   ├── models/
│   │   ├── inference/
│   │   ├── postprocessing/
│   │   └── change_detection/
│   │
│   ├── fusion/
│   │   ├── conflation/
│   │   ├── entity_resolution/
│   │   ├── confidence/
│   │   └── temporal/
│   │
│   ├── evaluation/
│   │   ├── accuracy/
│   │   ├── map_quality/
│   │   ├── regression/
│   │   └── golden_sets/
│   │
│   └── services/
│       └── api/
│
├── schemas/
│   ├── orbis/
│   ├── transportation/
│   ├── map-feature.schema.json
│   └── provenance.schema.json
│
├── configs/
│   ├── development/
│   ├── staging/
│   └── production/
│
├── data/
│   ├── samples/
│   ├── fixtures/
│   └── golden/
│
├── tests/
│   ├── rust/
│   ├── python/
│   ├── integration/
│   ├── geospatial/
│   ├── contract/
│   └── end_to_end/
│
├── benchmarks/
│   ├── rust/
│   ├── python/
│   ├── geospatial/
│   └── regression/
│
├── infra/
│   ├── docker/
│   ├── terraform/
│   ├── kubernetes/
│   └── policies/
│
├── docs/
│   ├── architecture/
│   ├── data-model/
│   ├── algorithms/
│   ├── api/
│   ├── operations/
│   └── security/
│
├── Dockerfile
├── docker-compose.yml
├── Makefile
├── pyproject.toml
├── rust-toolchain.toml
├── LICENSE
└── README.md
```

---

# 4. Required Rust codebases

## `geo-core`

The foundational type system.

```text
Coordinate
BoundingBox
LineString
Polygon
Segment
Connector
Lane
Intersection
Restriction
Road
FeatureId
MapVersion
SourceId
Timestamp
Confidence
```

The key objective is to ensure geometry and feature structures are **strongly typed**.

---

## `geometry-engine`

Responsibilities:

```text
projection
distance
bearing
heading
buffer
intersection
simplification
snapping
polyline operations
geometry normalization
Hausdorff distance
Fréchet-style similarity where required
```

Because Orbis Map Display uses EPSG:3857 and tile indexing, coordinate transformation and tile math should be centralized rather than scattered through the application. ([TomTom Documentation][3])

---

## `spatial-index`

Use:

```text
R-tree
STRtree-style indexing
H3 where advantageous
geohash where appropriate
tile index
bounding-box queries
nearest-neighbour search
```

The goal is to avoid O(N²) spatial matching.

---

## `topology-engine`

This should become one of the project's most important modules.

Examples:

```text
segment -> connector
connector -> segments
lane -> road
lane -> intersection
restriction -> segment
restriction -> connector
destination -> transition
```

Then implement invariants such as:

```text
No disconnected road graph
No impossible turn
No orphan connector
No self-intersection where forbidden
No invalid geometry
No duplicate canonical feature
```

Overture explicitly models connectors as physical connection points between segments, making this a natural validation abstraction. ([Overture Maps][4])

---

# 5. `conflation-engine`

This is potentially the intellectual heart of the project.

Example:

```text
                CV road geometry
                       │
                       ▼
             ┌───────────────────┐
             │ candidate search  │
             └─────────┬─────────┘
                       │
         ┌─────────────┼────────────┐
         ▼             ▼            ▼
     geometry       topology      metadata
       score          score          score
         │             │            │
         └─────────────┼────────────┘
                       ▼
                 global score
                       │
                       ▼
             accept / reject / review
```

Matching signals:

```text
geometric similarity
direction / heading
topology compatibility
distance
lane count
road classification
intersection context
temporal proximity
source reliability
CV confidence
```

The output should contain **both the result and why the result was accepted**.

---

# 6. Python computer-vision codebase

Recommended structure:

```text
cv/
├── datasets/
├── preprocessing/
├── models/
│   ├── road_detection/
│   ├── lane_detection/
│   ├── marking_detection/
│   ├── sign_detection/
│   ├── intersection/
│   └── change_detection/
├── inference/
├── postprocessing/
└── evaluation/
```

Recommended technology:

```text
Python
PyTorch
TorchVision
OpenCV
NumPy
SciPy
Albumentations
ONNX
ONNX Runtime
```

A good production flow is:

```text
PyTorch training
      ↓
model validation
      ↓
ONNX export
      ↓
Rust/Python inference
      ↓
feature extraction
      ↓
geospatial fusion
```

This gives you flexibility during research without forcing the production engine to remain Python-only.

---

# 7. Data architecture

I recommend a **lake + relational geospatial index** design.

### Object storage

```text
S3 / Azure Blob / MinIO
```

Store:

```text
raw PBF
raw API responses
imagery
GeoParquet
model artifacts
intermediate results
release snapshots
golden datasets
```

### PostGIS

Use for:

```text
feature queries
spatial relationships
QA
investigation
API queries
human review
change visualization
```

### DuckDB

Excellent for:

```text
analytical queries
GeoParquet inspection
CI regression tests
developer workflows
large dataset sampling
```

Overture itself documents access through object storage and tools such as DuckDB, and its current schema documentation is Pydantic-based. ([Overture Maps][5])

---

# 8. Canonical data contract

I strongly recommend a versioned internal contract.

```text
MapFeature
 ├── id
 ├── version
 ├── geometry
 ├── bbox
 ├── feature_type
 ├── properties
 ├── source[]
 ├── confidence
 ├── observation_time
 ├── processing_version
 └── provenance
```

Then:

```text
Orbis source
       ↓
adapter
       ↓
Canonical MapFeature
       ↑
Overture adapter
       ↑
OSM adapter
       ↑
CV adapter
       ↑
customer-data adapter
```

Do **not** allow individual modules to invent their own feature representation.

Because the current Overture schema has moved to a Pydantic-authored v2 model and its YAML source representation is being deprecated, I would pin the schema version and generate the machine-readable contract from the chosen version rather than copying schema fragments manually. ([Overture Maps][5])

---

# 9. Recommended technology stack

| Layer                  | Recommended technology                 |
| ---------------------- | -------------------------------------- |
| Core high performance  | **Rust**                               |
| Python ML/data         | **Python**                             |
| Rust/Python bridge     | **PyO3 + maturin**                     |
| Python package manager | **uv**                                 |
| API                    | **Axum + FastAPI**                     |
| CV                     | **PyTorch + OpenCV**                   |
| Model deployment       | **ONNX Runtime**                       |
| Numerical              | **NumPy / SciPy**                      |
| Geometry               | **GEOS / PROJ + Rust geo ecosystem**   |
| Geospatial DB          | **PostGIS**                            |
| Analytical engine      | **DuckDB**                             |
| Data format            | **GeoParquet + Apache Arrow**          |
| Object storage         | **S3 / Azure Blob / MinIO**            |
| Streaming, later       | **Kafka / Redpanda**                   |
| Cache                  | **Redis**                              |
| Frontend               | **TypeScript + MapLibre GL JS**        |
| Visualization          | **deck.gl**                            |
| Containers             | **Docker / BuildKit**                  |
| Deployment             | **Kubernetes**, when scale requires it |
| IaC                    | **Terraform**                          |
| Metrics                | **Prometheus**                         |
| Dashboards             | **Grafana**                            |
| Logs                   | **OpenTelemetry**                      |
| ML experiment tracking | **MLflow**                             |
| Dataset versioning     | **DVC or object-store manifests**      |

For the first implementation, I would keep Kafka, Kubernetes and Redis optional rather than mandatory.

---

# 10. GitHub Actions CI/CD architecture

I would build **separate pipelines**, not one giant workflow.

```text
                   Git Push / Pull Request
                              │
                              ▼
                    ┌─────────────────────┐
                    │   PR QUALITY GATE   │
                    └─────────┬───────────┘
                              │
          ┌───────────────────┼──────────────────────┐
          ▼                   ▼                      ▼
     Python CI            Rust CI                Security
          │                   │                      │
     Ruff/mypy             fmt/clippy           CodeQL
     pytest                nextest              dependency review
     coverage              cargo test            secrets
          │                   │                   action security
          └───────────────────┼──────────────────────┘
                              ▼
                     Contract / Geo Tests
                              │
                              ▼
                     Performance Gate
                              │
                              ▼
                         BUILD IMAGE
                              │
                              ▼
                    SBOM + vulnerability scan
                              │
                              ▼
                      artifact attestation
                              │
                              ▼
                     publish container
                              │
                              ▼
                      STAGING DEPLOY
                              │
                              ▼
                      integration tests
                              │
                              ▼
                        PROD GATE
                              │
                              ▼
                       PRODUCTION
```

---

# 11. GitHub Actions tools I would use

## Tier 1 — absolutely recommended

### GitHub-native

**GitHub CodeQL**

Static security analysis for the codebase.

**Dependency Review**

Prevent vulnerable dependency changes entering pull requests. GitHub says the dependency-review action can fail a check when vulnerable packages are introduced, making it useful as a merge gate. ([GitHub Docs][6])

**Dependabot**

For:

```text
Cargo
Python
GitHub Actions
Docker
other supported dependencies
```

GitHub documents using Actions to automate Dependabot pull requests and their review process. ([GitHub Docs][7])

**Secret scanning / push protection**

Particularly important because TomTom API keys and cloud credentials must never enter the repository. GitHub documents secret scanning and push protection for preventing credential leaks. ([GitHub Docs][8])

---

# 12. Rust CI toolchain

I would use:

```text
cargo fmt --check
cargo clippy
cargo test
cargo nextest
cargo audit
cargo deny
cargo check
cargo llvm-cov
criterion
proptest
```

### Why these matter

`cargo fmt`

Prevents formatting drift.

`clippy`

Catches Rust correctness and quality problems.

`nextest`

Faster and more structured test execution.

`cargo-audit`

Dependency vulnerability detection.

`cargo-deny`

Licensing + dependency-policy enforcement.

`proptest`

Extremely valuable for geometry/topology algorithms because it can generate pathological inputs.

`criterion`

Performance regression detection.

---

# 13. Python CI toolchain

I recommend:

```text
ruff
mypy
pytest
pytest-cov
hypothesis
bandit
pip-audit
pytest-benchmark
```

And:

```text
uv
```

for fast reproducible environment setup.

CI should reject:

```text
lint errors
type errors
test failures
coverage regression
dependency vulnerabilities
API/schema incompatibilities
performance regression
```

---

# 14. Geospatial-specific CI

This project needs a specialized test layer.

Normal unit tests are not sufficient.

Create a:

```text
geospatial-regression
```

workflow.

Test:

```text
projection correctness
coordinate conversion
tile boundaries
geometry validity
segment connectivity
connector integrity
intersection topology
route continuity
conflation accuracy
spatial index correctness
duplicate detection
map version consistency
```

Example:

```text
Input map A
     │
     ▼
enhancement pipeline
     │
     ▼
Output map B
     │
     ├── geometry assertions
     ├── topology assertions
     ├── semantic assertions
     ├── schema assertions
     └── golden-data comparison
```

---

# 15. Performance CI

This is where the Rust/Python architecture becomes valuable.

Set hard regression budgets.

Example:

| Benchmark            |              Gate |
| -------------------- | ----------------: |
| Geometry operation   | no >5% regression |
| Spatial lookup       |            no >5% |
| Conflation           |            no >5% |
| PBF parsing          |            no >5% |
| Tile generation      |            no >5% |
| API p95              |           no >10% |
| Memory               |           no >10% |
| CV inference         |    model-specific |
| Full sample pipeline |           no >10% |

Use:

```text
Criterion.rs
pytest-benchmark
hyperfine
memory profiler
```

Do not make "performance" just a dashboard metric.

Make it a **merge gate**.

---

# 16. Security tools I strongly recommend

Beyond CodeQL:

```text
zizmor
actionlint
Gitleaks
Trivy
Syft
Grype
OSV-Scanner
cargo-audit
cargo-deny
pip-audit
Bandit
```

### Particularly important: `zizmor`

Use this to inspect GitHub Actions workflows themselves.

That means your CI pipeline gets security-tested too.

GitHub itself recommends pinning third-party Actions to a **full-length commit SHA** because this provides an immutable reference and reduces supply-chain risk. ([GitHub Docs][9])

So instead of:

```yaml
uses: some/action@v4
```

the hardened production model should use a verified immutable commit SHA.

---

# 17. Container security

The build chain should be:

```text
source
   ↓
Docker BuildKit
   ↓
minimal image
   ↓
Trivy
   ↓
SBOM
   ↓
license scan
   ↓
image signing
   ↓
artifact attestation
   ↓
registry
```

GitHub artifact attestations provide cryptographically signed provenance and can include an SBOM. GitHub documents Sigstore-backed attestations for establishing build provenance and integrity. ([GitHub Docs][10])

This is something I would include from the beginning rather than bolt on later.

---

# 18. GitHub Actions caching

Use caching aggressively for:

```text
Cargo registry
Cargo git
Cargo target
Python wheels
uv cache
Docker layers
```

GitHub documents built-in caching support and specifically notes caching of Rust Cargo directories and Python dependencies. ([GitHub Docs][11])

But be careful with untrusted pull requests: GitHub now provides cache access modes, and low-trust jobs should not unnecessarily receive cache-write privileges because of cache-poisoning risk. ([GitHub Docs][12])

---

# 19. Cloud deployment security

For AWS/Azure/GCP deployment:

```text
GitHub Actions
      │
      ▼
GitHub OIDC
      │
      ▼
short-lived cloud credential
      │
      ▼
deploy
```

Do **not** store long-lived cloud access keys in GitHub Secrets when OIDC is available.

GitHub explicitly recommends OIDC for short-lived cloud credentials, with trust conditions restricting which repositories/workflows can obtain the credentials. ([GitHub Docs][13])

This is especially appropriate when moving the system into:

```text
AWS
Azure
GCP
```

---

# 20. TomTom API key handling

The TomTom key is different from your cloud identity.

Use:

```text
GitHub environment secret
        OR
AWS Secrets Manager
Azure Key Vault
GCP Secret Manager
```

and inject it only at runtime.

Never:

```text
README
source code
Dockerfile
test fixtures
Git history
workflow YAML
logs
benchmark artifacts
```

The Orbis Map Display API currently has public-preview documentation, and TomTom exposes vector/raster tile services through the Orbis Map Display API. ([TomTom Documentation][14])

---

# 21. GitHub Actions workflow set

I would create these workflows:

### `ci.yml`

```text
Python lint
Python type check
Python unit tests
Rust fmt
Rust clippy
Rust tests
integration tests
schema tests
```

### `security.yml`

```text
CodeQL
Gitleaks
Bandit
cargo audit
pip audit
Trivy
OSV
zizmor
actionlint
```

### `dependency.yml`

```text
Dependabot
dependency review
license policy
cargo-deny
```

### `geo-validation.yml`

```text
GeoParquet tests
geometry tests
topology tests
conflation tests
golden dataset tests
schema compatibility
```

### `benchmark.yml`

```text
Criterion
pytest-benchmark
memory
throughput
latency
regression threshold
```

### `build-release.yml`

```text
Rust binaries
Python wheels
Docker image
SBOM
attestation
GitHub release
```

### `deploy.yml`

```text
staging
smoke tests
approval
production
post-deployment validation
```

### `nightly.yml`

This is very important.

Run the expensive workloads here:

```text
large geospatial regression
CV model regression
large golden datasets
fuzzing
performance benchmark
full integration suite
dependency scans
data-schema drift
Orbis integration health checks
```

---

# 22. One more important component: Data/Model Drift

For this project, CI/CD cannot be only **code CI/CD**.

You need:

```text
Code CI/CD
+
Data CI/CD
+
Model CI/CD
```

### Data drift

Detect:

```text
schema change
feature distribution change
geometry distribution change
source availability
coverage changes
attribute frequency changes
```

### Model drift

Track:

```text
precision
recall
F1
IoU
lane detection accuracy
road detection accuracy
false positive rate
false negative rate
confidence calibration
```

### Map-quality drift

Track:

```text
topology errors
duplicate features
orphan features
invalid geometries
conflation errors
routing regressions
change-detection quality
```

This creates a much stronger system than ordinary software CI.

---

# 23. Production deployment model

I'd ultimately target:

```text
                 ┌──────────────┐
                 │ GitHub       │
                 │ Actions      │
                 └──────┬───────┘
                        │
                        ▼
                  Container Registry
                        │
                        ▼
                   Kubernetes
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
   API Service     Processing Jobs     CV Workers
        │               │                │
        └───────────────┼────────────────┘
                        ▼
                  PostGIS / Data Lake
                        │
                        ▼
                 Map / Analytics UI
```

For CV workloads requiring GPUs, I would use **dedicated self-hosted GPU runners** for training/inference CI rather than forcing every PR workflow onto expensive GPU infrastructure.

---

# 24. The most important architectural distinction

I would divide the project into these **five engines**:

```text
1. ORBIS DATA ENGINE
   ingestion + schema + provenance

2. GEO ENGINE
   geometry + projection + spatial indexing

3. MAP INTELLIGENCE ENGINE
   CV + change detection + feature extraction

4. CONFLATION ENGINE
   Orbis + Overture + OSM + CV fusion

5. QUALITY ENGINE
   topology + validation + scoring + regression
```

And then surround them with:

```text
API
CLI
UI
Observability
Security
CI/CD
```

That gives the repository a very clean engineering boundary.

# 25. My recommended implementation order

Do **not** begin by writing 50 modules.

Build in this sequence:

```text
PHASE 1
Repository foundation
        ↓
Rust workspace
Python package
schema contracts
CI baseline

PHASE 2
Orbis ingestion
Overture ingestion
canonical feature model
GeoParquet

PHASE 3
Rust geometry engine
spatial index
topology engine

PHASE 4
CV pipeline
road/lane/marking detection
model evaluation

PHASE 5
conflation engine
confidence scoring
temporal reconciliation

PHASE 6
quality engine
golden datasets
regression testing

PHASE 7
API + MapLibre UI

PHASE 8
performance optimization
containerization
deployment

PHASE 9
full production CI/CD
SBOM
attestation
OIDC
observability
DR
```

## The target KPI model

I would make the project's engineering scorecard:

```text
                 TOMTOM ORBIS ENHANCEMENT
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
     QUALITY           PERFORMANCE         RELIABILITY
        │                  │                  │
 geometry accuracy     throughput          zero crash
 topology accuracy     latency             reproducibility
 conflation accuracy   memory              deterministic tests
 CV accuracy            CPU/GPU use         rollback
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
                       SECURITY
                           │
                  supply-chain security
                  secrets protection
                  provenance
                  dependency security
```

The **highest-value technical idea** for this repository is therefore not simply “Python + Rust + CV.” It is a **deterministic geospatial core surrounded by an AI perception layer and a formally testable conflation/quality engine**.

That architecture is particularly well aligned with Orbis because Orbis is explicitly built around an interoperable Overture-style data model while incorporating proprietary and sensor-derived information. ([TomTom Documentation][1])

### Recommended baseline stack

```text
Rust
├── geo
├── geo-types
├── proj
├── GDAL bindings where required
├── rayon
├── tokio
├── axum
├── serde
├── arrow / parquet ecosystem
├── PyO3
└── criterion / proptest / nextest

Python
├── Python 3.12+
├── uv
├── Pydantic v2
├── FastAPI
├── NumPy
├── Pandas / Polars
├── GeoPandas
├── PyArrow
├── DuckDB
├── Shapely
├── PyTorch
├── OpenCV
├── ONNX Runtime
├── Ruff
├── mypy
└── pytest

GitHub Actions
├── CodeQL
├── Dependency Review
├── Dependabot
├── Secret Scanning
├── actionlint
├── zizmor
├── Gitleaks
├── Trivy
├── Syft
├── Grype
├── OSV-Scanner
├── cargo-audit
├── cargo-deny
├── pytest-benchmark
├── Criterion
└── artifact attestations
```

This is the architecture I would use as the **master blueprint for `TomTom-Orbis-Map-Enhancement`** before adding implementation code.

The next logical step is to turn this blueprint into the actual repository skeleton: `rust/`, `python/`, `schemas/`, tests, benchmarks, Docker, and the complete `.github/workflows/` CI/CD pipeline, with every component wired into the architecture above.

[1]: https://docs.tomtom.com/tomtom-orbis-maps/documentation/introduction?utm_source=chatgpt.com "Introduction | TomTom Orbis Maps"
[2]: https://docs.overturemaps.org/guides/transportation/?utm_source=chatgpt.com "Overview | Overture Documentation"
[3]: https://developer.tomtom.com/map-display-api/documentation/tomtom-orbis-maps/zoom-levels-and-tile-grid?utm_source=chatgpt.com "Zoom Levels and Tile Grid | Orbis Map Display API | TomTom Developer Portal"
[4]: https://docs.overturemaps.org/schema/reference/transportation/connector/?utm_source=chatgpt.com "Connector | Overture Documentation"
[5]: https://docs.overturemaps.org/schema/?utm_source=chatgpt.com "Schema Reference | Overture Documentation"
[6]: https://docs.github.com/en/code-security/concepts/supply-chain-security/dependency-review?utm_source=chatgpt.com "Dependency review - GitHub Docs"
[7]: https://docs.github.com/en/code-security/tutorials/secure-your-dependencies/automate-dependabot-with-actions?utm_source=chatgpt.com "Automating Dependabot with GitHub Actions - GitHub Docs"
[8]: https://docs.github.com/en/code-security/getting-started/github-security-features?utm_source=chatgpt.com "GitHub security features - GitHub Docs"
[9]: https://docs.github.com/en/actions/reference/security/secure-use?ref=marcinhoppe.com&utm_source=chatgpt.com "Secure use reference - GitHub Docs"
[10]: https://docs.github.com/en/actions/concepts/security/artifact-attestations?utm_source=chatgpt.com "Artifact attestations - GitHub Docs"
[11]: https://docs.github.com/en/actions/tutorials/build-and-test-code/rust?utm_source=chatgpt.com "Building and testing Rust - GitHub Docs"
[12]: https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching?apiVersion=2022-11-28&utm_source=chatgpt.com "Dependency caching reference - GitHub Docs"
[13]: https://docs.github.com/en/actions/reference/security/oidc?utm_source=chatgpt.com "OpenID Connect reference - GitHub Docs"
[14]: https://developer.tomtom.com/map-display-api/documentation/tomtom-orbis-maps/v1/product-information/introduction?utm_source=chatgpt.com "Introduction | Orbis Map Display API"

