/// 2D Flux Matrix Visualization Example
/// 
/// Demonstrates:
/// 1. Mapping data points to 2D flux positions
/// 2. Sacred geometry (3-6-9 triangle)
/// 3. Flow lines and intersections
/// 4. ELP tensor visualization
/// 5. Native Rust plotting with plotters (matplotlib equivalent)

use spatial_vortex::{
    lock_free_flux::LockFreeFluxMatrix,
    visualization::{FluxLayout, FluxVisualization, PositionAnalysis},
    models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex},
};
use std::collections::HashMap;
use plotters::prelude::*;

fn create_test_node(name: &str, position: u8, ethos: f64, logos: f64, pathos: f64) -> FluxNode {
    let mut parameters = HashMap::new();
    parameters.insert("ethos".to_string(), ethos);
    parameters.insert("logos".to_string(), logos);
    parameters.insert("pathos".to_string(), pathos);
    
    FluxNode {
        position,
        base_value: position,
        semantic_index: SemanticIndex {
            positive_associations: vec![],
            negative_associations: vec![],
            neutral_base: name.to_string(),
            predicates: vec![],
            relations: vec![],
        },
        attributes: NodeAttributes {
            properties: HashMap::new(),
            parameters,
            state: NodeState {
                active: true,
                last_accessed: chrono::Utc::now(),
                usage_count: 0,
                context_stack: vec![],
            },
            dynamics: NodeDynamics {
                evolution_rate: 1.0,
                stability_index: 1.0,
                interaction_patterns: vec![],
                learning_adjustments: vec![],
            },
        },
        connections: vec![],
    }
}

