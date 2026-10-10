# Orbis Map Enhancement — KPI Review Scorecard

*Run `20261010T050351Z` · pipeline `manual` · commit `unknown` · generated 2026-10-10T05:03:51Z*

## Executive summary

| Status | Count | Share |
|---|---:|---:|
| ✅ PASS | 99 | 99% |
| 🟡 WARN | 0 | 0% |
| 🔴 FAIL | 0 | 0% |
| ⚪ MISSING | 1 | 1% |

**CI/CD gate verdict:** ✅ GATE PASSED — 0 gated FAIL, 0 gated MISSING of 38 gated KPIs.

> Every row below is read as **Ideal Target vs Actual**. `Gap→Pass` is how far the actual has closed the distance between the warn and pass thresholds (100% = pass met).

## Category overview

| Category | KPIs | ✅ | 🟡 | 🔴 | ⚪ |
|---|---:|---:|---:|---:|---:|
| geospatial_core | 10 | 9 | 0 | 0 | 1 |
| topology | 6 | 6 | 0 | 0 | 0 |
| lanes | 5 | 5 | 0 | 0 | 0 |
| restrictions | 4 | 4 | 0 | 0 | 0 |
| conflation | 6 | 6 | 0 | 0 | 0 |
| change_detection | 5 | 5 | 0 | 0 | 0 |
| data_quality | 7 | 7 | 0 | 0 | 0 |
| computer_vision | 8 | 8 | 0 | 0 | 0 |
| routing | 5 | 5 | 0 | 0 | 0 |
| spatial_indexing | 4 | 4 | 0 | 0 | 0 |
| regression | 4 | 4 | 0 | 0 | 0 |
| orbis_integration | 4 | 4 | 0 | 0 | 0 |
| overture_integration | 4 | 4 | 0 | 0 | 0 |
| osm_integration | 3 | 3 | 0 | 0 | 0 |
| sensors_probe | 4 | 4 | 0 | 0 | 0 |
| rendering_tiles | 4 | 4 | 0 | 0 | 0 |
| coverage | 4 | 4 | 0 | 0 | 0 |
| freshness | 3 | 3 | 0 | 0 | 0 |
| reliability_security | 2 | 2 | 0 | 0 | 0 |
| infra_performance | 8 | 8 | 0 | 0 | 0 |

## geospatial_core

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-001 | Geometry Positional Accuracy RMSE | ↓ | m | 0.3 | 0.5 | 1 | **0.5** | ✅ PASS | 100% | — | 🔒 | geometry-engine | ci |
| KPI-002 | Horizontal Coordinate Error P95 | ↓ | m | 0.6 | 1 | 2 | **1** | ✅ PASS | 100% | — | 🔒 | geometry-engine | ci |
| KPI-003 | Projection Round-Trip Error 4326-3857 | ↓ | m | 0.001 | 0.01 | 0.05 | **0.01** | ✅ PASS | 100% | — | 🔒 | geo-geometry | ci |
| KPI-004 | Tile Boundary Alignment Error | ↓ | px | 0 | 0.25 | 0.5 | **0.25** | ✅ PASS | 100% | — |  | tile-engine | weekly |
| KPI-005 | Elevation Vertical Accuracy RMSE | ↓ | m | 0.5 | 1 | 2 | **1** | ✅ PASS | 100% | — |  | orbis-ingestion | monthly |
| KPI-006 | Curvature Representation Error | ↓ | pct | 1.00% | 2.00% | 4.00% | **—** | ⚪ MISSING | — | — |  | geometry-engine | weekly |
| KPI-007 | Address Point Placement Accuracy | ↑ | pct | 99.00% | 97.00% | 94.00% | **97.00%** | ✅ PASS | 100% | — |  | conflation-engine | monthly |
| KPI-008 | POI Coordinate Precision | ↑ | pct | 98.00% | 95.00% | 90.00% | **95.00%** | ✅ PASS | 100% | — |  | conflation-engine | monthly |
| KPI-009 | Geometry Normalization Idempotency | ↑ | pct | 100.00% | 100.00% | 99.90% | **100.00%** | ✅ PASS | 100% | — | 🔒 | validation-engine | ci |
| KPI-010 | Simplification Deviation RDP Tolerance | ↓ | m | 0.2 | 0.5 | 1 | **0.5** | ✅ PASS | 100% | — |  | geometry-engine | weekly |

