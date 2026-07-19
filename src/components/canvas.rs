use dioxus::prelude::*;

/// Component that renders star icons representing item enhancement level progression.
#[component]
pub fn StarsVisualizer(start: usize, target: usize) -> Element {
    let max_stars = if target > 25 { target } else { 25 };
    rsx! {
        div { class: "stars-visualizer",
            for i in 0..max_stars {
                {
                    let is_filled = i < start;
                    let is_target = i < target;
                    let class_name = if is_filled {
                        "visual-star filled"
                    } else if is_target {
                        "visual-star target"
                    } else {
                        "visual-star empty"
                    };
                    rsx! {
                        span { class: "{class_name}", "★" }
                    }
                }
            }
        }
    }
}
