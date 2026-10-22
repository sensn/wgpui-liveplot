use wgpui::{ AppContext, Bounds, WindowBounds, WindowOptions, px, size};
//component::theme::ThemeMode,
use wgpui_liveplot::{
    AxisConfig, LineStyle, MarkerShape, MarkerStyle, Plot, PlotView, PlotViewConfig, Rgba, Series, SeriesKind, Theme,
};

fn main() {
    // wgpui_kit::application() ersetzt die alten GPUI Application-Hooks,
    // um eine native wgpu + winit Fensterschleife aufzubauen.
    wgpui_kit::application().run(|cx| {
        // Initialisiert die UI-Komponenten und das globale Design-System des Toolkits.
        wgpui_kit::init(cx);
        //wgpui_kit::component::theme::Theme::change(ThemeMode::Dark, None, cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(720.0), px(480.0)),
                cx,
            ))),
            ..Default::default()
        };

        cx.open_window(options, |_window, cx| {
            // 1. Die primäre Datenreihe: Hellblaue Sinuskurve (Linie)
            let series = Series::from_iter_y(
                "signal",
                (0..400).map(|i| {
                    let x = i as f64 * 0.03;
                    x.sin()
                }),
                SeriesKind::Line(LineStyle {
                    color: Rgba {
                        r: 0.2,
                        g: 0.75,
                        b: 0.95,
                        a: 1.0,
                    },
                    width: 2.0,
                }),
            );

            // 2. ERWEITERUNG FÜR RECHTECK-TEST:
            // Wir streuen alle 40 Punkte ein quadratisches Element ein.
            // Nutzt im Hintergrund `window.paint_quad` (identisch zu paint_rect).
            let squares_series = Series::from_iter_points(
                "test-squares",
                (0..10).map(|i| {
                    let idx = i * 40;
                    let x = idx as f64;
                    let y = (idx as f64 * 0.03).sin();
                    wgpui_liveplot::Point::new(x, y)
                }),
                SeriesKind::Scatter(MarkerStyle {
                    color: Rgba {
                        r: 0.95,
                        g: 0.45,
                        b: 0.35,
                        a: 1.0,
                    },
                    size: 12.0, // Deutlich sichtbare Quadrate
                    shape: MarkerShape::Square,
                }),
            );

            let mut plot = Plot::builder()
                .theme(Theme::dark())
                .x_axis(AxisConfig::builder().title("Sample").build())
                .y_axis(AxisConfig::builder().title("Amplitude").build())
                .build();
            
            plot.add_series(&series);
            plot.add_series(&squares_series);

            let config = PlotViewConfig {
                show_legend: true,
                show_hover: true,
                ..Default::default()
            };

            // Den fertigen PlotView im wgpui Window-Modell registrieren
            let view = PlotView::with_config(plot, config);
            cx.new(|_| view)
        })
        .unwrap();
    });
}