## topology

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-011 | Segment-Connector Consistency Rate | ↑ | pct | 100.00% | 99.95% | 99.90% | **99.95%** | ✅ PASS | 100% | — | 🔒 | topology-engine | ci |
| KPI-012 | Orphan Connector Count | ↓ | count | 0 | 0 | 5 | **0** | ✅ PASS | 100% | — | 🔒 | topology-engine | ci |
| KPI-013 | Dangling Reference Count | ↓ | count | 0 | 0 | 3 | **0** | ✅ PASS | 100% | — | 🔒 | topology-engine | ci |
| KPI-014 | Intersection Topology Accuracy | ↑ | pct | 99.50% | 99.00% | 98.00% | **99.00%** | ✅ PASS | 100% | — |  | topology-engine | weekly |
| KPI-015 | Graph Connectivity Coverage | ↑ | pct | 100.00% | 99.90% | 99.50% | **99.90%** | ✅ PASS | 100% | — | 🔒 | routing-graph | ci |
| KPI-016 | Impossible Turn Rate | ↓ | per-10k | 0 | 0.5 | 2 | **0.5** | ✅ PASS | 100% | — |  | topology-engine | weekly |

## lanes

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-017 | Lane Count Accuracy | ↑ | pct | 99.00% | 97.00% | 94.00% | **97.00%** | ✅ PASS | 100% | — |  | cv-lane-detection | weekly |
| KPI-018 | Lane Divider Position Error P95 | ↓ | m | 0.3 | 0.5 | 0.8 | **0.5** | ✅ PASS | 100% | — |  | cv-lane-detection | weekly |
| KPI-019 | Lane Continuity Through Intersections | ↑ | pct | 98.00% | 95.00% | 90.00% | **95.00%** | ✅ PASS | 100% | — |  | topology-engine | weekly |
| KPI-020 | Lane Turn-Ban Restriction Accuracy | ↑ | pct | 98.00% | 95.00% | 92.00% | **95.00%** | ✅ PASS | 100% | — |  | conflation-engine | monthly |
| KPI-021 | Lane Width Estimation Error | ↓ | cm | 10 | 20 | 35 | **20** | ✅ PASS | 100% | — |  | cv-lane-detection | monthly |

## restrictions

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-022 | Turn Restriction Precision | ↑ | pct | 98.00% | 95.00% | 92.00% | **95.00%** | ✅ PASS | 100% | — |  | conflation-engine | weekly |
| KPI-023 | Turn Restriction Recall | ↑ | pct | 96.00% | 92.00% | 88.00% | **92.00%** | ✅ PASS | 100% | — |  | cv-sign-detection | weekly |
| KPI-024 | Access Property Attribute Accuracy | ↑ | pct | 98.00% | 95.00% | 90.00% | **95.00%** | ✅ PASS | 100% | — |  | orbis-ingestion | monthly |
| KPI-025 | Destination Info Accuracy | ↑ | pct | 97.00% | 94.00% | 90.00% | **94.00%** | ✅ PASS | 100% | — |  | conflation-engine | monthly |

## conflation

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-026 | Conflation Match Precision | ↑ | pct | 99.00% | 97.00% | 95.00% | **97.00%** | ✅ PASS | 100% | ↓ | 🔒 | conflation-engine | ci |
| KPI-027 | Conflation Match Recall | ↑ | pct | 97.00% | 94.00% | 90.00% | **94.00%** | ✅ PASS | 100% | — | 🔒 | conflation-engine | ci |
| KPI-028 | False Merge Rate | ↓ | per-1k | 0 | 1 | 3 | **1** | ✅ PASS | 100% | — | 🔒 | conflation-engine | ci |
| KPI-029 | Missed Merge Rate | ↓ | per-1k | 1 | 5 | 10 | **5** | ✅ PASS | 100% | — |  | conflation-engine | weekly |
| KPI-030 | Auto-Accept Decision Share | ↑ | pct | 90.00% | 80.00% | 70.00% | **80.00%** | ✅ PASS | 100% | — |  | conflation-engine | weekly |
| KPI-031 | Explanation Completeness Result Plus Why | ↑ | pct | 100.00% | 100.00% | 99.00% | **100.00%** | ✅ PASS | 100% | — | 🔒 | conflation-engine | ci |

## change_detection

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-032 | True Positive Change Detection Rate | ↑ | pct | 95.00% | 90.00% | 85.00% | **90.00%** | ✅ PASS | 100% | — |  | cv-change-detection | weekly |
| KPI-033 | Change Detection False Alarm Rate | ↓ | per-1k-img | 1 | 5 | 10 | **5** | ✅ PASS | 100% | — |  | cv-change-detection | weekly |
| KPI-034 | Change-to-Publication Latency P95 | ↓ | h | 24 | 72 | 168 | **72** | ✅ PASS | 100% | — |  | pipelines | monthly |
| KPI-035 | Revert Rate Wrongly Applied Changes | ↓ | per-1k | 0 | 2 | 5 | **2** | ✅ PASS | 100% | — |  | quality-engine | monthly |
| KPI-036 | Temporal Reconciliation Accuracy | ↑ | pct | 98.00% | 95.00% | 92.00% | **95.00%** | ✅ PASS | 100% | — |  | fusion-temporal | monthly |

