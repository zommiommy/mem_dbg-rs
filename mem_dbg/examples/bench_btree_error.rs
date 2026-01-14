use cap::Cap;
use deepsize::DeepSizeOf;
use get_size::GetSize;
use mem_dbg::*;
use plotters::prelude::*;
use std::alloc;
use std::collections::{BTreeMap, BTreeSet};

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sizes = [
        0, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000,
    ];

    println!("Running benchmark for usize...");
    run_benchmark(
        &sizes, 
        "usize", 
        |n| {
            let mut m = BTreeMap::new();
            for i in 0..n {
                m.insert(i, i);
            }
            m
        },
        |n| {
            let mut s = BTreeSet::new();
            for i in 0..n {
                s.insert(i);
            }
            s
        }
    )?;

    println!("Running benchmark for String...");
    run_benchmark(
        &sizes, 
        "String", 
        |n| {
            let mut m = BTreeMap::new();
            for i in 0..n {
                let s = format!("{}_{}", i, "x".repeat(i % 100));
                m.insert(s.clone(), s);
            }
            m
        },
        |n| {
            let mut s = BTreeSet::new();
            for i in 0..n {
                s.insert(format!("{}_{}", i, "x".repeat(i % 100)));
            }
            s
        }
    )?;

    println!("Running benchmark for Vec<usize>...");
    run_benchmark(
        &sizes, 
        "Vec", 
        |n| {
            let mut m = BTreeMap::new();
            for i in 0..n {
                let v = vec![i; (i % 100) + 1];
                m.insert(v.clone(), v);
            }
            m
        },
        |n| {
            let mut s = BTreeSet::new();
            for i in 0..n {
                let v = vec![i; (i % 100) + 1];
                s.insert(v);
            }
            s
        }
    )?;

    println!("Running benchmark for (usize, u16) [Padding Check]...");
    run_benchmark(
        &sizes,
        "Tuple_usize_u16",
        |n| {
            let mut m = BTreeMap::new();
            for i in 0..n {
                let v = (i, i as u16);
                m.insert(v, v);
            }
            m
        },
        |n| {
            let mut s = BTreeSet::new();
            for i in 0..n {
                s.insert((i, i as u16));
            }
            s
        }
    )?;

    println!("Running benchmark for Option<usize> [Enum Check]...");
    run_benchmark(
        &sizes,
        "Option_usize",
        |n| {
            let mut m = BTreeMap::new();
            for i in 0..n {
                let v = Some(i);
                m.insert(v, v);
            }
            m
        },
        |n| {
            let mut s = BTreeSet::new();
            for i in 0..n {
                s.insert(Some(i));
            }
            s
        }
    )?;

    println!("Running benchmark for [u8; 32] [Large Copy Check]...");
    run_benchmark(
        &sizes,
        "Array_u8_32",
        |n| {
            let mut m = BTreeMap::new();
            for i in 0..n {
                let v = [i as u8; 32];
                m.insert(v, v);
            }
            m
        },
        |n| {
            let mut s = BTreeSet::new();
            for i in 0..n {
                s.insert([i as u8; 32]);
            }
            s
        }
    )?;

    println!("Running benchmark for Vec<String> [Nested Heap Check]...");
    run_benchmark(
        &sizes,
        "Vec_String",
        |n| {
            let mut m = BTreeMap::new();
            for i in 0..n {
                let v = vec![format!("{}_{}", i, "x".repeat(i % 100))];
                m.insert(v.clone(), v);
            }
            m
        },
        |n| {
            let mut s = BTreeSet::new();
            for i in 0..n {
                s.insert(vec![format!("{}_{}", i, "x".repeat(i % 100))]);
            }
            s
        }
    )?;

    Ok(())
}

