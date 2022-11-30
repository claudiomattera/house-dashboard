use serde::Deserialize;

use house_dashboard_common::duration::Iso8601Duration;

#[derive(Debug, Deserialize)]
pub struct InfrastructureSummaryConfiguration {
    pub how_long_ago: Iso8601Duration,
    pub suffix: Option<String>,
    pub last_update_format: Option<String>,
    pub vertical_step: Option<i32>,
}