## data_quality

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-037 | Schema Validation Pass Rate | ↑ | pct | 100.00% | 100.00% | 99.90% | **100.00%** | ✅ PASS | 100% | — | 🔒 | validation-engine | ci |
| KPI-038 | Provenance Completeness | ↑ | pct | 100.00% | 99.90% | 99.00% | **99.90%** | ✅ PASS | 100% | — | 🔒 | provenance | ci |
| KPI-039 | Duplicate Canonical Feature Rate | ↓ | per-10k | 0 | 1 | 3 | **1** | ✅ PASS | 100% | — |  | topology-engine | weekly |
| KPI-040 | Invalid Geometry Rate | ↓ | per-10k | 0 | 0.5 | 2 | **0.5** | ✅ PASS | 100% | — | 🔒 | validation-engine | ci |
| KPI-041 | Null Required Attribute Rate | ↓ | per-10k | 0 | 2 | 5 | **2** | ✅ PASS | 100% | — |  | ingestion | weekly |
| KPI-042 | Mean Feature Confidence Score | ↑ | pct | 95.00% | 90.00% | 85.00% | **90.00%** | ✅ PASS | 100% | — |  | validation-engine | weekly |
| KPI-043 | License Metadata Tagging Coverage | ↑ | pct | 100.00% | 100.00% | 99.50% | **100.00%** | ✅ PASS | 100% | — |  | ingestion | monthly |

## computer_vision

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-044 | Road Segmentation IoU | ↑ | pct | 88.00% | 82.00% | 78.00% | **82.00%** | ✅ PASS | 100% | — |  | cv-road-detection | weekly |
| KPI-045 | Lane Marking F1 | ↑ | pct | 90.00% | 85.00% | 80.00% | **85.00%** | ✅ PASS | 100% | — |  | cv-marking-detection | weekly |
| KPI-046 | Sign Detection mAP at 0.5 IoU | ↑ | pct | 85.00% | 78.00% | 72.00% | **78.00%** | ✅ PASS | 100% | — |  | cv-sign-detection | weekly |
| KPI-047 | Object Detection Recall | ↑ | pct | 92.00% | 88.00% | 84.00% | **88.00%** | ✅ PASS | 100% | — |  | cv-models | weekly |
| KPI-048 | CV False Positive Rate | ↓ | per-1k-frame | 2 | 8 | 15 | **8** | ✅ PASS | 100% | — |  | cv-evaluation | weekly |
| KPI-049 | Confidence Calibration ECE | ↓ | pct | 2.00% | 5.00% | 8.00% | **5.00%** | ✅ PASS | 100% | — |  | cv-evaluation | monthly |
| KPI-050 | CV Inference Throughput | ↑ | fps | 120 | 80 | 50 | **80** | ✅ PASS | 100% | — | 🔒 | inference-onnx | ci |
| KPI-051 | Golden Set Regression Delta | ↓ | pct | 0.00% | 0.50% | 1.50% | **0.50%** | ✅ PASS | 100% | — | 🔒 | evaluation-golden | ci |

## routing

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-052 | Route Continuity Across Versions | ↑ | pct | 100.00% | 99.90% | 99.50% | **99.90%** | ✅ PASS | 100% | ↓ | 🔒 | routing-graph | ci |
| KPI-053 | Travel Time Estimation MAE | ↓ | pct | 5.00% | 8.00% | 12.00% | **8.00%** | ✅ PASS | 100% | — |  | routing-graph | monthly |
| KPI-054 | Path Optimality Ratio vs Ground Truth | ↑ | pct | 99.00% | 97.00% | 95.00% | **97.00%** | ✅ PASS | 100% | — |  | routing-graph | weekly |
| KPI-055 | Dead-End Anomaly Rate | ↓ | per-10k | 0 | 1 | 3 | **1** | ✅ PASS | 100% | — |  | topology-engine | weekly |
| KPI-056 | Map Matching Trajectory Accuracy | ↑ | pct | 98.00% | 95.00% | 92.00% | **95.00%** | ✅ PASS | 100% | — |  | map-matching | weekly |

