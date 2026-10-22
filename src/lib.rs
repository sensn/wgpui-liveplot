//! wgpui_liveplot is a high-performance plotting library built for wgpui.
//!
//! # Overview
//! - Designed for append-only, high-throughput telemetry and sensor streams.
//! - Plot-level axes with shared transforms across all series.
//! - Viewport-aware decimation keeps rendering near `O(width)` for smooth interaction.
//! - Interactive pan, zoom, box zoom, hover readout, and pin annotations via wgpui.
//!
//! # Feature flags
//! - `gpui_component_theme`: when enabled, [`wgpui_backend::PlotView`] will use the
//!   current `wgpui-kit` global theme if it exists.
//!
//! # Quick start
//! ```rust
//! use wgpui_liveplot::{LineStyle, Plot, Series, SeriesKind, Theme};
//!
//! let mut plot = Plot::builder().theme(Theme::dark()).build();
//! let series = Series::from_iter_y(
//!     "sensor",
//!     (0..1000).map(|i| (i as f64 * 0.01).sin()),
//!     SeriesKind::Line(LineStyle::default()),
//! );
//! plot.add_series(&series);
//! plot.refresh_viewport(0.05, 1e-6);
//! ```
//!
//! # wgpui integration
//! Use [`wgpui_backend::PlotView`] to render and interact with a plot inside a wgpui
//! window. See the `examples/` directory for complete runnable examples.

#![forbid(unsafe_code)]

pub mod axis;
pub mod datasource;
pub mod geom;
pub mod interaction;
pub mod plot;
pub mod render;
pub mod series;
pub mod style;
pub mod transform;
pub mod view;

// Renamed the backend module to match the new engine context
pub mod wgpui_backend;

// Exporting core canvas types straight from the wgpui crate
pub use wgpui::{Hsla, Rgba};

pub use axis::{AxisConfig, AxisConfigBuilder, AxisFormatter, TickConfig};
pub use datasource::AppendError;
pub use geom::Point;
pub use interaction::Pin;
pub use plot::{Plot, PlotBuilder};
pub use render::{LineStyle, MarkerShape, MarkerStyle};
pub use series::{Series, SeriesId, SeriesKind};
pub use style::Theme;
pub use view::{Range, View, Viewport};

pub use wgpui_backend::{
    LinkMemberId, PlotHandle, PlotLinkGroup, PlotLinkOptions, PlotView, PlotViewConfig,
};