fn main() -> anyhow::Result<()> {
    println!("\n🎨 2D FLUX MATRIX VISUALIZATION - MULTIPLE SUBJECTS");
    println!("===================================================\n");
    
    // Create output directory
    std::fs::create_dir_all("flux_matrix_images")?;
    
    // Define multiple test subjects
    let test_subjects = vec![
        (
            "Sacred_Virtues",
            vec![
                ("Love", 3, 0.7, 0.5, 0.95),       // Sacred - high pathos
                ("Truth", 6, 0.85, 0.95, 0.5),     // Sacred - high logos
                ("Creation", 9, 0.9, 0.6, 0.5),    // Sacred - high ethos
                ("Joy", 1, 0.6, 0.4, 0.9),
                ("Wisdom", 8, 0.85, 0.95, 0.5),
                ("Courage", 5, 0.95, 0.6, 0.4),
                ("Peace", 2, 0.6, 0.5, 0.8),
                ("Justice", 7, 0.9, 0.7, 0.5),
                ("Beauty", 4, 0.6, 0.6, 0.8),
                ("Freedom", 0, 0.7, 0.8, 0.6),
            ]
        ),
        (
            "Emotional_Spectrum",
            vec![
                ("Ecstasy", 3, 0.5, 0.3, 0.98),    // Sacred - extreme pathos
                ("Despair", 6, 0.4, 0.3, 0.95),    // Sacred - high pathos (negative)
                ("Euphoria", 9, 0.6, 0.4, 0.92),   // Sacred - high pathos
                ("Hope", 1, 0.7, 0.5, 0.85),
                ("Fear", 8, 0.5, 0.6, 0.80),
                ("Anger", 5, 0.9, 0.4, 0.75),
                ("Serenity", 2, 0.5, 0.5, 0.90),
                ("Grief", 7, 0.4, 0.5, 0.88),
                ("Surprise", 4, 0.6, 0.6, 0.70),
                ("Curiosity", 0, 0.6, 0.7, 0.65),
            ]
        ),
        (
            "Logical_Concepts",
            vec![
                ("Axiom", 3, 0.5, 0.95, 0.3),      // Sacred - high logos
                ("Theorem", 6, 0.6, 0.98, 0.4),    // Sacred - extreme logos
                ("Proof", 9, 0.7, 0.92, 0.35),     // Sacred - high logos
                ("Hypothesis", 1, 0.5, 0.85, 0.45),
                ("Deduction", 8, 0.6, 0.90, 0.4),
                ("Inference", 5, 0.65, 0.88, 0.42),
                ("Analysis", 2, 0.55, 0.82, 0.48),
                ("Synthesis", 7, 0.70, 0.85, 0.43),
                ("Validation", 4, 0.60, 0.80, 0.50),
                ("Reason", 0, 0.65, 0.87, 0.45),
            ]
        ),
        (
            "Ethical_Principles",
            vec![
                ("Integrity", 3, 0.95, 0.6, 0.5),  // Sacred - high ethos
                ("Honor", 6, 0.98, 0.7, 0.4),      // Sacred - extreme ethos
                ("Virtue", 9, 0.92, 0.65, 0.45),   // Sacred - high ethos
                ("Duty", 1, 0.88, 0.70, 0.50),
                ("Loyalty", 8, 0.90, 0.68, 0.48),
                ("Responsibility", 5, 0.85, 0.75, 0.52),
                ("Dignity", 2, 0.87, 0.65, 0.55),
                ("Character", 7, 0.91, 0.72, 0.47),
                ("Nobility", 4, 0.83, 0.68, 0.58),
                ("Principle", 0, 0.86, 0.74, 0.51),
            ]
        ),
        (
            "Balanced_Concepts",
            vec![
                ("Harmony", 3, 0.75, 0.75, 0.75),  // Sacred - perfectly balanced
                ("Unity", 6, 0.80, 0.80, 0.80),    // Sacred - balanced high
                ("Wholeness", 9, 0.78, 0.78, 0.78),// Sacred - balanced
                ("Balance", 1, 0.70, 0.72, 0.71),
                ("Equilibrium", 8, 0.73, 0.74, 0.73),
                ("Symmetry", 5, 0.76, 0.75, 0.74),
                ("Moderation", 2, 0.68, 0.70, 0.72),
                ("Integration", 7, 0.77, 0.78, 0.76),
                ("Coherence", 4, 0.72, 0.71, 0.73),
                ("Centeredness", 0, 0.74, 0.73, 0.75),
            ]
        ),
    ];
    
    // Generate visualization for each subject
    for (subject_name, test_data) in &test_subjects {
        println!("\n📋 Subject: {}", subject_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Create flux matrix
        let matrix = LockFreeFluxMatrix::new(format!("{}_Matrix", subject_name));
        
        println!("📦 Creating {} data points...", test_data.len());
        for (name, pos, e, l, p) in test_data {
            let node = create_test_node(name, *pos, *e, *l, *p);
            matrix.insert(node);
            
            let sacred = if [3, 6, 9].contains(pos) { "⭐" } else { " " };
            println!("   {} Position {}: {} (E:{:.2} L:{:.2} P:{:.2})", 
                sacred, pos, name, e, l, p);
        }
        
        // Create visualization with sacred geometry layout
        println!("\n🎨 Generating visualization...");
        let layout = FluxLayout::sacred_geometry_layout();
        let title = format!("Flux Matrix: {}", subject_name.replace('_', " "));
        let viz = FluxVisualization::from_flux_matrix(&matrix, layout, title);
        
        // Render with native Rust plotters
        let filename = format!("flux_matrix_images/flux_matrix_{}.png", subject_name.to_lowercase());
        render_flux_plot_to_file(&viz, &filename)?;
        println!("   ✅ {}", filename);
    }
    
    println!("\n✅ ALL VISUALIZATIONS COMPLETE!");
    println!("   {} images generated in flux_matrix_images/\n", test_subjects.len());
    
    Ok(())
}

/// Render flux matrix to a specific file with enhanced visualization
fn render_flux_plot_to_file(viz: &FluxVisualization, filename: &str) -> anyhow::Result<()> {
    // Maintain 1:1 aspect ratio with comfortable framing
    // X: [-13, 13] = 26 units, Y: [-8, 13] = 21 units
    // Height = 1400 * (21/26) = 1131 pixels for 1:1 aspect ratio
    let root = BitMapBackend::new(filename, (1400, 1131)).into_drawing_area();
    
    // Light gray background for better contrast
    root.fill(&RGBColor(250, 250, 250))?;
    
    // Create chart with 1:1 aspect ratio and comfortable margins
    let mut chart = ChartBuilder::on(&root)
        .caption(&viz.title, ("sans-serif", 48, FontStyle::Bold).into_font().color(&BLACK))
        .margin(80)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(-13.0f64..13.0f64, -8.0f64..13.0f64)?; // Comfortable framing
    
    chart.configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(10)
        .y_labels(10)
        .label_style(("sans-serif", 20))  // Increase axis label font size
        .draw()?;
    
    // Draw outer circle with gradient effect (multiple circles) centered at shifted center
    for i in 0..3 {
        let alpha = 0.15 - (i as f64 * 0.04);
        chart.draw_series(std::iter::once(Circle::new(
            (viz.layout.center.x, viz.layout.center.y),
            ((viz.layout.radius + (i as f64 * 0.1625)) * 55.38) as i32,
            ShapeStyle::from(&BLUE.mix(alpha)).stroke_width(2),
        )))?;
    }
    
    // Draw sacred triangle (3-6-9) - BOLD BLACK with thicker lines
    let triangle_points: Vec<(f64, f64)> = viz.sacred_elements.triangle_vertices
        .iter()
        .map(|p| (p.x, p.y))
        .chain(std::iter::once((
            viz.sacred_elements.triangle_vertices[0].x,
            viz.sacred_elements.triangle_vertices[0].y,
        )))
        .collect();
    
    chart.draw_series(LineSeries::new(
        triangle_points,
        ShapeStyle::from(&BLACK).stroke_width(5),
    ))?.label("Sacred Triangle (3-6-9)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));
    
    // Draw cyan intersection markers with pulsing dynamic effect
    let cyan = RGBColor(0, 191, 255);
    
    // Sacred triangle vertices (3, 6, 9) with pulse layers
    for vertex in &viz.sacred_elements.triangle_vertices {
        // Outer pulse - very soft
        chart.draw_series(std::iter::once(Circle::new(
            (vertex.x, vertex.y),
            20,
            ShapeStyle::from(&cyan.mix(0.15)).filled(),
        )))?;
        // Middle pulse - medium
        chart.draw_series(std::iter::once(Circle::new(
            (vertex.x, vertex.y),
            16,
            ShapeStyle::from(&cyan.mix(0.35)).filled(),
        )))?;
        // Core intersection marker
        chart.draw_series(std::iter::once(Circle::new(
            (vertex.x, vertex.y),
            12,
            ShapeStyle::from(&cyan).filled(),
        )))?;
    }
    
    // Center position (0) intersection with strongest pulse
    chart.draw_series(std::iter::once(Circle::new(
        (0.0, 0.0),
        24,
        ShapeStyle::from(&cyan.mix(0.12)).filled(),
    )))?;
    chart.draw_series(std::iter::once(Circle::new(
        (0.0, 0.0),
        18,
        ShapeStyle::from(&cyan.mix(0.3)).filled(),
    )))?;
    chart.draw_series(std::iter::once(Circle::new(
        (0.0, 0.0),
        12,
        ShapeStyle::from(&cyan).filled(),
    )))?;
    
    // Draw flow lines (vortex math star pattern)
    for flow_line in &viz.flow_lines {
        if flow_line.is_sacred {
            // Sacred connections are bold black
            chart.draw_series(LineSeries::new(
                vec![
                    (flow_line.from_coords.x, flow_line.from_coords.y),
                    (flow_line.to_coords.x, flow_line.to_coords.y),
                ],
                ShapeStyle::from(&BLACK).stroke_width(4),
            ))?;
        } else {
            // Other connections are thin gray
            chart.draw_series(LineSeries::new(
                vec![
                    (flow_line.from_coords.x, flow_line.from_coords.y),
                    (flow_line.to_coords.x, flow_line.to_coords.y),
                ],
                ShapeStyle::from(&BLACK.mix(0.4)).stroke_width(1),
            ))?;
        }
    }
    
    // Draw position markers (0-9) on circle
    for (pos, coords) in &viz.layout.positions {
        // All positions use same style (white circles with black border)
        let is_sacred = [3, 6, 9].contains(pos);
        
        // Position circle with shadow effect for sacred positions
        if is_sacred {
            // Shadow for sacred positions only
            chart.draw_series(std::iter::once(Circle::new(
                (coords.x + 0.01, coords.y - 0.01),
                17,
                ShapeStyle::from(&BLACK.mix(0.3)).filled(),
            )))?;
        }
        
        // Position circle (all same style now)
        chart.draw_series(std::iter::once(Circle::new(
            (coords.x, coords.y),
            18,
            ShapeStyle::from(&WHITE).filled().stroke_width(2),
        )))?;
        
        // Position label: ABOVE the circle for ALL positions (0-9) with small gap
        chart.draw_series(std::iter::once(Text::new(
            format!("{}", pos),
            (coords.x - 0.1625, coords.y + 1.1375),
            ("sans-serif", 32, plotters::style::FontStyle::Bold).into_font().color(&BLACK),
        )))?;
    }
    
    // Draw data points with ELP coloring and dynamic effects
    for point in &viz.data_points {
        // Color by dominant channel
        let color = match point.dominant_channel() {
            _ => RGBColor(100, 100, 100)
        };
            "Ethos" => &RED,
            "Logos" => &BLUE,
            "Pathos" => &GREEN,
            _ => &BLACK,
        };
        
        let pos = point.position;
        let is_sacred = [3, 6, 9].contains(&pos);
        let base_size = (point.tensor_magnitude() * 16.0) as i32;
        
        // Dynamic sizing: sacred positions pulse larger
        let size = if is_sacred {
            (base_size as f64 * 1.2) as i32
        } else {
            base_size
        };
        
        // Multi-layer glow halo for dynamic energy visualization
        // Outer halo - softer, larger
        let outer_halo = size + 8;
        chart.draw_series(std::iter::once(Circle::new(
            (point.coords.x, point.coords.y),
            outer_halo,
            ShapeStyle::from(color.mix(0.1)).filled(),
        )))?;
        
        // Inner halo - brighter, medium
        let inner_halo = size + 4;
        chart.draw_series(std::iter::once(Circle::new(
            (point.coords.x, point.coords.y),
            inner_halo,
            ShapeStyle::from(color.mix(0.25)).filled(),
        )))?;
        
        // Data point marker (all circles now)
        chart.draw_series(std::iter::once(Circle::new(
            (point.coords.x, point.coords.y),
            size,
            ShapeStyle::from(color).filled().stroke_width(2),
        )))?;
        
        // Extra golden halo for sacred positions
        if is_sacred {
            chart.draw_series(std::iter::once(Circle::new(
                (point.coords.x, point.coords.y),
                (size as f64 * 1.5) as i32,
                ShapeStyle::from(&RGBColor(255, 215, 0).mix(0.3)).filled(),
            )))?;
        }
        
        // Label text with proper centering
        let char_width = 0.29;  // Approximate width per character at 24pt in coordinate units
        let text_width = point.id.len() as f64 * char_width;
        let text_height = 0.65;  // Approximate height at 24pt
        
        // Center point for both box and text (anchor 0.5, 0.5)
        let center_x = point.coords.x;
        let center_y = point.coords.y;
        
        // Background box with padding
        let box_padding = 0.3;
        let box_width = text_width + (box_padding * 2.0);
        let box_height = text_height + (box_padding * 2.0);
        
        chart.draw_series(std::iter::once(plotters::prelude::Rectangle::new(
            [(center_x - box_width/2.0, center_y - box_height/2.0), 
             (center_x + box_width/2.0, center_y + box_height/2.0)],
            ShapeStyle::from(&WHITE.mix(0.5)).filled(),
        )))?;
        
        // Text centered using anchor point 0.5, 0.5 logic
        // Offset by half text dimensions to achieve center anchoring
        chart.draw_series(std::iter::once(Text::new(
            point.id.clone(),
            (center_x - text_width/2.0, center_y - text_height/3.0),  // Adjusted for baseline
            ("sans-serif", 24, FontStyle::Bold).into_font().color(&BLACK),
        )))?;
    }
    
    // Draw legend box with ELP color coding (positioned in upper right)
    let legend_x = 11.5;  // Moved even further right for optimal positioning
    let legend_y = 10.5;  // Positioned in upper area within [-8, 13] bounds
    
    // Legend title
    chart.draw_series(std::iter::once(Text::new(
        "ELP Channels",
        (legend_x, legend_y),
        ("sans-serif", 20, FontStyle::Bold).into_font().color(&BLACK),
    )))?;
    
    // Ethos (Red)
    chart.draw_series(std::iter::once(Circle::new(
        (legend_x - 1.21875, legend_y - 1.21875),
        8,
        ShapeStyle::from(&RED).filled(),
    )))?;
    chart.draw_series(std::iter::once(Text::new(
        "Ethos (Character)",
        (legend_x - 0.40625, legend_y - 1.21875),
        ("sans-serif", 14).into_font().color(&BLACK),
    )))?;
    
    // Logos (Blue)
    chart.draw_series(std::iter::once(Circle::new(
        (legend_x - 1.21875, legend_y - 2.4375),
        8,
        ShapeStyle::from(&BLUE).filled(),
    )))?;
    chart.draw_series(std::iter::once(Text::new(
        "Logos (Logic)",
        (legend_x - 0.40625, legend_y - 2.4375),
        ("sans-serif", 14).into_font().color(&BLACK),
    )))?;
    
    // Pathos (Green)
    chart.draw_series(std::iter::once(Circle::new(
        (legend_x - 1.21875, legend_y - 3.65625),
        8,
        ShapeStyle::from(&GREEN).filled(),
    )))?;
    chart.draw_series(std::iter::once(Text::new(
        "Pathos (Emotion)",
        (legend_x - 0.40625, legend_y - 3.65625),
        ("sans-serif", 14).into_font().color(&BLACK),
    )))?;
    
    // Add sacred positions annotation
    chart.draw_series(std::iter::once(Text::new(
        "⭐ Sacred: 3-6-9",
        (legend_x - 1.21875, legend_y - 5.28125),
        ("sans-serif", 16, FontStyle::Bold).into_font().color(&BLACK),
    )))?;
    
    // Statistics removed per user request
    
    // Draw original legend for sacred triangle
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.9))
        .border_style(&BLACK)
        .label_font(("sans-serif", 16))
        .draw()?;
    
    root.present()?;
    Ok(())
}