## spatial_indexing

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-057 | Spatial Query Correctness vs Brute Force | ↑ | pct | 100.00% | 100.00% | 99.99% | **100.00%** | ✅ PASS | 100% | → | 🔒 | spatial-index | ci |
| KPI-058 | Index Build Throughput | ↑ | feats/s | 5e+06 | 2e+06 | 1e+06 | **2e+06** | ✅ PASS | 100% | — | 🔒 | spatial-index | ci |
| KPI-059 | BBox Query Latency P99 | ↓ | ms | 1 | 5 | 10 | **5** | ✅ PASS | 100% | — | 🔒 | spatial-index | ci |
| KPI-060 | Nearest-Neighbour Exactness | ↑ | pct | 100.00% | 100.00% | 99.90% | **100.00%** | ✅ PASS | 100% | — |  | spatial-index | weekly |

## regression

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-061 | Golden Dataset Regression Pass Rate | ↑ | pct | 100.00% | 99.50% | 98.00% | **99.50%** | ✅ PASS | 100% | — | 🔒 | quality-engine | ci |
| KPI-062 | Version Diff Noise Rate | ↓ | per-10k | 0 | 2 | 5 | **2** | ✅ PASS | 100% | — |  | quality-engine | weekly |
| KPI-063 | Flaky Test Rate | ↓ | per-1k-runs | 0 | 2 | 5 | **2** | ✅ PASS | 100% | — |  | ci-harness | weekly |
| KPI-064 | Deterministic Replay Consistency | ↑ | pct | 100.00% | 100.00% | 99.90% | **100.00%** | ✅ PASS | 100% | — | 🔒 | ci-harness | ci |

## orbis_integration

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-065 | Orbis API Availability | ↑ | pct | 99.90% | 99.50% | 99.00% | **99.50%** | ✅ PASS | 100% | — |  | orbis-client | weekly |
| KPI-066 | Orbis Response Latency P95 | ↓ | ms | 150 | 300 | 500 | **300** | ✅ PASS | 100% | — | 🔒 | orbis-client | ci |
| KPI-067 | Orbis Schema Compatibility Rate | ↑ | pct | 100.00% | 100.00% | 99.00% | **100.00%** | ✅ PASS | 100% | — | 🔒 | ingestion | ci |
| KPI-068 | Orbis Data Freshness Lag | ↓ | h | 1 | 6 | 24 | **6** | ✅ PASS | 100% | — |  | orbis-ingestion | daily |

## overture_integration

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-069 | Overture Release Adoption Latency | ↓ | d | 3 | 7 | 14 | **7** | ✅ PASS | 100% | — |  | ingestion | monthly |
| KPI-070 | Overture Segment Connector Mapping Fidelity | ↑ | pct | 99.50% | 98.00% | 96.00% | **98.00%** | ✅ PASS | 100% | — |  | normalization | weekly |
| KPI-071 | Overture GeoParquet Read Success Rate | ↑ | pct | 100.00% | 99.90% | 99.50% | **99.90%** | ✅ PASS | 100% | — | 🔒 | data-format | ci |
| KPI-072 | Overture Schema Pin Drift Detected | ↓ | count | 0 | 0 | 1 | **0** | ✅ PASS | 100% | — |  | schemas | nightly |

## osm_integration

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-073 | OSM Extract Ingestion Success Rate | ↑ | pct | 100.00% | 99.50% | 98.00% | **99.50%** | ✅ PASS | 100% | — |  | ingestion | weekly |
| KPI-074 | OSM Attribute Normalization Coverage | ↑ | pct | 98.00% | 95.00% | 90.00% | **95.00%** | ✅ PASS | 100% | — |  | normalization | monthly |
| KPI-075 | OSM to Canonical Contract Violations | ↓ | per-10k | 0 | 1 | 3 | **1** | ✅ PASS | 100% | — | 🔒 | normalization | ci |

## sensors_probe

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-076 | Probe Data GPS Quality CEP50 | ↓ | m | 2 | 4 | 8 | **4** | ✅ PASS | 100% | — |  | ingestion | monthly |
| KPI-077 | Sensor Observation Admissibility Rate | ↑ | pct | 95.00% | 90.00% | 85.00% | **90.00%** | ✅ PASS | 100% | — |  | fusion-confidence | weekly |
| KPI-078 | Multi-Sensor Fusion Agreement | ↑ | pct | 90.00% | 85.00% | 80.00% | **85.00%** | ✅ PASS | 100% | — |  | fusion-confidence | monthly |
| KPI-079 | Imagery Georeferencing Accuracy | ↑ | pct | 98.00% | 95.00% | 92.00% | **95.00%** | ✅ PASS | 100% | — |  | cv-preprocessing | monthly |

