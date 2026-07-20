use dioxus::prelude::*;
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use gloo_timers::future::sleep;
use instant::Instant;
use std::time::Duration;

use yasf::starforce::{
    EnhancementMode, EnhanceConfig, BIN_SIZE, SimResult, CanvasItem, stars_engine, simulate_equip,
    propagate_canvas_items, format_mesos, format_duration,
};
use yasf::components::star_visual::StarsVisualizer;

static CSS: Asset = asset!("/assets/main.css");

#[derive(Clone, Copy, Debug, PartialEq)]
enum AppTab {
    Simulator,
    Canvas,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum DisplayMode {
    #[default]
    Average,
    Median,
}

struct PaletteTemplate {
    name: &'static str,
    level: u32,
    asset: Asset,
}

const TEMPLATES: &[PaletteTemplate] = &[
    PaletteTemplate { name: "Meister Ring", level: 140, asset: asset!("/assets/image/Eqp_Meister_Ring.png") },
    PaletteTemplate { name: "Superior Gollux Ring", level: 150, asset: asset!("/assets/image/Eqp_Superior_Gollux_Ring.png") },
    PaletteTemplate { name: "Kanna's Treasure", level: 140, asset: asset!("/assets/image/Eqp_Kanna's_Treasure.png") },
    PaletteTemplate { name: "Guardian Angel Ring", level: 160, asset: asset!("/assets/image/Eqp_Guardian_Angel_Ring.png") },
    PaletteTemplate { name: "Endless Terror", level: 200, asset: asset!("/assets/image/Eqp_Endless_Terror.png") },
    PaletteTemplate { name: "Whisper of the Source", level: 250, asset: asset!("/assets/image/Eqp_Whisper_of_the_Source.png") },
    PaletteTemplate { name: "Blissful Nightmare", level: 250, asset: asset!("/assets/image/Eqp_Blissful_Nightmare.png") },
    PaletteTemplate { name: "Twilight Mark", level: 140, asset: asset!("/assets/image/Eqp_Twilight_Mark.png") },
    PaletteTemplate { name: "Berserked", level: 160, asset: asset!("/assets/image/Eqp_Berserked.png") },
    PaletteTemplate { name: "Original Sin of Pride", level: 250, asset: asset!("/assets/image/Eqp_Original_Sin_of_Pride.png") },
    PaletteTemplate { name: "Magic Eyepatch", level: 160, asset: asset!("/assets/image/Eqp_Magic_Eyepatch.png") },
    PaletteTemplate { name: "Reinforced Gollux Earrings", level: 140, asset: asset!("/assets/image/Eqp_Reinforced_Gollux_Earrings.png") },
    PaletteTemplate { name: "Superior Gollux Earrings", level: 150, asset: asset!("/assets/image/Eqp_Superior_Gollux_Earrings.png") },
    PaletteTemplate { name: "Estella Earrings", level: 160, asset: asset!("/assets/image/Eqp_Estella_Earrings.png") },
    PaletteTemplate { name: "Commanding Force Earring", level: 200, asset: asset!("/assets/image/Eqp_Commanding_Force_Earring.png") },
    PaletteTemplate { name: "Dominator Pendant", level: 140, asset: asset!("/assets/image/Eqp_Dominator_Pendant.png") },
    PaletteTemplate { name: "Daybreak Pendant", level: 140, asset: asset!("/assets/image/Eqp_Daybreak_Pendant.png") },
    PaletteTemplate { name: "Superior Engraved Gollux Pendant", level: 150, asset: asset!("/assets/image/Eqp_Superior_Engraved_Gollux_Pendant.png") },
    PaletteTemplate { name: "Source of Suffering", level: 160, asset: asset!("/assets/image/Eqp_Source_of_Suffering.png") },
    PaletteTemplate { name: "Oath of Death", level: 250, asset: asset!("/assets/image/Eqp_Oath_of_Death.png") },
    PaletteTemplate { name: "Golden Clover Belt", level: 140, asset: asset!("/assets/image/Eqp_Golden_Clover_Belt.png") },
    PaletteTemplate { name: "Ayame's Treasure", level: 140, asset: asset!("/assets/image/Eqp_Ayame's_Treasure.png") },
    PaletteTemplate { name: "Reinforced Engraved Gollux Belt", level: 140, asset: asset!("/assets/image/Eqp_Reinforced_Engraved_Gollux_Belt.png") },
    PaletteTemplate { name: "Superior Engraved Gollux Belt", level: 150, asset: asset!("/assets/image/Eqp_Superior_Engraved_Gollux_Belt.png") },
    PaletteTemplate { name: "Dreamy Belt", level: 200, asset: asset!("/assets/image/Eqp_Dreamy_Belt.png") },
];

fn main() {
    dioxus::launch(App);
}

use num_format::{Locale, ToFormattedString};


#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum DeviceLayout {
    #[default]
    Auto,
    Desktop,
    Mobile,
}

