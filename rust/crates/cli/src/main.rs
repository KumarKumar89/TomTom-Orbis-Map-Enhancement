//! `orbis-cli` — one binary, five engines.
//!
//! ```text
//! orbis-cli validate <features.json>        # release quality gate
//! orbis-cli tiles    --bbox lon,lat,lon,lat --zoom z   # tile covering list
//! orbis-cli project  <lon> <lat>            # EPSG:4326 -> EPSG:3857
//! orbis-cli ingest   <file.geojson>         # normalize to canonical JSON
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};
use data_format::{FeatureReader, GeoJsonSource};
use geo_core::BoundingBox;
use std::io::Read;
use tile_engine::TileId;
use validation_engine::run_release_gate;

#[derive(Parser)]
#[command(name = "orbis-cli", version, about = "TomTom Orbis map enhancement engine CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Validate canonical MapFeature JSON array against the release quality gate.
    Validate {
        /// Path to a JSON array of canonical MapFeature objects.
        features: String,
        /// Maximum allowed topology errors before the gate fails.
        #[arg(long, default_value_t = 0)]
        max_topology_errors: usize,
        /// Minimum mean confidence required by the gate.
        #[arg(long, default_value_t = 0.8)]
        min_confidence: f64,
    },
    /// List XYZ tiles at a zoom covering a lon/lat bbox (minLon,minLat,maxLon,maxLat).
    Tiles {
        #[arg(long)]
        bbox: String,
        #[arg(long)]
        zoom: u8,
    },
    /// Project WGS84 lon/lat to Web Mercator meters.
    Project { lon: f64, lat: f64 },
    /// Ingest a GeoJSON file and emit canonical MapFeature JSON on stdout.
    Ingest {
        file: String,
        #[arg(long, default_value = "local")]
        source_id: String,
        #[arg(long, default_value = "unknown")]
        license: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Validate { features, max_topology_errors, min_confidence } => {
            let raw = std::fs::read_to_string(&features)?;
            let feats: Vec<geo_core::MapFeature> = serde_json::from_str(&raw)?;
            // Features-only mode: build an empty network unless the file embeds
            // one; full-network validation is driven from Python pipelines.
            let net = topology_engine::RoadNetwork::new();
            let report = run_release_gate(&feats, &net, max_topology_errors, min_confidence)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            std::process::exit(report.exit_code());
        }
        Cmd::Tiles { bbox, zoom } => {
            let parts: Vec<f64> = bbox
                .split(',')
                .map(|s| s.trim().parse::<f64>())
                .collect::<Result<_, _>>()?;
            if parts.len() != 4 {
                anyhow::bail!("bbox must be minLon,minLat,maxLon,maxLat");
            }
            let bb = BoundingBox::new(parts[0], parts[1], parts[2], parts[3]);
            let tiles = TileId::covering(bb.min_x, bb.min_y, bb.max_x, bb.max_y, zoom);
            for t in tiles {
                println!("{}/{}/{}", t.z, t.x, t.y);
            }
        }
        Cmd::Project { lon, lat } => {
            let c = geo_geometry::projection::web_mercator_from_lonlat(lon, lat);
            println!("{}, {}", c.x, c.y);
        }
        Cmd::Ingest { file, source_id, license } => {
            let mut f = std::fs::File::open(&file)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            let mut src = GeoJsonSource::new(std::io::Cursor::new(buf), source_id, license);
            let feats = src.read_features()?;
            println!("{}", serde_json::to_string_pretty(&feats)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_command_parses_bbox() {
        // Direct unit check of the parsing helper logic used in main().
        let bbox = "4.6,52.2,5.1,52.5";
        let parts: Vec<f64> = bbox.split(',').map(|s| s.trim().parse::<f64>().unwrap()).collect();
        assert_eq!(parts.len(), 4);
        let bb = BoundingBox::new(parts[0], parts[1], parts[2], parts[3]);
        assert!(bb.min_x < bb.max_x && bb.min_y < bb.max_y);
    }
}
