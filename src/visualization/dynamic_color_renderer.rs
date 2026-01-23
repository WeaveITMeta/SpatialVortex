/// Dynamic Color Renderer for 2D Flux Matrix
/// 
/// Renders the sacred triangle (3-6-9) with real-time colors based on
/// ELP analysis, while displaying the subject matter as the title.

use crate::dynamic_color_flux::{BrickColor, ELPScore, AspectAnalysis};
use crate::models::FluxMatrix;
use plotters::prelude::*;
use std::path::Path;

/// Configuration for dynamic color rendering
pub struct DynamicColorRenderConfig {
    /// Output image width
    pub width: u32,
    /// Output image height
    pub height: u32,
    /// Background color
    pub background_color: RGBColor,
    /// Show title
    pub show_title: bool,
    /// Show ELP breakdown
    pub show_elp_bars: bool,
}

impl Default for DynamicColorRenderConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            background_color: RGBColor(15, 15, 25), // Dark blue-gray
            show_title: true,
            show_elp_bars: true,
        }
    }
}

/// Render 2D flux matrix with dynamic triangle coloring
pub fn render_dynamic_flux_matrix<P: AsRef<Path>>(
    output_path: P,
    matrix: &FluxMatrix,
    analysis: &AspectAnalysis,
    config: DynamicColorRenderConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create drawing area with proper file path reference
    let root = BitMapBackend::new(&output_path, (config.width, config.height))
        .into_drawing_area();
    
    root.fill(&config.background_color)?;

    // Convert BrickColor to plotters RGBColor
    let triangle_color = brick_to_rgb(&analysis.brick_color);
    let triangle_glow = RGBColor(
        ((analysis.brick_color.rgb.0 * 255.0) as u8).saturating_add(50),
        ((analysis.brick_color.rgb.1 * 255.0) as u8).saturating_add(50),
        ((analysis.brick_color.rgb.2 * 255.0) as u8).saturating_add(50),
    );

    // Calculate center and scale
    let center_x = config.width as f32 / 2.0;
    let center_y = config.height as f32 / 2.0;
    let scale = (config.width.min(config.height) as f32 / 3.0).min(200.0);

    // Sacred triangle vertices (positions 3, 6, 9)
    // Position 3: Bottom right (Ethos)
    let pos_3 = (center_x + scale * 1.5, center_y + scale * 1.5);
    // Position 6: Bottom left (Pathos)  
    let pos_6 = (center_x - scale * 1.5, center_y + scale * 1.5);
    // Position 9: Top center (Logos)
    let pos_9 = (center_x, center_y - scale * 1.5);

    // Draw glowing triangle (multiple layers for glow effect)
    for glow_layer in 0..5 {
        let alpha = (255 - glow_layer * 40) as u8;
        let glow_width = 3 + glow_layer * 2;
        
        root.draw(&PathElement::new(
            vec![
                (pos_3.0 as i32, pos_3.1 as i32),
                (pos_6.0 as i32, pos_6.1 as i32),
                (pos_9.0 as i32, pos_9.1 as i32),
                (pos_3.0 as i32, pos_3.1 as i32),
            ],
            ShapeStyle {
                color: RGBAColor(
                    triangle_color.0,
                    triangle_color.1,
                    triangle_color.2,
                    alpha as f64 / 255.0,
                ),
                filled: false,
                stroke_width: glow_width,
            },
        ))?;
    }

    // Fill triangle with semi-transparent color
    root.draw(&Polygon::new(
        vec![
            (pos_3.0 as i32, pos_3.1 as i32),
            (pos_6.0 as i32, pos_6.1 as i32),
            (pos_9.0 as i32, pos_9.1 as i32),
        ],
        ShapeStyle {
            color: RGBAColor(
                triangle_color.0,
                triangle_color.1,
                triangle_color.2,
                0.3,
            ),
            filled: true,
            stroke_width: 0,
        },
    ))?;

    // Draw vertex circles at sacred positions
    let vertex_radius = 15;
    for (pos, label) in [
        (pos_3, "3\nEthos"),
        (pos_6, "6\nPathos"),
        (pos_9, "9\nLogo s"),
    ] {
        // Outer glow
        root.draw(&Circle::new(
            (pos.0 as i32, pos.1 as i32),
            vertex_radius + 5,
            ShapeStyle {
                color: triangle_glow.mix(0.5),
                filled: true,
                stroke_width: 0,
            },
        ))?;

        // Main circle
        root.draw(&Circle::new(
            (pos.0 as i32, pos.1 as i32),
            vertex_radius,
            ShapeStyle {
                color: triangle_color.into(),
                filled: true,
                stroke_width: 2,
            },
        ))?;

        // Label
        root.draw(&Text::new(
            label,
            (pos.0 as i32, pos.1 as i32 + vertex_radius + 20),
            ("sans-serif", 14).into_font().color(&WHITE),
        ))?;
    }

    // Draw center position (0)
    let center_color = RGBColor(150, 150, 150); // Neutral gray
    root.draw(&Circle::new(
        (center_x as i32, center_y as i32),
        10,
        ShapeStyle {
            color: center_color.into(),
            filled: true,
            stroke_width: 2,
        },
    ))?;

    root.draw(&Text::new(
        "0",
        (center_x as i32, center_y as i32 - 20),
        ("sans-serif", 12).into_font().color(&WHITE),
    ))?;

    // Draw other flux positions (1,2,4,5,7,8) in lighter colors
    let regular_positions = [
        (1, center_x + scale, center_y - scale * 0.8),
        (2, center_x + scale * 1.3, center_y),
        (4, center_x, center_y + scale * 1.8),
        (5, center_x - scale * 0.5, center_y + scale),
        (7, center_x - scale * 1.3, center_y),
        (8, center_x - scale, center_y - scale * 0.8),
    ];

    for (pos_num, x, y) in regular_positions {
        root.draw(&Circle::new(
            (x as i32, y as i32),
            8,
            ShapeStyle {
                color: RGBColor(100, 100, 120).into(),
                filled: true,
                stroke_width: 1,
            },
        ))?;

        root.draw(&Text::new(
            pos_num.to_string(),
            (x as i32 + 15, y as i32),
            ("sans-serif", 11).into_font().color(&RGBColor(180, 180, 200)),
        ))?;
    }

    // Draw title at top
    if config.show_title {
        let title_text = format!("{} - {}", matrix.subject, analysis.brick_color.name);
        root.draw(&Text::new(
            title_text,
            (config.width as i32 / 2, 30),
            ("sans-serif", 32)
                .into_font()
                .color(&WHITE),
        ))?;

        // Draw subtitle with dominant channel
        let subtitle = format!("ELP: E={:.2} L={:.2} P={:.2}", analysis.averaged_elp.ethos, analysis.averaged_elp.logos, analysis.averaged_elp.pathos);
        root.draw(&Text::new(
            subtitle,
            (config.width as i32 / 2, 65),
            ("sans-serif", 18).into_font().color(&RGBColor(200, 200, 220)),
        ))?;
    }

    // Draw ELP breakdown bars
    if config.show_elp_bars {
        draw_elp_bars(&root, &analysis.averaged_elp, config.width, config.height)?;
    }

    // Draw BrickColor swatch
    draw_color_swatch(
        &root,
        &analysis.brick_color,
        config.width - 150,
        config.height - 100,
    )?;

    root.present()?;
    Ok(())
}