#[component]
fn App() -> Element {
    // Target & General Inputs
    let mut target_input = use_signal(|| String::from("22"));
    let mut start_input = use_signal(|| String::from("0"));
    let mut equip_level_input = use_signal(|| String::from("200"));
    let mut trials_input = use_signal(|| 100_000_u32);

    // Event & Config Inputs
    let mut star_catch_input = use_signal(|| true);
    let mut ssf_boom_input = use_signal(|| true);
    let mut ssf_cost_input = use_signal(|| true);
    let mut safeguard_input = use_signal(|| true);
    
    // Enhancement Mode for stars 15-21 (7 levels)
    let mut mode_15_21_input = use_signal(|| [EnhancementMode::Level1; 7]);
    
    // UI Presets & states
    let mut equip_level_preset = use_signal(|| Some(200_u32));
    let mut is_drawer_open = use_signal(|| false);
    let layout_mode = use_signal(|| DeviceLayout::Auto);
    let mut equipment_count = use_signal(|| 5_u32);

    // Execution States
    let mut result = use_signal(|| None::<SimResult>);
    let mut time_elaps = use_signal(|| None::<Duration>);
    let mut error_msg = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);

    // Canvas & Tab States
    let mut active_tab = use_signal(|| AppTab::Simulator);
    let mut canvas_items = use_signal(|| Vec::<CanvasItem>::new());
    let mut selected_item_id = use_signal(|| None::<usize>);
    let mut next_id = use_signal(|| 1_usize);
    let mut dragging_id = use_signal(|| None::<usize>);
    let mut dragged_palette_idx = use_signal(|| None::<usize>);
    let mut drag_start_mouse = use_signal(|| (0.0f64, 0.0f64));
    let mut drag_start_item = use_signal(|| (0_i32, 0_i32));
    let mut is_palette_open = use_signal(|| true);
    let mut is_properties_open = use_signal(|| true);
    let mut is_canvas_drawer_open = use_signal(|| false);
    let mut connections = use_signal(|| Vec::<(usize, usize)>::new());
    let mut active_source_pin = use_signal(|| None::<usize>);
    let mut draft_mouse_coords = use_signal(|| (0_i32, 0_i32));
    let mut canvas_error = use_signal(|| String::new());
    // Infinite Canvas Viewport States
    let mut pan_x = use_signal(|| 0.0f64);
    let mut pan_y = use_signal(|| 0.0f64);
    let mut zoom_level = use_signal(|| 1.0f64);
    let mut is_panning = use_signal(|| false);
    let mut pan_start_mouse = use_signal(|| (0.0f64, 0.0f64));
    let mut pan_start_offset = use_signal(|| (0.0f64, 0.0f64));
    let mut is_space_pressed = use_signal(|| false);
    let mut canvas_client_offset = use_signal(|| (0.0f64, 0.0f64));
    // Canvas simulation config (global, affects all items on canvas)
    let mut canvas_trials = use_signal(|| 50_000_u32);
    let mut cost_display_mode = use_signal(|| DisplayMode::Average);
    let mut booms_display_mode = use_signal(|| DisplayMode::Average);
    // Compute joint probability distribution curve (CDF) based on spare equipment items
    let cdf_points = use_memo(move || {
        if let Some(res) = result() {
            let max_cost = res.percentile_99;
            let mut cdf = Vec::new();
            cdf.push((0u128, 0.0f64));
            
            let tolerated_booms = (equipment_count() as usize).saturating_sub(1);
            
            for i in 1..=20 {
                let budget = (max_cost as f64 * (i as f64 / 20.0)) as u128;
                let target_bin = (budget / BIN_SIZE as u128) as usize;
                let mut accumulated_count = 0;
                
                for bin_idx in 0..=target_bin {
                    if bin_idx < res.joint_histogram.len() {
                        for b in 0..=tolerated_booms {
                            if b < 100 {
                                accumulated_count += res.joint_histogram[bin_idx][b];
                            }
                        }
                    }
                }
                let prob = accumulated_count as f64 / res.total_trials as f64;
                cdf.push((budget, prob));
            }
            cdf
        } else {
            Vec::new()
        }
    });    // Client-side CDF Chart rendering (reactive to result, active_tab, layout_mode, and equipment_count changes)
    use_effect(move || {
        let tab = active_tab();
        if tab != AppTab::Simulator {
            return;
        }
        let res = result();
        let cdf_data = cdf_points();
        let _layout = layout_mode();
        if let Some(_) = res {
            let cdf_labels: Vec<String> = cdf_data.iter().map(|(budget, _)| format_mesos(*budget)).collect();
            let cdf_values: Vec<f64> = cdf_data.iter().map(|(_, prob)| prob * 100.0).collect();

            let cdf_labels_js = format!("{:?}", cdf_labels);
            let cdf_values_js = format!("{:?}", cdf_values);

            let js_code = format!(
                r#"
                function drawCDF() {{
                    if (typeof Chart === 'undefined') {{
                        let script = document.getElementById('chartjs-script');
                        if (!script) {{
                            script = document.createElement('script');
                            script.id = 'chartjs-script';
                            script.src = 'https://cdn.jsdelivr.net/npm/chart.js';
                            document.head.appendChild(script);
                        }}
                        setTimeout(drawCDF, 100);
                        return;
                    }}
                    
                    const cdfCanvas = document.getElementById('cdf-chart-canvas');
                    if (!cdfCanvas) {{
                        setTimeout(drawCDF, 50);
                        return;
                    }}
                    
                    const gridColor = 'rgba(255, 255, 255, 0.08)';
                    const textColor = '#9ca3af';
                    const fontMono = "'JetBrains Mono', monospace";
                    const fontSans = "'Plus Jakarta Sans', sans-serif";
                    
                    Chart.defaults.color = textColor;
                    Chart.defaults.font.family = fontSans;

                    if (window.cdfChartInstance) {{
                        window.cdfChartInstance.destroy();
                    }}
                    window.cdfChartInstance = new Chart(cdfCanvas, {{
                        type: 'line',
                        data: {{
                            labels: {cdf_labels},
                            datasets: [{{
                                label: 'Success Chance',
                                data: {cdf_values},
                                borderColor: '#e2b44c',
                                backgroundColor: 'rgba(226, 180, 76, 0.08)',
                                borderWidth: 2,
                                pointBackgroundColor: '#070913',
                                pointBorderColor: '#e2b44c',
                                pointRadius: 3,
                                fill: true,
                                tension: 0.4
                            }}]
                        }},
                        options: {{
                            responsive: true,
                            maintainAspectRatio: false,
                            plugins: {{
                                legend: {{ display: false }},
                                tooltip: {{
                                    callbacks: {{
                                        label: (ctx) => ctx.parsed.y.toFixed(1) + '% chance',
                                        title: (ctx) => 'Budget: ' + ctx[0].label
                                    }},
                                    titleFont: {{ family: fontMono }},
                                    bodyFont: {{ family: fontMono }}
                                }}
                            }},
                            scales: {{
                                x: {{
                                    title: {{ display: true, text: 'Meso Budget', color: textColor }},
                                    grid: {{ color: gridColor }},
                                    ticks: {{ font: {{ family: fontMono }} }}
                                }},
                                y: {{
                                    title: {{ display: true, text: 'Probability (%)', color: textColor }},
                                    grid: {{ color: gridColor }},
                                    min: 0,
                                    max: 100,
                                    ticks: {{ font: {{ family: fontMono }}, stepSize: 20 }}
                                }}
                            }}
                        }}
                    }});
                }}
                drawCDF();
                "#,
                cdf_labels = cdf_labels_js,
                cdf_values = cdf_values_js
            );

            let _ = dioxus::document::eval(&js_code);
        }
    });

    // Client-side Bottlenecks & Boom Chart rendering (reactive to result, active_tab, and layout_mode changes)
    use_effect(move || {
        let tab = active_tab();
        if tab != AppTab::Simulator {
            return;
        }
        let res = result();
        let _layout = layout_mode();
        if let Some(res) = res {
            let has_high_stars = res.star_friction.iter().any(|f| f.star >= 15);
            let filter_limit = if has_high_stars { 15 } else { 0 };

            let bottleneck_labels: Vec<String> = res.star_friction.iter()
                .filter(|f| f.star >= filter_limit)
                .map(|f| format!("{}➔{}", f.star, f.star + 1))
                .collect();
            let bottleneck_costs: Vec<f64> = res.star_friction.iter()
                .filter(|f| f.star >= filter_limit)
                .map(|f| f.average_cost as f64)
                .collect();
            let bottleneck_booms: Vec<f64> = res.star_friction.iter()
                .filter(|f| f.star >= filter_limit)
                .map(|f| f.average_booms)
                .collect();

            let boom_labels: Vec<String> = res.boom_distribution.iter()
                .map(|(booms, _)| {
                    let limit = res.boom_distribution.len() - 1;
                    if *booms == limit as u32 {
                        format!("{}+", booms)
                    } else {
                        booms.to_string()
                    }
                })
                .collect();
            let boom_values: Vec<f64> = res.boom_distribution.iter()
                .map(|(_, prob)| prob * 100.0)
                .collect();

            let bottleneck_labels_js = format!("{:?}", bottleneck_labels);
            let bottleneck_costs_js = format!("{:?}", bottleneck_costs);
            let bottleneck_booms_js = format!("{:?}", bottleneck_booms);
            let boom_labels_js = format!("{:?}", boom_labels);
            let boom_values_js = format!("{:?}", boom_values);

            let js_code = format!(
                r#"
                function drawBottlenecksAndBooms() {{
                    if (typeof Chart === 'undefined') {{
                        let script = document.getElementById('chartjs-script');
                        if (!script) {{
                            script = document.createElement('script');
                            script.id = 'chartjs-script';
                            script.src = 'https://cdn.jsdelivr.net/npm/chart.js';
                            document.head.appendChild(script);
                        }}
                        setTimeout(drawBottlenecksAndBooms, 100);
                        return;
                    }}
                    
                    const costCanvas = document.getElementById('bottlenecks-cost-canvas');
                    const boomBnCanvas = document.getElementById('bottlenecks-boom-canvas');
                    const boomCanvas = document.getElementById('boom-chart-canvas');

                    if (!costCanvas || !boomBnCanvas || !boomCanvas) {{
                        setTimeout(drawBottlenecksAndBooms, 50);
                        return;
                    }}
                    
                    const gridColor = 'rgba(255, 255, 255, 0.08)';
                    const textColor = '#9ca3af';
                    const fontMono = "'JetBrains Mono', monospace";
                    const fontSans = "'Plus Jakarta Sans', sans-serif";
                    
                    Chart.defaults.color = textColor;
                    Chart.defaults.font.family = fontSans;
                    
                    // Helper to format values
                    function formatMesosJS(val) {{
                        if (val >= 1e9) return (val / 1e9).toFixed(2) + 'B';
                        if (val >= 1e6) return (val / 1e6).toFixed(1) + 'M';
                        if (val >= 1e3) return (val / 1e3).toFixed(1) + 'K';
                        return val;
                    }}

                    // Meso Cost Bottlenecks Chart
                    if (window.bnCostChartInstance) {{
                        window.bnCostChartInstance.destroy();
                    }}
                    window.bnCostChartInstance = new Chart(costCanvas, {{
                        type: 'bar',
                        data: {{
                            labels: {bottleneck_labels},
                            datasets: [{{
                                label: 'Avg Cost',
                                data: {bottleneck_costs},
                                backgroundColor: 'rgba(79, 70, 229, 0.75)',
                                borderColor: '#4f46e5',
                                borderWidth: 1,
                                borderRadius: 4
                            }}]
                        }},
                        options: {{
                            indexAxis: 'y',
                            responsive: true,
                            maintainAspectRatio: false,
                            plugins: {{
                                legend: {{ display: false }},
                                tooltip: {{
                                    callbacks: {{
                                        label: (ctx) => formatMesosJS(ctx.parsed.x) + ' mesos'
                                    }},
                                    bodyFont: {{ family: fontMono }}
                                }}
                            }},
                            scales: {{
                                x: {{
                                    type: 'linear',
                                    title: {{ display: true, text: 'Average Cost (Mesos)', color: textColor }},
                                    grid: {{ color: gridColor }},
                                    ticks: {{
                                        font: {{ family: fontMono }},
                                        callback: (val) => formatMesosJS(val)
                                    }}
                                }},
                                y: {{
                                    grid: {{ display: false }},
                                    ticks: {{ font: {{ family: fontMono, weight: 'bold' }} }}
                                }}
                            }}
                        }}
                    }});

                    // Boom Bottlenecks Chart
                    if (window.bnBoomChartInstance) {{
                        window.bnBoomChartInstance.destroy();
                    }}
                    window.bnBoomChartInstance = new Chart(boomBnCanvas, {{
                        type: 'bar',
                        data: {{
                            labels: {bottleneck_labels},
                            datasets: [{{
                                label: 'Avg Booms',
                                data: {bottleneck_booms},
                                backgroundColor: 'rgba(239, 68, 68, 0.75)',
                                borderColor: '#ef4444',
                                borderWidth: 1,
                                borderRadius: 4
                            }}]
                        }},
                        options: {{
                            indexAxis: 'y',
                            responsive: true,
                            maintainAspectRatio: false,
                            plugins: {{
                                legend: {{ display: false }},
                                tooltip: {{
                                    callbacks: {{
                                        label: (ctx) => ctx.parsed.x.toFixed(2) + ' booms'
                                    }},
                                    bodyFont: {{ family: fontMono }}
                                }}
                            }},
                            scales: {{
                                x: {{
                                    type: 'linear',
                                    title: {{ display: true, text: 'Average Booms', color: textColor }},
                                    grid: {{ color: gridColor }},
                                    ticks: {{
                                        font: {{ family: fontMono }}
                                    }}
                                }},
                                y: {{
                                    grid: {{ display: false }},
                                    ticks: {{ font: {{ family: fontMono, weight: 'bold' }} }}
                                }}
                            }}
                        }}
                    }});

                    // Boom Chart
                    if (window.boomChartInstance) {{
                        window.boomChartInstance.destroy();
                    }}
                    const bLabels = {boom_labels};
                    window.boomChartInstance = new Chart(boomCanvas, {{
                        type: 'bar',
                        data: {{
                            labels: bLabels,
                            datasets: [{{
                                label: 'Probability (%)',
                                data: {boom_values},
                                backgroundColor: bLabels.map((lbl) => lbl === '0' ? 'rgba(16, 185, 129, 0.75)' : 'rgba(233, 69, 96, 0.75)'),
                                borderColor: bLabels.map((lbl) => lbl === '0' ? '#10b981' : '#e94560'),
                                borderWidth: 1,
                                borderRadius: 4
                            }}]
                        }},
                        options: {{
                            responsive: true,
                            maintainAspectRatio: false,
                            plugins: {{
                                legend: {{ display: false }},
                                tooltip: {{
                                    callbacks: {{
                                        label: (ctx) => ctx.parsed.y.toFixed(1) + '% chance'
                                    }},
                                    bodyFont: {{ family: fontMono }}
                                }}
                            }},
                            scales: {{
                                x: {{
                                    title: {{ display: true, text: 'Number of Booms', color: textColor }},
                                    grid: {{ display: false }},
                                    ticks: {{ font: {{ family: fontMono }} }}
                                }},
                                y: {{
                                    title: {{ display: true, text: 'Probability (%)', color: textColor }},
                                    grid: {{ color: gridColor }},
                                    min: 0,
                                    max: 100,
                                    ticks: {{ font: {{ family: fontMono }}, stepSize: 25 }}
                                }}
                            }}
                        }}
                    }});
                }}
                drawBottlenecksAndBooms();
                "#,
                bottleneck_labels = bottleneck_labels_js,
                bottleneck_costs = bottleneck_costs_js,
                bottleneck_booms = bottleneck_booms_js,
                boom_labels = boom_labels_js,
                boom_values = boom_values_js
            );

            let _ = dioxus::document::eval(&js_code);
        }
    });

    // Helper functions for incrementing/decrementing equipment count
    let increment_equip = move |_| {
        let current = equipment_count();
        if current < 100 {
            equipment_count.set(current + 1);
        }
    };

    let decrement_equip = move |_| {
        let current = equipment_count();
        if current > 1 {
            equipment_count.set(current - 1);
        }
    };

    // Helper functions for incrementing/decrementing stars
    let increment_start = move |_| {
        let current = start_input().parse::<usize>().unwrap_or(0);
        if current < 29 {
            start_input.set((current + 1).to_string());
        }
    };
    
    let decrement_start = move |_| {
        let current = start_input().parse::<usize>().unwrap_or(0);
        if current > 0 {
            start_input.set((current - 1).to_string());
        }
    };

    let increment_target = move |_| {
        let current = target_input().parse::<usize>().unwrap_or(0);
        if current < 30 {
            target_input.set((current + 1).to_string());
        }
    };
    
    let decrement_target = move |_| {
        let current = target_input().parse::<usize>().unwrap_or(0);
        if current > 1 {
            target_input.set((current - 1).to_string());
        }
    };

    // Computes if all star enhancement modes are identical
    let get_uniform_mode = move || {
        let modes = mode_15_21_input();
        let first = modes[0];
        if modes.iter().all(|&m| m == first) {
            Some(first)
        } else {
            None
        }
    };

    rsx! {
        document::Stylesheet { href: "https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700;800&family=Outfit:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" }
        document::Stylesheet { href: CSS }

        div {
            class: {
                let base = match layout_mode() {
                    DeviceLayout::Auto => "app-container layout-auto",
                    DeviceLayout::Desktop => "app-container layout-desktop-forced",
                    DeviceLayout::Mobile => "app-container layout-mobile-forced",
                };
                if active_tab() == AppTab::Canvas {
                    format!("{} full-screen-canvas-mode", base)
                } else {
                    base.to_string()
                }
            },
            // Header
            header { class: "app-header",
                h1 { class: "title-glow",
                    span { class: "visual-star filled", "★" }
                    "YASF - Yet Another StarForce"
                }
                p { class: "subtitle", "High-performance Monte Carlo simulation for MapleStory Star Force enhancement costs, booms, and luck distribution." }
            }

            // Tab Buttons
            div { class: "tabs-container",
                button {
                    class: if active_tab() == AppTab::Simulator { "tab-button active" } else { "tab-button" },
                    r#type: "button",
                    onclick: move |_| active_tab.set(AppTab::Simulator),
                    span { "🎲" }
                    "Monte Carlo Simulator"
                }
                button {
                    class: if active_tab() == AppTab::Canvas { "tab-button active" } else { "tab-button" },
                    r#type: "button",
                    onclick: move |_| active_tab.set(AppTab::Canvas),
                    span { "🎨" }
                    "Equipment Builder Canvas"
                }
            }

            if active_tab() == AppTab::Simulator {
                // Grid Dashboard
                div { class: "dashboard-grid",
                
                // Left Column: Control Panel
                div { class: "glass-card",
                    h2 { class: "panel-title gold",
                        span { "⚙" }
                        "Simulation Setup"
                    }

                    div { class: "form-group",
                        StarsVisualizer {
                            start: start_input().trim().parse::<usize>().unwrap_or(0),
                            target: target_input().trim().parse::<usize>().unwrap_or(0)
                        }
                    }
                    
                    // Start Stars Input
                    div { class: "form-group",
                        div { class: "label-container",
                            label { class: "input-label", r#for: "start-stars", "Start Stars" }
                            span { class: "input-value-badge", "{start_input} ★" }
                        }
                        div { style: "display: flex; gap: 8px;",
                            button {
                                class: "preset-btn",
                                style: "max-width: 42px;",
                                r#type: "button",
                                disabled: is_loading(),
                                onclick: decrement_start,
                                "-"
                            }
                            input {
                                id: "start-stars",
                                class: "input-control",
                                r#type: "number",
                                min: "0",
                                max: "29",
                                value: "{start_input}",
                                disabled: is_loading(),
                                oninput: move |evt| start_input.set(evt.value()),
                            }
                            button {
                                class: "preset-btn",
                                style: "max-width: 42px;",
                                r#type: "button",
                                disabled: is_loading(),
                                onclick: increment_start,
                                "+"
                            }
                        }
                    }

                    // Target Stars Input
                    div { class: "form-group",
                        div { class: "label-container",
                            label { class: "input-label", r#for: "target-stars", "Target Stars" }
                            span { class: "input-value-badge", "{target_input} ★" }
                        }
                        div { style: "display: flex; gap: 8px;",
                            button {
                                class: "preset-btn",
                                style: "max-width: 42px;",
                                r#type: "button",
                                disabled: is_loading(),
                                onclick: decrement_target,
                                "-"
                            }
                            input {
                                id: "target-stars",
                                class: "input-control",
                                r#type: "number",
                                min: "1",
                                max: "30",
                                value: "{target_input}",
                                disabled: is_loading(),
                                oninput: move |evt| target_input.set(evt.value()),
                            }
                            button {
                                class: "preset-btn",
                                style: "max-width: 42px;",
                                r#type: "button",
                                disabled: is_loading(),
                                onclick: increment_target,
                                "+"
                            }
                        }
                    }

                    // Stars visual progress bar


                    // Equipment Level Input & Presets
                    div { class: "form-group",
                        div { class: "label-container",
                            label { class: "input-label", r#for: "equip-level", "Equipment Level" }
                            span { class: "input-value-badge", "Lv. {equip_level_input}" }
                        }
                        input {
                            id: "equip-level",
                            class: "input-control",
                            r#type: "number",
                            min: "0",
                            max: "300",
                            value: "{equip_level_input}",
                            disabled: is_loading(),
                            oninput: move |evt| {
                                equip_level_input.set(evt.value());
                                let val = evt.value().parse::<u32>().ok();
                                if val == Some(140) || val == Some(150) || val == Some(160) || val == Some(200) || val == Some(250) {
                                    equip_level_preset.set(val);
                                } else {
                                    equip_level_preset.set(None);
                                }
                            },
                        }
                        div { class: "preset-row",
                            for level in [140, 150, 160, 200, 250] {
                                button {
                                    class: if equip_level_preset() == Some(level) { "preset-btn active" } else { "preset-btn" },
                                    r#type: "button",
                                    disabled: is_loading(),
                                    onclick: move |_| {
                                        equip_level_preset.set(Some(level));
                                        equip_level_input.set(level.to_string());
                                    },
                                    "Lv. {level}"
                                }
                            }
                        }
                    }

                    // Trials selection
                    div { class: "form-group",
                        label { class: "input-label", r#for: "sim-trials", "Simulation Quality (Trials)" }
                        select {
                            id: "sim-trials",
                            class: "select-control",
                            value: "{trials_input}",
                            disabled: is_loading(),
                            onchange: move |evt| {
                                if let Ok(val) = evt.value().parse::<u32>() {
                                    trials_input.set(val);
                                }
                            },
                            option { value: "10000", "Fast (10,000 trials)" }
                            option { value: "100000", "Standard (100,000 trials)" }
                            option { value: "1000000", "Precise (1,000,000 trials)" }
                        }
                    }

                    // Enhancement Mode
                    div { class: "form-group",
                        label { class: "input-label", "Enhancement Mode (Stars 15-21)" }
                        div { class: "mode-toggle-grid",
                            for (mode, label) in [
                                (EnhancementMode::Level1, "Lvl 1"),
                                (EnhancementMode::Level2, "Lvl 2"),
                                (EnhancementMode::Level3, "Lvl 3"),
                                (EnhancementMode::Level4, "Lvl 4"),
                            ] {
                                button {
                                    class: if get_uniform_mode() == Some(mode) { "mode-toggle-btn active" } else { "mode-toggle-btn" },
                                    r#type: "button",
                                    disabled: is_loading(),
                                    onclick: move |_| {
                                        mode_15_21_input.set([mode; 7]);
                                    },
                                    "{label}"
                                }
                            }
                        }

                        // Advanced per star configuration
                        div { class: "individual-star-config-drawer", style: "margin-top: 10px;",
                            div {
                                class: "drawer-header",
                                onclick: move |_| is_drawer_open.set(!is_drawer_open()),
                                span { class: "drawer-title", "Customize per Star Level" }
                                span {
                                    class: if is_drawer_open() { "drawer-arrow open" } else { "drawer-arrow" },
                                    "▶"
                                }
                            }
                            div {
                                class: if is_drawer_open() { "drawer-content open" } else { "drawer-content" },
                                div { class: "individual-star-config",
                                    for i in 0..7 {
                                        div { class: "star-config-row",
                                            span { class: "star-config-row-label",
                                                span { class: "star-icon", "★" }
                                                "{15 + i} ➔ {15 + i + 1}"
                                            }
                                            div { class: "star-config-row-selectors",
                                                div { class: "mode-toggle-grid",
                                                    for (m, lbl) in [
                                                        (EnhancementMode::Level1, "L1"),
                                                        (EnhancementMode::Level2, "L2"),
                                                        (EnhancementMode::Level3, "L3"),
                                                        (EnhancementMode::Level4, "L4"),
                                                    ] {
                                                        button {
                                                            class: if mode_15_21_input()[i] == m { "mode-toggle-btn active" } else { "mode-toggle-btn" },
                                                            r#type: "button",
                                                            disabled: is_loading(),
                                                            onclick: move |_| {
                                                                let mut modes = mode_15_21_input();
                                                                modes[i] = m;
                                                                mode_15_21_input.set(modes);
                                                            },
                                                            "{lbl}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Event Modifier Cards
                    div { class: "form-group",
                        label { class: "input-label", "Event Modifiers" }
                        div { class: "modifiers-grid",
                            div {
                                class: if star_catch_input() { "modifier-card active" } else { "modifier-card" },
                                onclick: move |_| {
                                    if !is_loading() { star_catch_input.set(!star_catch_input()); }
                                },
                                div { class: "modifier-checkbox" }
                                span { class: "modifier-label", "Star Catch (+5% success)" }
                            }
                            div {
                                class: if safeguard_input() { "modifier-card active" } else { "modifier-card" },
                                onclick: move |_| {
                                    if !is_loading() { safeguard_input.set(!safeguard_input()); }
                                },
                                div { class: "modifier-checkbox" }
                                span { class: "modifier-label", "Safeguard (15-17★)" }
                            }
                            div {
                                class: if ssf_cost_input() { "modifier-card active" } else { "modifier-card" },
                                onclick: move |_| {
                                    if !is_loading() { ssf_cost_input.set(!ssf_cost_input()); }
                                },
                                div { class: "modifier-checkbox" }
                                span { class: "modifier-label", "SSF Cost Red." }
                            }
                            div {
                                class: if ssf_boom_input() { "modifier-card active" } else { "modifier-card" },
                                onclick: move |_| {
                                    if !is_loading() { ssf_boom_input.set(!ssf_boom_input()); }
                                },
                                div { class: "modifier-checkbox" }
                                span { class: "modifier-label", "SSF Boom Red." }
                            }
                        }
                    }

                    // Action trigger
                    div { style: "margin-top: 25px;",
                        button {
                            class: "action-button",
                            disabled: is_loading(),
                            r#type: "button",
                            onclick: move |_| {
                                error_msg.set(String::new());

                                let s_val = start_input().trim().parse::<usize>().unwrap_or(0);
                                let t_val = target_input().trim().parse::<usize>().unwrap_or(0);
                                let e_val = equip_level_input().trim().parse::<u32>().unwrap_or(200);

                                if t_val <= s_val {
                                    result.set(None);
                                    error_msg.set("Target stars must be greater than Start stars.".to_string());
                                    return;
                                }
                                if t_val > 30 || s_val > 29 {
                                    result.set(None);
                                    error_msg.set("Stars must be within the 0-30 range.".to_string());
                                    return;
                                }

                                is_loading.set(true);
                                result.set(None);

                                let num_trials = trials_input();
                                let sim_config = EnhanceConfig {
                                    mode_15_21: mode_15_21_input(),
                                    star_catch: star_catch_input(),
                                    ssf_boom_reduce_event: ssf_boom_input(),
                                    ssf_cost_reduce_event: ssf_cost_input(),
                                    safeguard: safeguard_input(),
                                };

                                spawn(async move {
                                    sleep(Duration::from_millis(1)).await;

                                    let start_time = Instant::now();
                                    let output = stars_engine(s_val, t_val, e_val, num_trials, sim_config);

                                    result.set(Some(output));
                                    time_elaps.set(Some(start_time.elapsed()));
                                    is_loading.set(false);
                                });
                            },
                            if is_loading() {
                                span { "Simulating..." }
                            } else {
                                span { "Run Simulation" }
                            }
                        }
                    }

                    if is_loading() {
                        div { class: "loader-container",
                            div { class: "spinner" }
                            span { style: "font-size: 0.85rem; color: var(--color-gold); font-weight: 500;",
                                "Running {trials_input().to_formatted_string(&Locale::en)} trials..."
                            }
                        }
                    }

                    if !error_msg().is_empty() {
                        div { class: "error-banner", "{error_msg}" }
                    }
                }

                // Right Column: Dashboard Results Display
                div { class: "glass-card",
                    if let Some(res) = result() {
                        div { class: "results-container",
                            h2 { class: "panel-title ruby",
                                span { "📊" }
                                "Simulation Results"
                            }

                            // Metrics grid
                            div { class: "summary-cards-grid",
                                div { class: "summary-card highlight-gold",
                                    span { class: "summary-card-label", "Average Cost" }
                                    span { class: "summary-card-value", "{format_mesos(res.average_cost)}" }
                                    span { class: "summary-card-subtext", "Total expected mesos spent" }
                                }
                                div { class: "summary-card",
                                    span { class: "summary-card-label", "Median Cost (50%)" }
                                    span { class: "summary-card-value", "{format_mesos(res.median_cost)}" }
                                    span { class: "summary-card-subtext", "50% of players hit within this" }
                                }
                                div { class: "summary-card highlight-ruby",
                                    span { class: "summary-card-label", "Expected Booms" }
                                    span { class: "summary-card-value", "{res.average_booms:.2}" }
                                    span { class: "summary-card-subtext", "Avg destructions per item" }
                                }
                                div { class: "summary-card highlight-ruby",
                                    span { class: "summary-card-label", "Median Booms" }
                                    span { class: "summary-card-value", "{res.median_booms}" }
                                    span { class: "summary-card-subtext", "50% of players boom ≤ this" }
                                }
                            }

                            // Desktop results grid split layout (Percentiles & Booms side-by-side, Friction full width below)
                            div { class: "desktop-results-grid",


                                // CDF Chart
                                div { class: "glass-card p-5 flex flex-col full-width-desktop", style: "min-height: 320px;",
                                    div { class: "mb-2 flex flex-col sm:flex-row sm:justify-between sm:items-center gap-2",
                                        div {
                                            h3 { class: "section-subtitle",
                                                span { "📈" }
                                                "Cumulative Success Probability (CDF)"
                                            }
                                            p { style: "font-size: 0.75rem; color: var(--text-muted); margin-top: 4px;",
                                                "Probability of achieving target star level within a given meso and equipment spare budget."
                                            }
                                        }
                                        div { class: "flex items-center gap-2 self-start sm:self-auto",
                                            span { style: "font-size: 0.75rem; color: var(--text-muted); font-weight: 500;", "Total Spare:" }
                                            div { style: "display: flex; gap: 4px; align-items: center;",
                                                button {
                                                    class: "preset-btn",
                                                    style: "max-width: 32px; padding: 4px 10px; font-size: 0.75rem;",
                                                    r#type: "button",
                                                    disabled: is_loading(),
                                                    onclick: decrement_equip,
                                                    "-"
                                                }
                                                input {
                                                    r#type: "number",
                                                    min: "1",
                                                    max: "100",
                                                    value: "{equipment_count}",
                                                    class: "input-control",
                                                    style: "width: 60px; padding: 4px 8px; font-size: 0.75rem; background: rgba(0, 0, 0, 0.3); border: 1px solid var(--border-color); color: #fff; border-radius: 4px; font-family: var(--font-mono); text-align: center;",
                                                    oninput: move |evt| {
                                                        if let Ok(val) = evt.value().parse::<u32>() {
                                                            if val >= 1 {
                                                                equipment_count.set(val);
                                                            }
                                                        }
                                                    }
                                                }
                                                button {
                                                    class: "preset-btn",
                                                    style: "max-width: 32px; padding: 4px 10px; font-size: 0.75rem;",
                                                    r#type: "button",
                                                    disabled: is_loading(),
                                                    onclick: increment_equip,
                                                    "+"
                                                }
                                            }
                                        }
                                    }
                                    div { style: "position: relative; flex-grow: 1; min-height: 240px; width: 100%; overflow: hidden;",
                                        canvas { id: "cdf-chart-canvas" }
                                    }
                                }

                                // Meso Cost Bottlenecks
                                div { class: "glass-card p-5 flex flex-col", style: "min-height: 320px;",
                                    div { class: "mb-2",
                                        h3 { class: "section-subtitle",
                                            span { "🪙" }
                                            "Meso Cost Bottlenecks"
                                        }
                                        p { style: "font-size: 0.75rem; color: var(--text-muted); margin-top: 4px;",
                                            "Average mesos spent isolated per star bracket."
                                        }
                                    }
                                    div { style: "position: relative; flex-grow: 1; min-height: 240px; width: 100%; overflow: hidden;",
                                        canvas { id: "bottlenecks-cost-canvas" }
                                    }
                                }

                                // Boom Bottlenecks
                                div { class: "glass-card p-5 flex flex-col", style: "min-height: 320px;",
                                    div { class: "mb-2",
                                        h3 { class: "section-subtitle",
                                            span { "💥" }
                                            "Boom Bottlenecks"
                                        }
                                        p { style: "font-size: 0.75rem; color: var(--text-muted); margin-top: 4px;",
                                            "Expected equipment destructions isolated per star bracket."
                                        }
                                    }
                                    div { style: "position: relative; flex-grow: 1; min-height: 240px; width: 100%; overflow: hidden;",
                                        canvas { id: "bottlenecks-boom-canvas" }
                                    }
                                }
                                // Luck Percentiles list card
                                div { class: "glass-card p-5 flex flex-col percentiles-section", style: "min-height: 320px;",
                                    h3 { class: "section-subtitle",
                                        span { "🍀" }
                                        "Luck Percentiles (Cost to Hit Target)"
                                    }
                                    div { class: "percentiles-list", style: "margin-top: 10px;",
                                        div { class: "percentile-row lucky",
                                            span { class: "percentile-name", "Top 10% Luck (Very Lucky)" }
                                            span { class: "percentile-value", "{format_mesos(res.percentile_10)}" }
                                        }
                                        div { class: "percentile-row lucky",
                                            span { class: "percentile-name", "Top 25% Luck (Lucky)" }
                                            span { class: "percentile-value", "{format_mesos(res.percentile_25)}" }
                                        }
                                        div { class: "percentile-row",
                                            span { class: "percentile-name", "50% Luck (Median / Average)" }
                                            span { class: "percentile-value", "{format_mesos(res.median_cost)}" }
                                        }
                                        div { class: "percentile-row unlucky",
                                            span { class: "percentile-name", "Bottom 25% Luck (Unlucky)" }
                                            span { class: "percentile-value", "{format_mesos(res.percentile_75)}" }
                                        }
                                        div { class: "percentile-row unlucky",
                                            span { class: "percentile-name", "Bottom 10% Luck (Very Unlucky)" }
                                            span { class: "percentile-value", "{format_mesos(res.percentile_90)}" }
                                        }
                                        div { class: "percentile-row unlucky",
                                            span { class: "percentile-name", "Bottom 1% Luck (Worst Case)" }
                                            span { class: "percentile-value", "{format_mesos(res.percentile_99)}" }
                                        }
                                    }
                                }


                                // Boom probabilities bar chart card
                                div { class: "glass-card p-5 flex flex-col boom-dist-section", style: "min-height: 320px;",
                                    h3 { class: "section-subtitle",
                                        span { "💥" }
                                        "Boom / Destruction Count Probability"
                                    }
                                    div { style: "position: relative; flex-grow: 1; min-height: 220px; width: 100%; margin-top: 10px; overflow: hidden;",
                                        canvas { id: "boom-chart-canvas" }
                                    }
                                }
                            }

                            if let Some(dur) = time_elaps() {
                                p { style: "font-size: 0.75rem; color: var(--text-muted); text-align: center; margin-top: 15px;",
                                    "Simulation computed in {format_duration(dur)} using Monte Carlo simulation"
                                }
                            }
                        }
                    } else {
                        div { class: "empty-state",
                            span { class: "empty-state-icon", "🎲" }
                            h3 { "Ready for Simulation" }
                            p { "Configure the parameters on the left and click 'Run Simulation' to calculate expected costs, percentile distributions, and bottlenecks." }
                        }
                    }
                }
            }
        } else {
            div { class: "canvas-page-grid",
                div { class: "canvas-workspace",
                    div { class: if is_panning() { "canvas-area grabbing" } else if is_space_pressed() { "canvas-area grab" } else { "canvas-area" },
                    style: "background-size: {24.0 * zoom_level()}px {24.0 * zoom_level()}px; background-position: {pan_x()}px {pan_y()}px;",
                    tabindex: 0,
                    onkeydown: move |evt| {
                        if evt.key().to_string() == " " {
                            is_space_pressed.set(true);
                        } else if evt.key() == Key::Delete {
                            if let Some(item_id) = selected_item_id() {
                                canvas_items.write().retain(|x| x.id != item_id);
                                connections.write().retain(|&(s, t)| s != item_id && t != item_id);
                                selected_item_id.set(None);
                            }
                        }
                    },
                    onkeyup: move |evt| {
                        if evt.key().to_string() == " " {
                            is_space_pressed.set(false);
                        }
                    },
                    onwheel: move |evt| {
                        evt.prevent_default();
                        let delta = evt.data().delta();
                        let dy = match delta {
                            WheelDelta::Pixels(vec) => vec.y,
                            WheelDelta::Lines(vec) => vec.y * 20.0,
                            WheelDelta::Pages(vec) => vec.y * 100.0,
                        };
                        let zoom_factor = if dy < 0.0 { 1.15f64 } else { 0.85f64 };
                        let cur_zoom = zoom_level();
                        let new_zoom = (cur_zoom * zoom_factor).clamp(0.15, 3.0);
                        
                        let coords = evt.data().element_coordinates();
                        let mouse_x = coords.x;
                        let mouse_y = coords.y;
                        
                        let ratio = new_zoom / cur_zoom;
                        let new_pan_x = mouse_x - (mouse_x - pan_x()) * ratio;
                        let new_pan_y = mouse_y - (mouse_y - pan_y()) * ratio;
                        
                        zoom_level.set(new_zoom);
                        pan_x.set(new_pan_x);
                        pan_y.set(new_pan_y);
                    },
                    onmousedown: move |evt| {
                        let is_middle = evt.data().held_buttons().contains(MouseButton::Auxiliary) || evt.data().trigger_button() == Some(MouseButton::Auxiliary);
                        let is_primary = evt.data().held_buttons().contains(MouseButton::Primary) || evt.data().trigger_button() == Some(MouseButton::Primary);
                        
                        selected_item_id.set(None);
                        active_source_pin.set(None);

                        if is_middle || is_space_pressed() || is_primary {
                            is_panning.set(true);
                            let coords = evt.data().client_coordinates();
                            pan_start_mouse.set((coords.x, coords.y));
                            pan_start_offset.set((pan_x(), pan_y()));
                        }
                    },
                    ondragover: move |evt| {
                        evt.prevent_default();
                    },
                    ondrop: move |evt| {
                        if let Some(idx) = dragged_palette_idx() {
                            let client = evt.data().client_coordinates();
                            let offset = canvas_client_offset();
                            
                            let canvas_x = if offset.0 > 0.0 { client.x - offset.0 } else { evt.data().element_coordinates().x };
                            let canvas_y = if offset.1 > 0.0 { client.y - offset.1 } else { evt.data().element_coordinates().y };
                            
                            let z = zoom_level();
                            let world_x = ((canvas_x - pan_x()) / z) as i32;
                            let world_y = ((canvas_y - pan_y()) / z) as i32;
                            
                            let id = next_id();
                            next_id.set(id + 1);
                            
                            let drop_x = world_x - 105;
                            let drop_y = world_y - 40;
                            
                            if idx == 999 {
                                let (cost, booms, med_cost, med_booms) = simulate_equip(0, 22, 140, true, true, true, true, [EnhancementMode::Level1; 7], canvas_trials());
                                canvas_items.write().push(CanvasItem {
                                    id,
                                    name: "Custom Equipment".to_string(),
                                    level: 140,
                                    png_url: "custom".to_string(),
                                    is_custom: true,
                                    start_stars: 0,
                                    target_stars: 22,
                                    safeguard: true,
                                    star_catch: true,
                                    ssf_cost: true,
                                    ssf_boom: true,
                                    mode_15_21: [EnhancementMode::Level1; 7],
                                    x: drop_x,
                                    y: drop_y,
                                    avg_cost: cost,
                                    avg_booms: booms,
                                    median_cost: med_cost,
                                    median_booms: med_booms,
                                });
                            } else if idx < TEMPLATES.len() {
                                let template = &TEMPLATES[idx];
                                let (cost, booms, med_cost, med_booms) = simulate_equip(0, 22, template.level, true, true, true, true, [EnhancementMode::Level1; 7], canvas_trials());
                                canvas_items.write().push(CanvasItem {
                                    id,
                                    name: template.name.to_string(),
                                    level: template.level,
                                    png_url: template.asset.to_string(),
                                    is_custom: false,
                                    start_stars: 0,
                                    target_stars: 22,
                                    safeguard: true,
                                    star_catch: true,
                                    ssf_cost: true,
                                    ssf_boom: true,
                                    mode_15_21: [EnhancementMode::Level1; 7],
                                    x: drop_x,
                                    y: drop_y,
                                    avg_cost: cost,
                                    avg_booms: booms,
                                    median_cost: med_cost,
                                    median_booms: med_booms,
                                });
                            }
                            selected_item_id.set(Some(id));
                            is_properties_open.set(true); // Auto-open properties
                            dragged_palette_idx.set(None);
                        }
                    },
                    onmousemove: move |evt| {
                        let client = evt.data().client_coordinates();
                        let elem = evt.data().element_coordinates();
                        if elem.x > 0.0 && elem.y > 0.0 {
                            canvas_client_offset.set((client.x - elem.x, client.y - elem.y));
                        }

                        if is_panning() {
                            let coords = evt.data().client_coordinates();
                            let dx = coords.x - pan_start_mouse().0;
                            let dy = coords.y - pan_start_mouse().1;
                            pan_x.set(pan_start_offset().0 + dx);
                            pan_y.set(pan_start_offset().1 + dy);
                        } else if let Some(id) = dragging_id() {
                            let coords = evt.data().client_coordinates();
                            let start_mouse = drag_start_mouse();
                            let start_item = drag_start_item();
                            
                            let z = zoom_level();
                            let dx = ((coords.x - start_mouse.0) / z) as i32;
                            let dy = ((coords.y - start_mouse.1) / z) as i32;
                            
                            let pos = canvas_items.read().iter().position(|item| item.id == id);
                            if let Some(pos) = pos {
                                let mut items = canvas_items.write();
                                items[pos].x = start_item.0 + dx;
                                items[pos].y = start_item.1 + dy;
                            }
                        } else if active_source_pin().is_some() {
                            let coords = evt.data().client_coordinates();
                            let start_mouse = drag_start_mouse();
                            let start_item = drag_start_item();
                            let z = zoom_level();
                            let dx = (coords.x - start_mouse.0) / z;
                            let dy = (coords.y - start_mouse.1) / z;
                            draft_mouse_coords.set(((start_item.0 as f64 + dx) as i32, (start_item.1 as f64 + dy) as i32));
                        }
                    },
                    onmouseup: move |_| {
                        is_panning.set(false);
                        dragging_id.set(None);
                        active_source_pin.set(None);
                    },
                    onmouseleave: move |_| {
                        is_panning.set(false);
                        dragging_id.set(None);
                    },

                    // Floating Zoom Controls HUD Toolbar
                    div { class: "canvas-zoom-hud",
                        button {
                            class: "hud-btn",
                            r#type: "button",
                            title: "Zoom Out",
                            onclick: move |evt| {
                                let z = (zoom_level() / 1.15).max(0.15);
                                zoom_level.set(z);
                                evt.stop_propagation();
                            },
                            "−"
                        }
                        span { class: "hud-zoom-label", "{(zoom_level() * 100.0).round() as u32}%" }
                        button {
                            class: "hud-btn",
                            r#type: "button",
                            title: "Zoom In",
                            onclick: move |evt| {
                                let z = (zoom_level() * 1.15).min(3.0);
                                zoom_level.set(z);
                                evt.stop_propagation();
                            },
                            "+"
                        }
                        div { class: "hud-divider" }
                        button {
                            class: "hud-btn text-btn",
                            r#type: "button",
                            title: "Reset View (1:1)",
                            onclick: move |evt| {
                                zoom_level.set(1.0);
                                pan_x.set(0.0);
                                pan_y.set(0.0);
                                evt.stop_propagation();
                            },
                            "1:1"
                        }
                        button {
                            class: "hud-btn text-btn",
                            r#type: "button",
                            title: "Fit View to Content",
                            onclick: move |evt| {
                                let items = canvas_items();
                                if !items.is_empty() {
                                    let mut min_x = i32::MAX;
                                    let mut max_x = i32::MIN;
                                    let mut min_y = i32::MAX;
                                    let mut max_y = i32::MIN;
                                    for it in &items {
                                        min_x = min_x.min(it.x);
                                        max_x = max_x.max(it.x + 210);
                                        min_y = min_y.min(it.y);
                                        max_y = max_y.max(it.y + 130);
                                    }
                                    let content_w = (max_x - min_x) as f64 + 100.0;
                                    let content_h = (max_y - min_y) as f64 + 100.0;
                                    let target_scale_x = 800.0 / content_w;
                                    let target_scale_y = 600.0 / content_h;
                                    let fit_zoom = target_scale_x.min(target_scale_y).clamp(0.2, 1.5);
                                    let target_pan_x = (800.0 - (min_x as f64 + max_x as f64) * fit_zoom) / 2.0;
                                    let target_pan_y = (600.0 - (min_y as f64 + max_y as f64) * fit_zoom) / 2.0;
                                    zoom_level.set(fit_zoom);
                                    pan_x.set(target_pan_x);
                                    pan_y.set(target_pan_y);
                                }
                                evt.stop_propagation();
                            },
                            "Fit"
                        }
                    }

                    // World Transformation Layer
                    div {
                        class: "canvas-world-layer",
                        style: "transform: translate({pan_x()}px, {pan_y()}px) scale({zoom_level()}); transform-origin: 0 0;",

                        // SVG overlay for connection lines
                        svg {
                            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 5; overflow: visible;",
                            
                            // Render established connections
                            {
                                connections.read().iter().filter_map(|&(source_id, target_id)| {
                                    let source_opt = canvas_items().iter().find(|x| x.id == source_id).cloned();
                                    let target_opt = canvas_items().iter().find(|x| x.id == target_id).cloned();
                                    if let (Some(src), Some(tgt)) = (source_opt, target_opt) {
                                        let x1 = (src.x + 210) as f64;
                                        let y1 = (src.y + 75) as f64;
                                        let x2 = tgt.x as f64;
                                        let mut y2 = (tgt.y + 75) as f64;
                                        if (y2 - y1).abs() < 0.1 {
                                            y2 += 0.1;
                                        }
                                        let path_data = if (y2 - y1).abs() < 2.0 || (x2 - x1).abs() < 2.0 {
                                            format!("M {} {} L {} {}", x1, y1, x2, y2)
                                        } else {
                                            let dx = (x2 - x1).abs() / 2.0;
                                            format!("M {} {} C {} {}, {} {}, {} {}", x1, y1, x1 + dx, y1, x2 - dx, y2, x2, y2)
                                        };
                                        
                                        Some(rsx! {
                                            path {
                                                d: "{path_data}",
                                                stroke: "url(#neon-gold-gradient)",
                                                stroke_width: "3",
                                                fill: "none",
                                                style: "filter: drop-shadow(0 0 4px rgba(226, 180, 76, 0.4));",
                                            }
                                            circle {
                                                cx: (x1 + x2) / 2.0,
                                                cy: (y1 + y2) / 2.0,
                                                r: "8",
                                                fill: "rgba(233, 69, 96, 0.9)",
                                                stroke: "#ffffff",
                                                stroke_width: "1.5",
                                                style: "cursor: pointer; pointer-events: auto;",
                                                onclick: move |evt| {
                                                    connections.write().retain(|&edge| edge != (source_id, target_id));
                                                    evt.stop_propagation();
                                                },
                                            }
                                            text {
                                                x: (x1 + x2) / 2.0,
                                                y: (y1 + y2) / 2.0 + 4.0,
                                                text_anchor: "middle",
                                                fill: "#ffffff",
                                                font_size: "10px",
                                                font_weight: "bold",
                                                style: "cursor: pointer; pointer-events: none;",
                                                "×"
                                            }
                                        })
                                    } else {
                                        None
                                    }
                                })
                            }
                            
                            // Render draft connection preview
                            {
                                if let Some(src_id) = active_source_pin() {
                                    canvas_items().iter().find(|x| x.id == src_id).map(|src| {
                                        let x1 = (src.x + 210) as f64;
                                        let y1 = (src.y + 75) as f64;
                                        let (x2, mut y2) = {
                                            let coords = draft_mouse_coords();
                                            (coords.0 as f64, coords.1 as f64)
                                        };
                                        if (y2 - y1).abs() < 0.1 {
                                            y2 += 0.1;
                                        }
                                        let path_data = if (y2 - y1).abs() < 2.0 || (x2 - x1).abs() < 2.0 {
                                            format!("M {} {} L {} {}", x1, y1, x2, y2)
                                        } else {
                                            let dx = (x2 - x1).abs() / 2.0;
                                            format!("M {} {} C {} {}, {} {}, {} {}", x1, y1, x1 + dx, y1, x2 - dx, y2, x2, y2)
                                        };
                                        
                                        rsx! {
                                            path {
                                                d: "{path_data}",
                                                stroke: "var(--color-gold)",
                                                stroke_width: "2.5",
                                                stroke_dasharray: "5,5",
                                                fill: "none",
                                            }
                                        }
                                    })
                                } else {
                                    None
                                }
                            }
                            
                            defs {
                                linearGradient {
                                    id: "neon-gold-gradient",
                                    x1: "0%", y1: "0%", x2: "100%", y2: "0%",
                                    stop { offset: "0%", stop_color: "var(--color-gold)" }
                                    stop { offset: "100%", stop_color: "#f59e0b" }
                                }
                            }
                        }

                        if canvas_items().is_empty() {
                            div { class: "canvas-placeholder",
                                span { class: "canvas-placeholder-icon", "🎨" }
                                span { class: "canvas-placeholder-text", "Drag equipment from the palette here, or click them to place them on the canvas grid." }
                            }
                        }

                        for item in canvas_items() {
                            div {
                                key: "{item.id}",
                                class: if selected_item_id() == Some(item.id) { "canvas-equip-card active" } else { "canvas-equip-card" },
                                style: if dragging_id() == Some(item.id) {
                                    "left: {item.x}px; top: {item.y}px; opacity: 0.8; z-index: 100;"
                                } else {
                                    "left: {item.x}px; top: {item.y}px;"
                                },
                                onmousedown: move |evt| {
                                    let item_id = item.id;
                                    dragging_id.set(Some(item_id));
                                    selected_item_id.set(Some(item_id));
                                    is_properties_open.set(true); // Open properties panel on item click
                                    let client_coords = evt.data().client_coordinates();
                                    drag_start_mouse.set((client_coords.x, client_coords.y));
                                    drag_start_item.set((item.x, item.y));
                                    evt.stop_propagation();
                                },

                                // Input Pin (Left Edge)
                                div {
                                    class: "canvas-card-pin input-pin",
                                    title: "Release mouse to connect",
                                    onmousedown: move |evt| {
                                        evt.stop_propagation();
                                    },
                                    onmouseup: move |evt| {
                                        if let Some(source_id) = active_source_pin() {
                                            let target_id = item.id;
                                            if source_id != target_id {
                                                let src_opt = canvas_items().iter().find(|x| x.id == source_id).cloned();
                                                if let Some(src) = src_opt {
                                                    if item.level <= src.level {
                                                        canvas_error.set("Invalid: Target level must be higher than source level.".to_string());
                                                    } else if item.level - src.level != 10 {
                                                        canvas_error.set("Invalid: Level difference must be exactly 10.".to_string());
                                                    } else if connections.read().iter().any(|&(_, t)| t == target_id) {
                                                        canvas_error.set("Invalid: Target item already has an incoming connection.".to_string());
                                                    } else if connections.read().iter().any(|&(s, _)| s == source_id) {
                                                        canvas_error.set("Invalid: Source item already has an outgoing connection.".to_string());
                                                    } else {
                                                        connections.write().push((source_id, target_id));
                                                        let mut items_write = canvas_items.write();
                                                        propagate_canvas_items(&mut items_write, &connections(), canvas_trials());
                                                        canvas_error.set(String::new());
                                                    }
                                                    
                                                    if !canvas_error().is_empty() {
                                                        spawn(async move {
                                                            sleep(Duration::from_secs(4)).await;
                                                            canvas_error.set(String::new());
                                                        });
                                                    }
                                                }
                                            }
                                            active_source_pin.set(None);
                                        }
                                        evt.stop_propagation();
                                    }
                                }

                                // Output Pin (Right Edge)
                                div {
                                    class: "canvas-card-pin output-pin",
                                    title: "Drag or click to connect",
                                    onmousedown: move |evt| {
                                        active_source_pin.set(Some(item.id));
                                        let client = evt.data().client_coordinates();
                                        drag_start_mouse.set((client.x, client.y));
                                        drag_start_item.set((item.x + 210, item.y + 75));
                                        draft_mouse_coords.set((item.x + 210, item.y + 75));
                                        evt.stop_propagation();
                                    },
                                    onmouseup: move |evt| {
                                        evt.stop_propagation();
                                    }
                                }

                                div { class: "canvas-equip-card-header",
                                    if item.is_custom {
                                        span { style: "font-size: 1.2rem; margin-right: 6px; display: flex; align-items: center;", "🛠️" }
                                    } else {
                                        img {
                                            src: "{item.png_url}",
                                            style: "width: 24px; height: 24px; object-fit: contain; margin-right: 6px; border-radius: 4px;"
                                        }
                                    }
                                    span { class: "canvas-equip-card-title", "{item.name}" }
                                    button {
                                        class: "canvas-equip-card-delete-btn",
                                        title: "Delete equipment",
                                        onclick: move |evt| {
                                            let id = item.id;
                                            canvas_items.write().retain(|x| x.id != id);
                                            connections.write().retain(|&(s, t)| s != id && t != id);
                                            if selected_item_id() == Some(id) {
                                                selected_item_id.set(None);
                                            }
                                            evt.stop_propagation();
                                        },
                                        "×"
                                    }
                                }
                                div { style: "display: flex; gap: 6px; align-items: center; flex-wrap: wrap;",
                                    span { class: "canvas-equip-card-badge", "Lv. {item.level}" }
                                    if item.ssf_cost || item.ssf_boom {
                                        span { class: "canvas-equip-card-ssf-badge", "⚡ SSF" }
                                    }
                                    {
                                        let display_lbl = if item.mode_15_21.iter().all(|&m| m == EnhancementMode::Level1) {
                                            "L1".to_string()
                                        } else if item.mode_15_21.iter().all(|&m| m == EnhancementMode::Level2) {
                                            "L2".to_string()
                                        } else if item.mode_15_21.iter().all(|&m| m == EnhancementMode::Level3) {
                                            "L3".to_string()
                                        } else if item.mode_15_21.iter().all(|&m| m == EnhancementMode::Level4) {
                                            "L4".to_string()
                                        } else {
                                            let get_char = |m: EnhancementMode| match m {
                                                EnhancementMode::Level1 => "1",
                                                EnhancementMode::Level2 => "2",
                                                EnhancementMode::Level3 => "3",
                                                EnhancementMode::Level4 => "4",
                                                _ => "S",
                                            };
                                            format!("L{}{}{}{}",
                                                get_char(item.mode_15_21[3]),
                                                get_char(item.mode_15_21[4]),
                                                get_char(item.mode_15_21[5]),
                                                get_char(item.mode_15_21[6]),
                                            )
                                        };
                                        rsx! {
                                            span { 
                                                class: "canvas-equip-card-badge", 
                                                style: "background: rgba(79, 70, 229, 0.15); border: 1px solid rgba(79, 70, 229, 0.3); color: #818cf8; font-weight: 600;",
                                                "{display_lbl}"
                                            }
                                        }
                                    }
                                }
                                div { class: "canvas-equip-card-stars",
                                    span { class: "star-val", "{item.start_stars} ★" }
                                    span { class: "arrow", "➔" }
                                    span { class: "star-val target", "{item.target_stars} ★" }
                                }
                                div { class: "canvas-equip-card-metrics",
                                    div { class: "metric-item",
                                        span { class: "metric-label", "Cost" }
                                        span { class: "metric-val gold",
                                            {
                                                let val = if cost_display_mode() == DisplayMode::Median {
                                                    item.median_cost
                                                } else {
                                                    item.avg_cost
                                                };
                                                format_mesos(val)
                                            }
                                        }
                                    }
                                    div { class: "metric-item",
                                        span { class: "metric-label", "Booms" }
                                        span { class: "metric-val ruby",
                                            {
                                                let val = if booms_display_mode() == DisplayMode::Median {
                                                    item.median_booms
                                                } else {
                                                    item.avg_booms
                                                };
                                                format!("{:.2}", val)
                                            }
                                        }
                                    }
                                }
                            }

                            // Equipment Set Summary Cards (rendered in World Space)
                            {
                                let directed_adj = {
                                    let mut adj = std::collections::HashMap::<usize, Vec<usize>>::new();
                                    for item in canvas_items().iter() {
                                        adj.insert(item.id, Vec::new());
                                    }
                                    for &(src, tgt) in connections().iter() {
                                        if adj.contains_key(&src) && adj.contains_key(&tgt) {
                                            adj.get_mut(&src).unwrap().push(tgt);
                                        }
                                    }
                                    adj
                                };

                                let sets = {
                                    let mut visited = std::collections::HashSet::<usize>::new();
                                    let mut result_sets = Vec::<(Vec<usize>, u128, f64)>::new();
                                    let items = canvas_items();
                                    let conn_list = connections();

                                    let mut undir_adj = std::collections::HashMap::<usize, Vec<usize>>::new();
                                    for item in items.iter() {
                                        undir_adj.insert(item.id, Vec::new());
                                    }
                                    for &(src, tgt) in conn_list.iter() {
                                        if undir_adj.contains_key(&src) && undir_adj.contains_key(&tgt) {
                                            undir_adj.get_mut(&src).unwrap().push(tgt);
                                            undir_adj.get_mut(&tgt).unwrap().push(src);
                                        }
                                    }

                                    for item in items.iter() {
                                        let id = item.id;
                                        if !visited.contains(&id) {
                                            let mut comp = Vec::new();
                                            let mut queue = std::collections::VecDeque::new();
                                            queue.push_back(id);
                                            visited.insert(id);

                                            while let Some(node) = queue.pop_front() {
                                                comp.push(node);
                                                if let Some(edges) = undir_adj.get(&node) {
                                                    for &next_node in edges.iter() {
                                                        if !visited.contains(&next_node) {
                                                            visited.insert(next_node);
                                                            queue.push_back(next_node);
                                                        }
                                                    }
                                                }
                                            }

                                            if comp.len() >= 1 {
                                                let mut total_cost = 0;
                                                let mut total_booms = 0.0;
                                                for &node_id in comp.iter() {
                                                    if let Some(comp_item) = items.iter().find(|x| x.id == node_id) {
                                                        total_cost += comp_item.avg_cost;
                                                        total_booms += comp_item.avg_booms;
                                                    }
                                                }
                                                result_sets.push((comp, total_cost, total_booms));
                                            }
                                        }
                                    }
                                    result_sets
                                };

                                let summary_elements = sets.iter().flat_map(|(comp, total_cost, _)| {
                                    let directed_adj = &directed_adj;
                                    comp.iter().filter_map(move |&node_id| {
                                        let is_last = directed_adj.get(&node_id).map_or(false, |out_edges| out_edges.is_empty());
                                        if is_last {
                                            canvas_items().iter().find(|x| x.id == node_id).cloned().map(|last_item| {
                                                let summary_x = last_item.x + 230;
                                                let summary_y = last_item.y;
                                                let num_items = comp.len();
                                                let t_cost = *total_cost;
                                                
                                                let mut comp_items = Vec::new();
                                                for &cid in comp.iter() {
                                                    if let Some(citem) = canvas_items().iter().find(|x| x.id == cid).cloned() {
                                                        comp_items.push(citem);
                                                    }
                                                }
                                                comp_items.sort_by_key(|x| x.level);

                                                rsx! {
                                                    div {
                                                        key: "summary-{node_id}",
                                                        class: "canvas-set-summary-card",
                                                        style: "left: {summary_x}px; top: {summary_y}px;",
                                                        div { class: "set-summary-header", "📦 Equipment Set" }
                                                        div { class: "set-summary-item",
                                                            span { class: "set-summary-label", "Items:" }
                                                            span { class: "set-summary-value", "{num_items}" }
                                                        }
                                                        div { class: "set-summary-item",
                                                            span { class: "set-summary-label", "Total Cost:" }
                                                            {
                                                                let display_cost = if cost_display_mode() == DisplayMode::Median {
                                                                    let mut sum = 0u128;
                                                                    for &cid in comp.iter() {
                                                                        if let Some(ci) = canvas_items().iter().find(|x| x.id == cid) {
                                                                            sum += ci.median_cost;
                                                                        }
                                                                    }
                                                                    sum
                                                                } else {
                                                                    t_cost
                                                                };
                                                                rsx! {
                                                                    span { class: "set-summary-value gold", "{format_mesos(display_cost)}" }
                                                                }
                                                            }
                                                        }
                                                        div { 
                                                            class: "set-summary-item", 
                                                            style: "flex-direction: column; align-items: stretch; margin-top: 6px; gap: 4px; border-top: 1px dashed rgba(16, 185, 129, 0.2); padding-top: 6px;",
                                                            div { class: "set-summary-item", style: "font-size: 0.7rem; color: var(--text-muted); font-weight: 700; text-transform: uppercase; margin-bottom: 2px;",
                                                                span { "Expected Booms:" }
                                                            }
                                                            for comp_item in comp_items.iter() {
                                                                div { class: "set-summary-item", style: "font-size: 0.75rem; display: flex; align-items: center;",
                                                                    if comp_item.is_custom {
                                                                        span { style: "font-size: 0.9rem; margin-right: 4px; display: flex; align-items: center;", "🛠️" }
                                                                    } else {
                                                                        img {
                                                                            src: "{comp_item.png_url}",
                                                                            style: "width: 16px; height: 16px; object-fit: contain; margin-right: 4px; border-radius: 2px;"
                                                                        }
                                                                    }
                                                                    span { class: "set-summary-label", "{comp_item.name} (Lv. {comp_item.level}):" }
                                                                    span { class: "set-summary-value ruby",
                                                                        {
                                                                            let v = if booms_display_mode() == DisplayMode::Median {
                                                                                comp_item.median_booms
                                                                            } else {
                                                                                comp_item.avg_booms
                                                                            };
                                                                            format!("{:.3}", v)
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                });

                                rsx! {
                                    {summary_elements}
                                }
                            }
                        }
                    }
                }

                    // Floating Top Instruction Guide
                    div { class: "floating-summary-bar", style: "border-color: rgba(16, 185, 129, 0.3); background: rgba(5, 7, 18, 0.85); width: auto; max-width: 90%; display: flex; align-items: center; justify-content: space-between; gap: 20px;",
                        span { style: "font-size: 0.75rem; color: #10b981; font-weight: 700; display: flex; align-items: center; gap: 6px;",
                            span { "🔌" }
                            "Click output pin (right) then click input pin (left) of another card to connect. Click connection dot to delete."
                        }
                        button {
                            class: "reset-canvas-btn",
                            style: "background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4); color: #f87171; font-size: 0.75rem; font-weight: 700; padding: 4px 10px; border-radius: 12px; cursor: pointer; transition: all var(--transition-fast); display: flex; align-items: center; gap: 4px;",
                            onclick: move |_| {
                                canvas_items.write().clear();
                                connections.write().clear();
                                selected_item_id.set(None);
                                active_source_pin.set(None);
                            },
                            span { "🗑️" }
                            "Reset Canvas"
                        }
                    }

                    // Canvas Global Config Panel
                    div { class: "canvas-config-panel",
                        // Trials row
                        div { class: "canvas-config-row",
                            span { class: "canvas-config-label", "🎲 Trials" }
                            div { class: "canvas-config-chips",
                                for (val, lbl) in [
                                    (10_000_u32, "10K"),
                                    (50_000_u32, "50K"),
                                    (100_000_u32, "100K"),
                                    (500_000_u32, "500K"),
                                    (1_000_000_u32, "1M"),
                                ] {
                                    button {
                                        class: if canvas_trials() == val { "config-chip active" } else { "config-chip" },
                                        r#type: "button",
                                        onclick: move |_| {
                                            canvas_trials.set(val);
                                            // Re-simulate all canvas items
                                            let mut items = canvas_items.write();
                                            for item in items.iter_mut() {
                                                let (cost, booms, med_cost, med_booms) = simulate_equip(
                                                    item.start_stars, item.target_stars, item.level,
                                                    item.safeguard, item.star_catch, item.ssf_cost,
                                                    item.ssf_boom, item.mode_15_21, val,
                                                );
                                                item.avg_cost = cost;
                                                item.avg_booms = booms;
                                                item.median_cost = med_cost;
                                                item.median_booms = med_booms;
                                            }
                                        },
                                        "{lbl}"
                                    }
                                }
                            }
                        }
                        // Display mode row
                        div { class: "canvas-config-row",
                            span { class: "canvas-config-label", "💰 Cost" }
                            div { class: "canvas-config-chips",
                                button {
                                    class: if cost_display_mode() == DisplayMode::Average { "config-chip active" } else { "config-chip" },
                                    r#type: "button",
                                    onclick: move |_| cost_display_mode.set(DisplayMode::Average),
                                    "Avg"
                                }
                                button {
                                    class: if cost_display_mode() == DisplayMode::Median { "config-chip active" } else { "config-chip" },
                                    r#type: "button",
                                    onclick: move |_| cost_display_mode.set(DisplayMode::Median),
                                    "Median"
                                }
                            }
                        }
                        div { class: "canvas-config-row",
                            span { class: "canvas-config-label", "💥 Booms" }
                            div { class: "canvas-config-chips",
                                button {
                                    class: if booms_display_mode() == DisplayMode::Average { "config-chip active" } else { "config-chip" },
                                    r#type: "button",
                                    onclick: move |_| booms_display_mode.set(DisplayMode::Average),
                                    "Avg"
                                }
                                button {
                                    class: if booms_display_mode() == DisplayMode::Median { "config-chip active" } else { "config-chip" },
                                    r#type: "button",
                                    onclick: move |_| booms_display_mode.set(DisplayMode::Median),
                                    "Median"
                                }
                            }
                        }
                    }

                    if !canvas_error().is_empty() {
                        div { class: "canvas-warning-toast",
                            span { "⚠️" }
                            span { "{canvas_error}" }
                        }
                    }

                    // Floating Toggle Buttons (if panel is closed)
                    if !is_palette_open() {
                        button {
                            class: "panel-toggle-btn left-btn",
                            r#type: "button",
                            onclick: move |_| is_palette_open.set(true),
                            title: "Open Equipment Palette",
                            "📦"
                        }
                    }

                    if !is_properties_open() && selected_item_id().is_some() {
                        button {
                            class: "panel-toggle-btn right-btn",
                            r#type: "button",
                            onclick: move |_| is_properties_open.set(true),
                            title: "Open Item Properties",
                            "✏️"
                        }
                    }

                    // Floating Left Panel: Equipment Palette
                    div {
                        class: if is_palette_open() { "floating-left-panel" } else { "floating-left-panel collapsed" },
                        div { class: "close-panel-header",
                            h2 { class: "panel-title gold", style: "margin: 0;",
                                span { "📦 " }
                                "Palette"
                            }
                            button {
                                class: "close-panel-btn",
                                r#type: "button",
                                onclick: move |_| is_palette_open.set(false),
                                "×"
                            }
                        }
                        p { style: "font-size: 0.75rem; color: var(--text-muted); margin-bottom: 5px;",
                            "Drag items or click '+' to place."
                        }
                        div { class: "palette-scroll",
                            // Custom Free-Form Item
                            div {
                                class: "palette-item",
                                style: "border-color: rgba(167, 139, 250, 0.4); background: rgba(167, 139, 250, 0.05);",
                                draggable: true,
                                ondragstart: move |_| {
                                    dragged_palette_idx.set(Some(999)); // index 999 stands for custom free-form item
                                },
                                onclick: move |_| {
                                    let id = next_id();
                                    next_id.set(id + 1);
                                    
                                    let count = canvas_items().len() as i32;
                                    let x = 320 + (count * 25) % 200;
                                    let y = 100 + (count * 20) % 200;
                                    
                                    let (cost, booms, med_cost, med_booms) = simulate_equip(0, 22, 140, true, true, true, true, [EnhancementMode::Level1; 7], canvas_trials());
                                    
                                    canvas_items.write().push(CanvasItem {
                                        id,
                                        name: "Custom Equipment".to_string(),
                                        level: 140,
                                        png_url: "custom".to_string(),
                                        is_custom: true,
                                        start_stars: 0,
                                        target_stars: 22,
                                        safeguard: true,
                                        star_catch: true,
                                        ssf_cost: true,
                                        ssf_boom: true,
                                        mode_15_21: [EnhancementMode::Level1; 7],
                                        x,
                                        y,
                                        avg_cost: cost,
                                        avg_booms: booms,
                                        median_cost: med_cost,
                                        median_booms: med_booms,
                                    });
                                    selected_item_id.set(Some(id));
                                    is_properties_open.set(true);
                                },
                                div { class: "palette-item-icon",
                                    style: "font-size: 1.25rem; display: flex; align-items: center; justify-content: center;",
                                    "🛠️"
                                }
                                div { class: "palette-item-info",
                                    span { class: "palette-item-name", style: "color: #c084fc;", "Custom Equipment" }
                                    span { class: "palette-item-level", "Freeform Level & Name" }
                                }
                                button { class: "palette-item-add-btn", style: "background: #a78bfa; color: #111827;", "+" }
                            }

                            for (idx, template) in TEMPLATES.iter().enumerate() {
                                div {
                                    class: "palette-item",
                                    draggable: true,
                                    ondragstart: move |_| {
                                        dragged_palette_idx.set(Some(idx));
                                    },
                                    onclick: move |_| {
                                        let id = next_id();
                                        next_id.set(id + 1);
                                        
                                        let count = canvas_items().len() as i32;
                                        let x = 320 + (count * 25) % 200;
                                        let y = 100 + (count * 20) % 200;
                                        
                                        let (cost, booms, med_cost, med_booms) = simulate_equip(0, 22, template.level, true, true, true, true, [EnhancementMode::Level1; 7], canvas_trials());
                                        
                                        canvas_items.write().push(CanvasItem {
                                            id,
                                            name: template.name.to_string(),
                                            level: template.level,
                                            png_url: template.asset.to_string(),
                                            is_custom: false,
                                            start_stars: 0,
                                            target_stars: 22,
                                            safeguard: true,
                                            star_catch: true,
                                            ssf_cost: true,
                                            ssf_boom: true,
                                            mode_15_21: [EnhancementMode::Level1; 7],
                                            x,
                                            y,
                                            avg_cost: cost,
                                            avg_booms: booms,
                                            median_cost: med_cost,
                                            median_booms: med_booms,
                                        });
                                        selected_item_id.set(Some(id));
                                        is_properties_open.set(true);
                                    },
                                    div { class: "palette-item-icon",
                                        img {
                                            src: template.asset,
                                            style: "width: 24px; height: 24px; object-fit: contain;"
                                        }
                                    }
                                    div { class: "palette-item-info",
                                        span { class: "palette-item-name", "{template.name}" }
                                        span { class: "palette-item-level", "Lv. {template.level}" }
                                    }
                                    button { class: "palette-item-add-btn", "+" }
                                }
                            }
                        }
                    }

                    // Floating Right Panel: Item Properties
                    if let Some(idx) = canvas_items().iter().position(|x| x.id == selected_item_id().unwrap_or(0)) {
                        {
                            let item = canvas_items.read()[idx].clone();
                            rsx! {
                                div {
                                    class: if is_properties_open() { "floating-right-panel" } else { "floating-right-panel collapsed" },
                                    div { class: "close-panel-header",
                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            if item.is_custom {
                                                span { style: "font-size: 1.2rem; display: flex; align-items: center;", "🛠️" }
                                            } else {
                                                img {
                                                    src: "{item.png_url}",
                                                    style: "width: 24px; height: 24px; object-fit: contain; border-radius: 4px;"
                                                }
                                            }
                                            h2 { class: "panel-title ruby", style: "margin: 0;",
                                                "Properties"
                                            }
                                        }
                                        button {
                                            class: "close-panel-btn",
                                            r#type: "button",
                                            onclick: move |_| is_properties_open.set(false),
                                            "×"
                                        }
                                    }
                                    
                                    div { style: "display: flex; flex-direction: column; gap: 15px;",
                                        div { class: "form-group",
                                            label { class: "input-label", "Item Name" }
                                            input {
                                                class: "input-control",
                                                value: "{item.name}",
                                                disabled: !item.is_custom,
                                                oninput: move |evt| {
                                                    let val = evt.value();
                                                    canvas_items.write()[idx].name = val;
                                                }
                                            }
                                        }
                                        
                                        div { class: "form-group",
                                            label { class: "input-label", "Item Level" }
                                            {
                                                let incoming_src_level = connections().iter()
                                                    .find(|&&(_, t)| t == item.id)
                                                    .and_then(|&(s, _)| canvas_items().iter().find(|x| x.id == s).map(|x| x.level));
                                                
                                                let outgoing_tgt_level = connections().iter()
                                                    .find(|&&(s, _)| s == item.id)
                                                    .and_then(|&(_, t)| canvas_items().iter().find(|x| x.id == t).map(|x| x.level));
                                                
                                                let is_connected = connections().iter().any(|&(s, t)| s == item.id || t == item.id);
                                                
                                                let disable_140 = incoming_src_level.map(|l| 140 <= l).unwrap_or(false) || outgoing_tgt_level.map(|l| 140 >= l).unwrap_or(false);
                                                let disable_150 = incoming_src_level.map(|l| 150 <= l).unwrap_or(false) || outgoing_tgt_level.map(|l| 150 >= l).unwrap_or(false);
                                                let disable_160 = incoming_src_level.map(|l| 160 <= l).unwrap_or(false) || outgoing_tgt_level.map(|l| 160 >= l).unwrap_or(false);
                                                let disable_200 = is_connected || incoming_src_level.map(|l| 200 <= l).unwrap_or(false) || outgoing_tgt_level.map(|l| 200 >= l).unwrap_or(false);
                                                let disable_250 = is_connected || incoming_src_level.map(|l| 250 <= l).unwrap_or(false) || outgoing_tgt_level.map(|l| 250 >= l).unwrap_or(false);
                                                
                                                rsx! {
                                                    select {
                                                        class: "select-control",
                                                        value: "{item.level}",
                                                        disabled: !item.is_custom,
                                                        onchange: move |evt| {
                                                            let level = evt.value().parse::<u32>().unwrap_or(200);
                                                            let mut items = canvas_items.write();
                                                            items[idx].level = level;
                                                            let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, items[idx].target_stars, level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                            items[idx].avg_cost = cost;
                                                            items[idx].avg_booms = booms;
                                                            items[idx].median_cost = med_cost;
                                                            items[idx].median_booms = med_booms;
                                                            propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                        },
                                                        option { value: "140", disabled: disable_140, "Lv. 140" }
                                                        option { value: "150", disabled: disable_150, "Lv. 150" }
                                                        option { value: "160", disabled: disable_160, "Lv. 160" }
                                                        option { value: "200", disabled: disable_200, "Lv. 200" }
                                                        option { value: "250", disabled: disable_250, "Lv. 250" }
                                                    }
                                                }
                                            }
                                        }

                                        {
                                            let has_input = connections().iter().any(|&(_, t)| t == item.id);
                                            rsx! {
                                                div { class: "form-group",
                                                    div { class: "label-container",
                                                        label { class: "input-label", "Start Stars" }
                                                        if has_input {
                                                            span { class: "input-value-badge inherited", "⛓️ Inherited" }
                                                        } else {
                                                            span { class: "input-value-badge", "{item.start_stars} ★" }
                                                        }
                                                    }
                                                    div { style: "display: flex; gap: 8px;",
                                                        button {
                                                            class: "preset-btn",
                                                            style: "max-width: 42px;",
                                                            r#type: "button",
                                                            disabled: has_input,
                                                            onclick: move |_| {
                                                                let mut items = canvas_items.write();
                                                                let start = items[idx].start_stars.saturating_sub(1);
                                                                items[idx].start_stars = start;
                                                                if items[idx].target_stars < start {
                                                                    items[idx].target_stars = start;
                                                                }
                                                                let (cost, booms, med_cost, med_booms) = simulate_equip(start, items[idx].target_stars, items[idx].level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                                items[idx].avg_cost = cost;
                                                                items[idx].avg_booms = booms;
                                                                items[idx].median_cost = med_cost;
                                                                items[idx].median_booms = med_booms;
                                                                propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                            },
                                                            "-"
                                                        }
                                                        input {
                                                            class: "input-control",
                                                            r#type: "number",
                                                            min: "0",
                                                            max: "29",
                                                            value: "{item.start_stars}",
                                                            disabled: has_input,
                                                            oninput: move |evt| {
                                                                let start = evt.value().parse::<usize>().unwrap_or(0).min(29);
                                                                let mut items = canvas_items.write();
                                                                items[idx].start_stars = start;
                                                                if items[idx].target_stars < start {
                                                                    items[idx].target_stars = start;
                                                                }
                                                                let (cost, booms, med_cost, med_booms) = simulate_equip(start, items[idx].target_stars, items[idx].level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                                items[idx].avg_cost = cost;
                                                                items[idx].avg_booms = booms;
                                                                items[idx].median_cost = med_cost;
                                                                items[idx].median_booms = med_booms;
                                                                propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                            }
                                                        }
                                                        button {
                                                            class: "preset-btn",
                                                            style: "max-width: 42px;",
                                                            r#type: "button",
                                                            disabled: has_input,
                                                            onclick: move |_| {
                                                                let mut items = canvas_items.write();
                                                                let start = (items[idx].start_stars + 1).min(29);
                                                                items[idx].start_stars = start;
                                                                if items[idx].target_stars < start {
                                                                    items[idx].target_stars = start;
                                                                }
                                                                let (cost, booms, med_cost, med_booms) = simulate_equip(start, items[idx].target_stars, items[idx].level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                                items[idx].avg_cost = cost;
                                                                items[idx].avg_booms = booms;
                                                                items[idx].median_cost = med_cost;
                                                                items[idx].median_booms = med_booms;
                                                                propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                            },
                                                            "+"
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        div { class: "form-group",
                                            div { class: "label-container",
                                                label { class: "input-label", "Target Stars" }
                                                span { class: "input-value-badge", "{item.target_stars} ★" }
                                            }
                                            div { style: "display: flex; gap: 8px;",
                                                button {
                                                    class: "preset-btn",
                                                    style: "max-width: 42px;",
                                                    r#type: "button",
                                                    onclick: move |_| {
                                                        let mut items = canvas_items.write();
                                                        let target = items[idx].target_stars.saturating_sub(1).max(1);
                                                        items[idx].target_stars = target;
                                                        if items[idx].start_stars > target {
                                                            items[idx].start_stars = target;
                                                        }
                                                        let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, target, items[idx].level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                        items[idx].avg_cost = cost;
                                                        items[idx].avg_booms = booms;
                                                        items[idx].median_cost = med_cost;
                                                        items[idx].median_booms = med_booms;
                                                        propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                    },
                                                    "-"
                                                }
                                                input {
                                                    class: "input-control",
                                                    r#type: "number",
                                                    min: "1",
                                                    max: "30",
                                                    value: "{item.target_stars}",
                                                    oninput: move |evt| {
                                                        let target = evt.value().parse::<usize>().unwrap_or(1).min(30).max(1);
                                                        let mut items = canvas_items.write();
                                                        items[idx].target_stars = target;
                                                        if items[idx].start_stars > target {
                                                            items[idx].start_stars = target;
                                                        }
                                                        let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, target, items[idx].level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                        items[idx].avg_cost = cost;
                                                        items[idx].avg_booms = booms;
                                                        items[idx].median_cost = med_cost;
                                                        items[idx].median_booms = med_booms;
                                                        propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                    }
                                                }
                                                button {
                                                    class: "preset-btn",
                                                    style: "max-width: 42px;",
                                                    r#type: "button",
                                                    onclick: move |_| {
                                                        let mut items = canvas_items.write();
                                                        let target = (items[idx].target_stars + 1).min(30);
                                                        items[idx].target_stars = target;
                                                        if items[idx].start_stars > target {
                                                            items[idx].start_stars = target;
                                                        }
                                                        let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, target, items[idx].level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                        items[idx].avg_cost = cost;
                                                        items[idx].avg_booms = booms;
                                                        items[idx].median_cost = med_cost;
                                                        items[idx].median_booms = med_booms;
                                                        propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                    },
                                                    "+"
                                                }
                                            }
                                        }

                                        div { class: "form-group",
                                            label { class: "input-label", "Event Modifiers" }
                                            div { class: "modifiers-grid",
                                                style: "grid-template-columns: 1fr 1fr; gap: 8px; margin-top: 5px;",
                                                div {
                                                    class: if item.star_catch { "modifier-card active" } else { "modifier-card" },
                                                    onclick: move |_| {
                                                        let mut items = canvas_items.write();
                                                        let val = !items[idx].star_catch;
                                                        items[idx].star_catch = val;
                                                        let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, items[idx].target_stars, items[idx].level, items[idx].safeguard, val, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                        items[idx].avg_cost = cost;
                                                        items[idx].avg_booms = booms;
                                                        items[idx].median_cost = med_cost;
                                                        items[idx].median_booms = med_booms;
                                                    },
                                                    div { class: "modifier-checkbox" }
                                                    span { class: "modifier-label", "Star Catch" }
                                                }
                                                div {
                                                    class: if item.safeguard { "modifier-card active" } else { "modifier-card" },
                                                    onclick: move |_| {
                                                        let mut items = canvas_items.write();
                                                        let val = !items[idx].safeguard;
                                                        items[idx].safeguard = val;
                                                        let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, items[idx].target_stars, items[idx].level, val, items[idx].star_catch, items[idx].ssf_cost, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                        items[idx].avg_cost = cost;
                                                        items[idx].avg_booms = booms;
                                                        items[idx].median_cost = med_cost;
                                                        items[idx].median_booms = med_booms;
                                                    },
                                                    div { class: "modifier-checkbox" }
                                                    span { class: "modifier-label", "Safeguard" }
                                                }
                                                div {
                                                    class: if item.ssf_cost { "modifier-card active" } else { "modifier-card" },
                                                    onclick: move |_| {
                                                        let mut items = canvas_items.write();
                                                        let val = !items[idx].ssf_cost;
                                                        items[idx].ssf_cost = val;
                                                        let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, items[idx].target_stars, items[idx].level, items[idx].safeguard, items[idx].star_catch, val, items[idx].ssf_boom, items[idx].mode_15_21, canvas_trials());
                                                        items[idx].avg_cost = cost;
                                                        items[idx].avg_booms = booms;
                                                        items[idx].median_cost = med_cost;
                                                        items[idx].median_booms = med_booms;
                                                    },
                                                    div { class: "modifier-checkbox" }
                                                    span { class: "modifier-label", "SSF Cost" }
                                                }
                                                div {
                                                    class: if item.ssf_boom { "modifier-card active" } else { "modifier-card" },
                                                    onclick: move |_| {
                                                        let mut items = canvas_items.write();
                                                        let val = !items[idx].ssf_boom;
                                                        items[idx].ssf_boom = val;
                                                        let (cost, booms, med_cost, med_booms) = simulate_equip(items[idx].start_stars, items[idx].target_stars, items[idx].level, items[idx].safeguard, items[idx].star_catch, items[idx].ssf_cost, val, items[idx].mode_15_21, canvas_trials());
                                                        items[idx].avg_cost = cost;
                                                        items[idx].avg_booms = booms;
                                                        items[idx].median_cost = med_cost;
                                                        items[idx].median_booms = med_booms;
                                                    },
                                                    div { class: "modifier-checkbox" }
                                                    span { class: "modifier-label", "SSF Boom" }
                                                }
                                            }
                                        }

                                        div { class: "form-group",
                                            label { class: "input-label", "Enhancement Mode (Stars 15-21)" }
                                            div { class: "mode-toggle-grid",
                                                for (mode, label) in [
                                                    (EnhancementMode::Level1, "Lvl 1"),
                                                    (EnhancementMode::Level2, "Lvl 2"),
                                                    (EnhancementMode::Level3, "Lvl 3"),
                                                    (EnhancementMode::Level4, "Lvl 4"),
                                                ] {
                                                    button {
                                                        class: if item.mode_15_21.iter().all(|&m| m == mode) { "mode-toggle-btn active" } else { "mode-toggle-btn" },
                                                        r#type: "button",
                                                        onclick: move |_| {
                                                            let mut items = canvas_items.write();
                                                            items[idx].mode_15_21 = [mode; 7];
                                                            let (cost, booms, med_cost, med_booms) = simulate_equip(
                                                                items[idx].start_stars,
                                                                items[idx].target_stars,
                                                                items[idx].level,
                                                                items[idx].safeguard,
                                                                items[idx].star_catch,
                                                                items[idx].ssf_cost,
                                                                items[idx].ssf_boom,
                                                                items[idx].mode_15_21,
                                                                canvas_trials()
                                                            );
                                                            items[idx].avg_cost = cost;
                                                            items[idx].avg_booms = booms;
                                                            items[idx].median_cost = med_cost;
                                                            items[idx].median_booms = med_booms;
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }

                                            // Collapsible Advanced per Star configuration
                                            div { class: "individual-star-config-drawer", style: "margin-top: 10px;",
                                                div {
                                                    class: "drawer-header",
                                                    onclick: move |_| is_canvas_drawer_open.set(!is_canvas_drawer_open()),
                                                    span { class: "drawer-title", "Customize per Star Level" }
                                                    span {
                                                        class: if is_canvas_drawer_open() { "drawer-arrow open" } else { "drawer-arrow" },
                                                        "▶"
                                                    }
                                                }
                                                div {
                                                    class: if is_canvas_drawer_open() { "drawer-content open" } else { "drawer-content" },
                                                    div { class: "individual-star-config",
                                                        for i in 0..7 {
                                                            div { class: "star-config-row",
                                                                span { class: "star-config-row-label",
                                                                    span { class: "star-icon", "★" }
                                                                    "{15 + i} ➔ {15 + i + 1}"
                                                                }
                                                                div { class: "star-config-row-selectors",
                                                                    div { class: "mode-toggle-grid",
                                                                        for (m, lbl) in [
                                                                            (EnhancementMode::Level1, "L1"),
                                                                            (EnhancementMode::Level2, "L2"),
                                                                            (EnhancementMode::Level3, "L3"),
                                                                            (EnhancementMode::Level4, "L4"),
                                                                        ] {
                                                                            button {
                                                                                class: if item.mode_15_21[i] == m { "mode-toggle-btn active" } else { "mode-toggle-btn" },
                                                                                r#type: "button",
                                                                                onclick: move |_| {
                                                                                    let mut items = canvas_items.write();
                                                                                    items[idx].mode_15_21[i] = m;
                                                                                    let (cost, booms, med_cost, med_booms) = simulate_equip(
                                                                                        items[idx].start_stars,
                                                                                        items[idx].target_stars,
                                                                                        items[idx].level,
                                                                                        items[idx].safeguard,
                                                                                        items[idx].star_catch,
                                                                                        items[idx].ssf_cost,
                                                                                        items[idx].ssf_boom,
                                                                                        items[idx].mode_15_21,
                                                                                        canvas_trials()
                                                                                    );
                                                                                    items[idx].avg_cost = cost;
                                                                                    items[idx].avg_booms = booms;
                                                                                    items[idx].median_cost = med_cost;
                                                                                    items[idx].median_booms = med_booms;
                                                                                },
                                                                                "{lbl}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        div { class: "form-group",
                                            label { class: "input-label", "Quick Star Target Presets" }
                                            div { class: "preset-row",
                                                for star in [17, 21, 22] {
                                                    button {
                                                        class: if item.target_stars == star { "preset-btn active" } else { "preset-btn" },
                                                        r#type: "button",
                                                        onclick: move |_| {
                                                            let mut items = canvas_items.write();
                                                            items[idx].target_stars = star;
                                                            
                                                            let has_input = connections().iter().any(|&(_, t)| t == item.id);
                                                            if !has_input {
                                                                items[idx].start_stars = 0;
                                                            }
                                                            items[idx].ssf_cost = true;
                                                            items[idx].ssf_boom = true;
                                                            
                                                            let (cost, booms, med_cost, med_booms) = simulate_equip(
                                                                items[idx].start_stars,
                                                                star,
                                                                items[idx].level,
                                                                items[idx].safeguard,
                                                                items[idx].star_catch,
                                                                items[idx].ssf_cost,
                                                                items[idx].ssf_boom,
                                                                items[idx].mode_15_21,
                                                                canvas_trials());
                                                            items[idx].avg_cost = cost;
                                                            items[idx].avg_booms = booms;
                                                            items[idx].median_cost = med_cost;
                                                            items[idx].median_booms = med_booms;
                                                            propagate_canvas_items(&mut items, &connections(), canvas_trials());
                                                        },
                                                        "{star}★"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
}