## rendering_tiles

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-080 | Vector Tile Generation Throughput | ↑ | tiles/s | 2,000 | 1,000 | 500 | **1,000** | ✅ PASS | 100% | — | 🔒 | tile-engine | ci |
| KPI-081 | Tile p95 Serve Latency | ↓ | ms | 50 | 100 | 200 | **100** | ✅ PASS | 100% | — | 🔒 | api-service | ci |
| KPI-082 | Rendering Visual Regression Pass Rate | ↑ | pct | 100.00% | 99.00% | 97.00% | **99.00%** | ✅ PASS | 100% | — |  | web-maplibre | weekly |
| KPI-083 | Tile Size Budget Compliance | ↑ | pct | 100.00% | 99.50% | 98.00% | **99.50%** | ✅ PASS | 100% | — |  | tile-engine | weekly |

## coverage

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-084 | Geographic Area Coverage | ↑ | pct | 100.00% | 99.00% | 97.00% | **99.00%** | ✅ PASS | 100% | — |  | quality-engine | monthly |
| KPI-085 | Road Network Completeness vs Reference | ↑ | pct | 99.50% | 98.00% | 96.00% | **98.00%** | ✅ PASS | 100% | — |  | conflation-engine | weekly |
| KPI-086 | Urban Area Lane-Level Coverage | ↑ | pct | 90.00% | 80.00% | 70.00% | **80.00%** | ✅ PASS | 100% | — |  | cv-lane-detection | monthly |
| KPI-087 | Rural Area Coverage Gap Count | ↓ | count | 0 | 10 | 25 | **10** | ✅ PASS | 100% | — |  | quality-engine | monthly |

## freshness

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-088 | Map Update Cycle Time P95 | ↓ | h | 12 | 24 | 48 | **24** | ✅ PASS | 100% | — |  | pipelines | weekly |
| KPI-089 | Feature Age Median | ↓ | d | 30 | 90 | 180 | **90** | ✅ PASS | 100% | — |  | provenance | monthly |
| KPI-090 | Stale Feature Rate Over 180 Days | ↓ | pct | 1.00% | 5.00% | 10.00% | **5.00%** | ✅ PASS | 100% | — |  | provenance | monthly |

## reliability_security

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| KPI-091 | Pipeline Crash-Free Rate | ↑ | pct | 100.00% | 99.90% | 99.50% | **99.90%** | ✅ PASS | 100% | — |  | ci-harness | weekly |
| KPI-092 | Secret Leak Incidents Past Push Protection | ↓ | count | 0 | 0 | 0 | **0** | ✅ PASS | — | — | 🔒 | security-ci | ci |

## infra_performance

| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |
|---|---|---|---|---|---|---|---|---|---:|---|---|---|
| INFRA-001 | Geometry Op Benchmark Regression | ↓ | pct | 0.00% | 5.00% | 10.00% | **5.00%** | ✅ PASS | 100% | ↑ | 🔒 | criterion | ci |
| INFRA-002 | Spatial Lookup Benchmark Regression | ↓ | pct | 0.00% | 5.00% | 10.00% | **5.00%** | ✅ PASS | 100% | — | 🔒 | criterion | ci |
| INFRA-003 | Conflation Benchmark Regression | ↓ | pct | 0.00% | 5.00% | 10.00% | **5.00%** | ✅ PASS | 100% | — | 🔒 | criterion | ci |
| INFRA-004 | PBF Parsing Benchmark Regression | ↓ | pct | 0.00% | 5.00% | 10.00% | **5.00%** | ✅ PASS | 100% | — | 🔒 | criterion | ci |
| INFRA-005 | Tile Generation Benchmark Regression | ↓ | pct | 0.00% | 5.00% | 10.00% | **5.00%** | ✅ PASS | 100% | — | 🔒 | criterion | ci |
| INFRA-006 | API p95 Latency Regression | ↓ | pct | 0.00% | 10.00% | 15.00% | **10.00%** | ✅ PASS | 100% | — | 🔒 | locust | ci |
| INFRA-007 | Memory Footprint Regression | ↓ | pct | 0.00% | 10.00% | 15.00% | **10.00%** | ✅ PASS | 100% | — | 🔒 | memory-profiler | ci |
| INFRA-008 | Sample Pipeline Wall Time Regression | ↓ | pct | 0.00% | 10.00% | 15.00% | **10.00%** | ✅ PASS | 100% | — | 🔒 | pytest-benchmark | ci |

## Action items (gated KPIs needing attention)

None — all gated KPIs are meeting their thresholds.

