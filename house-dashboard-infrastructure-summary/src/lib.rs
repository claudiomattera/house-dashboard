use std::collections::{HashMap, HashSet};
use std::path::Path;

use tracing::instrument;

use miette::{Report, WrapErr};

use chrono::Utc;

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_infrastructure_summary;

mod configuration;
pub use self::configuration::InfrastructureSummaryConfiguration;

mod error;
pub use self::error::Error;

mod loadbar;
pub use self::loadbar::Loadbar;

/// Fetch data and draw chart for infrastructure summary
#[instrument(
    name = "infrastructure_summary",
    skip(infrastructure_summary_configuration, style_configuration)
)]
#[allow(clippy::unused_async)]
pub async fn process_infrastructure_summary(
    infrastructure_summary_configuration: &InfrastructureSummaryConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<(), Report> {
    let now = Utc::now();

    let (hosts, loads) = fetch_data()
        .await
        .wrap_err("cannot fetch data for infrastructure summary")?;

    let filename = format!("{:02}.bmp", index + 1);
    let path = Path::new(&filename);
    let backend = BitMapBackend::new(&path, style_configuration.resolution);
    draw_infrastructure_summary(
        infrastructure_summary_configuration,
        now,
        hosts,
        loads,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw infrastructure summary")?;

    Ok(())
}

/// Fetch data for infrastructure summary
async fn fetch_data() -> Result<(HashSet<String>, HashMap<String, f64>), Report> {
    let mut hosts: HashSet<String> = HashSet::new();
    hosts.insert("dashboard.dk.claudiomattera.it".to_owned());
    hosts.insert("h2plus.dk.claudiomattera.it".to_owned());
    hosts.insert("media-center.dk.claudiomattera.it".to_owned());
    hosts.insert("vps.de.claudiomattera.it".to_owned());

    let mut loads: HashMap<String, f64> = HashMap::new();
    loads.insert("dashboard.dk.claudiomattera.it".to_owned(), 0.2);
    loads.insert("h2plus.dk.claudiomattera.it".to_owned(), 0.6);
    loads.insert("media-center.dk.claudiomattera.it".to_owned(), 0.9);
    loads.insert("vps.de.claudiomattera.it".to_owned(), 0.1);

    Ok((hosts, loads))
}
