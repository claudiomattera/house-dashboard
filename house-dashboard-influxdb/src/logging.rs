// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for setting up logging system

use std::io::stderr;

use tracing::subscriber::set_global_default;
use tracing_journald::Layer as JournaldLayer;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::layer as SubscriberFmtLayer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Registry};

/// Setup logging
///
/// # Errors
///
/// Return an error when
///
/// * it failed to set a wrapper for `log::*`,
/// * it failed to open journald socket,
/// * it failed to set the tracing subscriber.
pub fn setup(verbosity: u8) {
    LogTracer::init().unwrap();

    let default_log_filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug,isahc=info",
        _ => "trace,isahc=info",
    };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_log_filter));

    let formatting_layer = SubscriberFmtLayer()
        .with_target(true)
        .without_time()
        .with_ansi(true)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE)
        .with_writer(stderr);

    let journald_layer = JournaldLayer::new().unwrap();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(journald_layer);

    set_global_default(subscriber).unwrap();
}