fn run_benchmark<M, S>(
    sizes: &[usize],
    type_name: &str,
    mut map_factory: impl FnMut(usize) -> M,
    mut set_factory: impl FnMut(usize) -> S,
) -> Result<(), Box<dyn std::error::Error>> 
where 
    M: MemSize + DeepSizeOf + GetSize,
    S: MemSize + DeepSizeOf + GetSize,
{
    let mut map_mem_dbg = Vec::new();
    let mut map_deepsize = Vec::new();
    let mut map_get_size = Vec::new();

    let mut set_mem_dbg = Vec::new();
    let mut set_deepsize = Vec::new();
    let mut set_get_size = Vec::new();

    for &n in sizes {
        // BTreeMap
        {
            let base = ALLOCATOR.allocated();
            let m = map_factory(n);
            let allocated = ALLOCATOR.allocated().saturating_sub(base);

            let computed = m.mem_size(SizeFlags::default());
            let error = if allocated > 0 {
                ((computed as isize - allocated as isize) as f64 / allocated as f64 * 100.0).abs()
            } else {
                0.0
            };
            map_mem_dbg.push((n, error));

            let computed = m.deep_size_of();
            let error = if allocated > 0 {
                ((computed as isize - allocated as isize) as f64 / allocated as f64 * 100.0).abs()
            } else {
                0.0
            };
            map_deepsize.push((n, error));

            let computed = m.get_size();
            let error = if allocated > 0 {
                ((computed as isize - allocated as isize) as f64 / allocated as f64 * 100.0).abs()
            } else {
                0.0
            };
            map_get_size.push((n, error));
            drop(m);
        }

        // BTreeSet
        {
            let base = ALLOCATOR.allocated();
            let s = set_factory(n);
            let allocated = ALLOCATOR.allocated().saturating_sub(base);

            let computed = s.mem_size(SizeFlags::default());
            let error = if allocated > 0 {
                ((computed as isize - allocated as isize) as f64 / allocated as f64 * 100.0).abs()
            } else {
                0.0
            };
            set_mem_dbg.push((n, error));

            let computed = s.deep_size_of();
            let error = if allocated > 0 {
                ((computed as isize - allocated as isize) as f64 / allocated as f64 * 100.0).abs()
            } else {
                0.0
            };
            set_deepsize.push((n, error));

            let computed = s.get_size();
            let error = if allocated > 0 {
                ((computed as isize - allocated as isize) as f64 / allocated as f64 * 100.0).abs()
            } else {
                0.0
            };
            set_get_size.push((n, error));
            drop(s);
        }
    }

    // Pantone Colors
    let classic_blue = RGBColor(15, 76, 129);  // Pantone 19-4052
    let living_coral = RGBColor(255, 111, 97); // Pantone 16-1546
    let emerald = RGBColor(0, 148, 115);       // Pantone 17-5641
    let ultra_violet = RGBColor(95, 75, 139);  // Pantone 18-3838
    let marsala = RGBColor(150, 79, 76);       // Pantone 18-1438
    let greenery = RGBColor(136, 176, 75);     // Pantone 15-0343

    let calc_mean = |errors: &Vec<(usize, f64)>| -> f64 {
        errors.iter().map(|(_, e)| *e).sum::<f64>() / errors.len() as f64
    };

    let mean_map_mem_dbg = calc_mean(&map_mem_dbg);
    let mean_map_deepsize = calc_mean(&map_deepsize);
    let mean_map_get_size = calc_mean(&map_get_size);
    let mean_set_mem_dbg = calc_mean(&set_mem_dbg);
    let mean_set_deepsize = calc_mean(&set_deepsize);
    let mean_set_get_size = calc_mean(&set_get_size);

    println!("Mean Absolute Percentage Error ({}):", type_name);
    println!("BTreeMap mem_dbg: {:.2}%", mean_map_mem_dbg);
    println!("BTreeMap deepsize: {:.2}%", mean_map_deepsize);
    println!("BTreeMap get-size: {:.2}%", mean_map_get_size);
    println!("BTreeSet mem_dbg: {:.2}%", mean_set_mem_dbg);
    println!("BTreeSet deepsize: {:.2}%", mean_set_deepsize);
    println!("BTreeSet get-size: {:.2}%", mean_set_get_size);
    println!();

    // Plotting
    let filename = format!("btree_error_plot_{}.png", type_name.to_lowercase());
    let root = BitMapBackend::new(&filename, (2048, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((1, 2));

    // Determine Y-axis max
    let max_err = map_mem_dbg.iter()
        .chain(map_deepsize.iter())
        .chain(map_get_size.iter())
        .chain(set_mem_dbg.iter())
        .chain(set_deepsize.iter())
        .chain(set_get_size.iter())
        .map(|(_, e)| *e)
        .fold(0.0f64, f64::max);
    
    let y_max = (max_err as f32 * 1.05).max(1.0);

    // Determine Y-axis min for log scale (avoid 0)
    let min_err_nonzero = map_mem_dbg.iter()
        .chain(map_deepsize.iter())
        .chain(map_get_size.iter())
        .chain(set_mem_dbg.iter())
        .chain(set_deepsize.iter())
        .chain(set_get_size.iter())
        .map(|(_, e)| *e)
        .filter(|&v| v > 1e-6)
        .fold(f64::INFINITY, f64::min);
    
    let y_min_log = if min_err_nonzero.is_infinite() { 0.01 } else { min_err_nonzero as f32 };

    // Define chart drawing closure
    let draw_chart = |area: &DrawingArea<BitMapBackend, plotters::coord::Shift>, use_log_y: bool| -> Result<(), Box<dyn std::error::Error>> {
        let mut chart_builder = ChartBuilder::on(area);
        
        let caption = if use_log_y { 
            format!("{} Abs. Err. vs Size (Log)", type_name) 
        } else { 
            format!("{} Abs. Err. vs Size (Lin)", type_name)
        };

        chart_builder
            .caption(caption, ("sans-serif", 30).into_font())
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(80);

        // DATA SERIES
        let data = [
            (&map_mem_dbg, &classic_blue, "Map mem_dbg", mean_map_mem_dbg),
            (&map_deepsize, &living_coral, "Map deepsize", mean_map_deepsize),
            (&map_get_size, &greenery, "Map get-size", mean_map_get_size),
            (&set_mem_dbg, &ultra_violet, "Set mem_dbg", mean_set_mem_dbg),
            (&set_deepsize, &marsala, "Set deepsize", mean_set_deepsize),
            (&set_get_size, &emerald, "Set get-size", mean_set_get_size),
        ];

        if use_log_y {
            let mut chart = chart_builder.build_cartesian_2d(
                (sizes[0] as f32..sizes[sizes.len() - 1] as f32).log_scale(),
                (y_min_log..y_max).log_scale(),
            )?;

            chart.configure_mesh()
                .x_desc("Number of Elements (n)")
                .y_desc("Absolute Error (%)")
                .x_label_formatter(&|x| format!("{:.0e}", x))
                .y_label_formatter(&|y| format!("{:.0e}", y))
                .draw()?;

            for (series, color, name, mean_err) in data.iter() {
                chart.draw_series(LineSeries::new(
                    series.iter().map(|(x, y)| (*x as f32, (*y).max(y_min_log as f64) as f32)), // Clamp to avoid log(0)
                    *color,
                ))?
                .label(format!("{} (Mean: {:.2}%)", name, mean_err))
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], *color));
            }
            
            chart.configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()?;
        } else {
             let mut chart = chart_builder.build_cartesian_2d(
                (sizes[0] as f32..sizes[sizes.len() - 1] as f32).log_scale(),
             0.0f32..y_max,
            )?;

            chart.configure_mesh()
                .x_desc("Number of Elements (n)")
                .y_desc("Absolute Error (%)")
                .x_label_formatter(&|x| format!("{:.0e}", x))
                .draw()?;

            for (series, color, name, mean_err) in data.iter() {
                chart.draw_series(LineSeries::new(
                    series.iter().map(|(x, y)| (*x as f32, *y as f32)),
                    *color,
                ))?
                .label(format!("{} (Mean: {:.2}%)", name, mean_err))
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], *color));
            }

            chart.configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()?;
        }

        Ok(())
    };

    draw_chart(&areas[0], false)?; // Linear
    draw_chart(&areas[1], true)?;  // Log

    println!("Plot saved to {}", filename);
    Ok(())
}