/// Draw ELP breakdown bars
fn draw_elp_bars(
    root: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
    elp: &ELPScore,
    _width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let bar_width = 200;
    let bar_height = 20;
    let start_y = height as i32 - 180;
    let start_x = 30;

    // Ethos bar (Blue)
    root.draw(&Rectangle::new(
        [
            (start_x, start_y),
            (start_x + (bar_width as f32 * elp.ethos) as i32, start_y + bar_height),
        ],
        ShapeStyle {
            color: RGBColor(100, 150, 255).into(),
            filled: true,
            stroke_width: 1,
        },
    ))?;
    root.draw(&Text::new(
        format!("Ethos: {:.0}%", elp.ethos * 100.0),
        (start_x + bar_width + 10, start_y + 5),
        ("sans-serif", 14).into_font().color(&WHITE),
    ))?;

    // Logos bar (Green)
    let logos_y = start_y + 35;
    root.draw(&Rectangle::new(
        [
            (start_x, logos_y),
            (start_x + (bar_width as f32 * elp.logos) as i32, logos_y + bar_height),
        ],
        ShapeStyle {
            color: RGBColor(100, 255, 150).into(),
            filled: true,
            stroke_width: 1,
        },
    ))?;
    root.draw(&Text::new(
        format!("Logos: {:.0}%", elp.logos * 100.0),
        (start_x + bar_width + 10, logos_y + 5),
        ("sans-serif", 14).into_font().color(&WHITE),
    ))?;

    // Pathos bar (Red)
    let pathos_y = start_y + 70;
    root.draw(&Rectangle::new(
        [
            (start_x, pathos_y),
            (start_x + (bar_width as f32 * elp.pathos) as i32, pathos_y + bar_height),
        ],
        ShapeStyle {
            color: RGBColor(255, 100, 100).into(),
            filled: true,
            stroke_width: 1,
        },
    ))?;
    root.draw(&Text::new(
        format!("Pathos: {:.0}%", elp.pathos * 100.0),
        (start_x + bar_width + 10, pathos_y + 5),
        ("sans-serif", 14).into_font().color(&WHITE),
    ))?;

    Ok(())
}

/// Draw BrickColor swatch in corner
fn draw_color_swatch(
    root: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
    brick_color: &BrickColor,
    x: u32,
    y: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let swatch_size = 60;
    let color = brick_to_rgb(brick_color);

    // Draw swatch rectangle
    root.draw(&Rectangle::new(
        [(x as i32, y as i32), (x as i32 + swatch_size, y as i32 + swatch_size)],
        ShapeStyle {
            color: color.into(),
            filled: true,
            stroke_width: 2,
        },
    ))?;

    // Draw border
    root.draw(&Rectangle::new(
        [(x as i32, y as i32), (x as i32 + swatch_size, y as i32 + swatch_size)],
        ShapeStyle {
            color: WHITE.into(),
            filled: false,
            stroke_width: 2,
        },
    ))?;

    // Draw label
    root.draw(&Text::new(
        format!("#{}", brick_color.id),
        (x as i32, y as i32 + swatch_size + 15),
        ("sans-serif", 12).into_font().color(&WHITE),
    ))?;

    Ok(())
}

/// Convert BrickColor to plotters RGBColor
fn brick_to_rgb(brick_color: &BrickColor) -> RGBColor {
    RGBColor(
        (brick_color.rgb.0 * 255.0) as u8,
        (brick_color.rgb.1 * 255.0) as u8,
        (brick_color.rgb.2 * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic_color_flux::DynamicColorFluxGenerator;

    #[tokio::test]
    async fn test_dynamic_rendering() {
        let gen = DynamicColorFluxGenerator::new();
        let (matrix, analysis) = gen
            .generate_from_input("Test".to_string(), "I love logical proofs")
            .await
            .unwrap();

        let config = DynamicColorRenderConfig::default();
        let result = render_dynamic_flux_matrix(
            "test_output.png",
            &matrix,
            &analysis,
            config,
        );

        assert!(result.is_ok());
    }
}
