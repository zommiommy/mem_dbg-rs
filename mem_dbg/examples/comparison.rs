use cap::Cap;
use comfy_table::*;
use deepsize::DeepSizeOf;
use get_size::GetSize;
use mem_dbg::*;
use std::alloc;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::time::Instant;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sizes to benchmark
    let sizes = [0, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000];
    let mut all_results = Vec::new();

    // --- usize ---
    println!("Running benchmark for usize...");
    all_results.extend(run_benchmark(&sizes, "usize", "BTreeMap", |n| {
        let mut m = BTreeMap::new();
        for i in 0..n {
            m.insert(i, i);
        }
        m
    })?);
    all_results.extend(run_benchmark(&sizes, "usize", "BTreeSet", |n| {
        let mut m = BTreeSet::new();
        for i in 0..n {
            m.insert(i);
        }
        m
    })?);
    all_results.extend(run_benchmark(&sizes, "usize", "HashMap", |n| {
        let mut m = HashMap::new();
        for i in 0..n {
            m.insert(i, i);
        }
        m
    })?);
    all_results.extend(run_benchmark(&sizes, "usize", "HashSet", |n| {
        let mut m = HashSet::new();
        for i in 0..n {
            m.insert(i);
        }
        m
    })?);

    // --- String ---
    println!("Running benchmark for String...");
    all_results.extend(run_benchmark(&sizes, "String", "BTreeMap", |n| {
        let mut m = BTreeMap::new();
        for i in 0..n {
            let s = format!("{}_{}", i, "x".repeat(i % 100));
            m.insert(s.clone(), s);
        }
        m
    })?);
    all_results.extend(run_benchmark(&sizes, "String", "BTreeSet", |n| {
        let mut m = BTreeSet::new();
        for i in 0..n {
            let s = format!("{}_{}", i, "x".repeat(i % 100));
            m.insert(s);
        }
        m
    })?);
    all_results.extend(run_benchmark(&sizes, "String", "HashMap", |n| {
        let mut m = HashMap::new();
        for i in 0..n {
            let s = format!("{}_{}", i, "x".repeat(i % 100));
            m.insert(s.clone(), s);
        }
        m
    })?);
    all_results.extend(run_benchmark(&sizes, "String", "HashSet", |n| {
        let mut m = HashSet::new();
        for i in 0..n {
            let s = format!("{}_{}", i, "x".repeat(i % 100));
            m.insert(s);
        }
        m
    })?);

    // --- Save full table ---
    save_full_table(&all_results)?;

    Ok(())
}

struct BenchResult {
    type_name: String,
    container_name: String,
    crate_name: String,
    size: usize,
    computed_size: usize,
    error: f64,
    time: f64,
}

// Aggregated stats for the table
struct AggregatedResult {
    type_name: String,
    container_name: String,
    crate_name: String,
    mean_error: f64,
    std_dev_error: f64,
    mean_time_per_elem: f64,
}

fn run_benchmark<M>(
    sizes: &[usize],
    type_name: &str,
    container_name: &str,
    mut factory: impl FnMut(usize) -> M,
) -> Result<Vec<BenchResult>, Box<dyn std::error::Error>>
where
    M: MemSize + DeepSizeOf + GetSize,
{
    let mut results = Vec::new();

    for &n in sizes {
        // Run test for size n
        let m = factory(n);
        let stack_size = std::mem::size_of_val(&m);
        let start_alloc = ALLOCATOR.allocated();

        // --- mem_dbg ---
        let start = Instant::now();
        let computed_mem_dbg = m.mem_size(SizeFlags::default());
        let time_mem_dbg = start.elapsed().as_nanos() as f64;

        // --- deepsize ---
        let start = Instant::now();
        let computed_deepsize = m.deep_size_of();
        let time_deepsize = start.elapsed().as_nanos() as f64;

        // --- get-size ---
        let start = Instant::now();
        let computed_get_size = m.get_size();
        let time_get_size = start.elapsed().as_nanos() as f64;

        drop(m);
        let end_alloc = ALLOCATOR.allocated();

        // Correct allocation: Heap allocation of m + Stack size of the struct
        // We use the drop delta to ensure we only measure m's heap memory
        let heap_allocated = start_alloc.saturating_sub(end_alloc);
        let allocated = heap_allocated + stack_size;

        let mut push_result = |crate_name: &str, computed: usize, time: f64| {
            let error = if allocated > 0 {
                ((computed as isize - allocated as isize) as f64 / allocated as f64 * 100.0).abs()
            } else {
                0.0
            };
            results.push(BenchResult {
                type_name: type_name.to_string(),
                container_name: container_name.to_string(),
                crate_name: crate_name.to_string(),
                size: n,
                computed_size: computed,
                error,
                time,
            });
        };

        push_result("mem_dbg", computed_mem_dbg, time_mem_dbg);
        push_result("deepsize", computed_deepsize, time_deepsize);
        push_result("get-size", computed_get_size, time_get_size);
    }

    Ok(results)
}

#[allow(clippy::type_complexity)]
fn save_full_table(results: &[BenchResult]) -> Result<(), Box<dyn std::error::Error>> {
    // --- Full Table ---
    let mut table = Table::new();
    table
        .load_preset(presets::ASCII_MARKDOWN)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Type",
            "Container",
            "Crate",
            "Size",
            "Computed (bytes)",
            "Error %",
            "Time (ns)",
        ]);

    // Sort: Type, Container, Size, Crate
    let mut results = results.iter().collect::<Vec<_>>();
    results.sort_by(|a, b| {
        a.type_name
            .cmp(&b.type_name)
            .then(a.container_name.cmp(&b.container_name))
            .then(a.size.cmp(&b.size))
            .then(a.crate_name.cmp(&b.crate_name))
    });

    for res in results.iter() {
        table.add_row(vec![
            res.type_name.clone(),
            res.container_name.clone(),
            res.crate_name.clone(),
            res.size.to_string(),
            res.computed_size.to_string(),
            format!("{:.4}", res.error),
            format!("{:.0}", res.time),
        ]);
    }

    let mut file = std::fs::File::create("comparison_results.md")?;
    use std::io::Write;
    writeln!(file, "# Comparison Results\n")?;
    writeln!(file, "## Full Data\n")?;
    writeln!(file, "{}", table)?;

    // --- Aggregated Table ---
    let mut agg_table = Table::new();
    agg_table
        .load_preset(presets::ASCII_MARKDOWN)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Type",
            "Container",
            "Crate",
            "Error (%)",
            "Time/Elem (ns)",
        ]);

    // Aggregate
    // Map<(type, container, crate), (Vec<error>, Vec<time>)>
    use std::collections::HashMap;
    let mut grouped_data: HashMap<(String, String, String), (Vec<f64>, Vec<(usize, f64)>)> =
        HashMap::new();

    for res in results.iter() {
        let key = (
            res.type_name.clone(),
            res.container_name.clone(),
            res.crate_name.clone(),
        );
        let entry = grouped_data.entry(key).or_insert((Vec::new(), Vec::new()));
        entry.0.push(res.error);
        entry.1.push((res.size, res.time));
    }

    let mut agg_results = Vec::new();
    for ((t, c, cr), (errs, times)) in grouped_data {
        let n = errs.len() as f64;
        let mean_err = errs.iter().sum::<f64>() / n;
        let var_err = errs.iter().map(|e| (e - mean_err).powi(2)).sum::<f64>() / (n - 1.0);
        let std_err = var_err.sqrt();

        let times_per_elem: Vec<f64> = times
            .iter()
            .filter_map(|(size, time)| {
                if *size > 0 {
                    Some(time / (*size as f64))
                } else {
                    None
                }
            })
            .collect();
        let n_t = times_per_elem.len() as f64;
        let mean_time_per_elem = if n_t > 0.0 {
            times_per_elem.iter().sum::<f64>() / n_t
        } else {
            0.0
        };

        agg_results.push(AggregatedResult {
            type_name: t,
            container_name: c,
            crate_name: cr,
            mean_error: mean_err,
            std_dev_error: std_err,
            mean_time_per_elem,
        });
    }

    // Sort for consistent display
    agg_results.sort_by(|a, b| {
        a.type_name
            .cmp(&b.type_name)
            .then(a.container_name.cmp(&b.container_name))
            .then(a.crate_name.cmp(&b.crate_name))
    });

    // Best highlighting
    let mut min_error_per_group: HashMap<(String, String), f64> = HashMap::new();
    let mut max_error_per_group: HashMap<(String, String), f64> = HashMap::new();
    let mut min_time_per_group: HashMap<(String, String), f64> = HashMap::new();
    let mut max_time_per_group: HashMap<(String, String), f64> = HashMap::new();

    for res in &agg_results {
        let key = (res.type_name.clone(), res.container_name.clone());

        let current_min_err = min_error_per_group
            .entry(key.clone())
            .or_insert(f64::INFINITY);
        if res.mean_error < *current_min_err {
            *current_min_err = res.mean_error;
        }

        let current_max_err = max_error_per_group
            .entry(key.clone())
            .or_insert(f64::NEG_INFINITY);
        if res.mean_error > *current_max_err {
            *current_max_err = res.mean_error;
        }

        let current_min_time = min_time_per_group
            .entry(key.clone())
            .or_insert(f64::INFINITY);
        if res.mean_time_per_elem < *current_min_time {
            *current_min_time = res.mean_time_per_elem;
        }

        let current_max_time = max_time_per_group.entry(key).or_insert(f64::NEG_INFINITY);
        if res.mean_time_per_elem > *current_max_time {
            *current_max_time = res.mean_time_per_elem;
        }
    }

    let mut last_type = String::new();
    let mut last_container = String::new();

    for res in agg_results {
        let key = (res.type_name.clone(), res.container_name.clone());
        let min_err = *min_error_per_group.get(&key).unwrap();
        let max_err = *max_error_per_group.get(&key).unwrap();
        let min_time = *min_time_per_group.get(&key).unwrap();
        let max_time = *max_time_per_group.get(&key).unwrap();

        let err_diff = (max_err - min_err).abs();
        let is_best_err = err_diff > 1e-9 && (res.mean_error - min_err).abs() < 1e-9;

        // Only highlight if there is a difference between min and max
        let time_diff = (max_time - min_time).abs();
        let is_best_time = time_diff > 1e-9 && (res.mean_time_per_elem - min_time).abs() < 1e-9;

        let show_type = if res.type_name == last_type {
            String::new()
        } else {
            res.type_name.clone()
        };
        let show_container = if res.container_name == last_container && show_type.is_empty() {
            String::new()
        } else {
            res.container_name.clone()
        };

        last_type = res.type_name.clone();
        last_container = res.container_name.clone();

        let format_bold = |s: String, b: bool| {
            if b && !s.is_empty() {
                format!("**{}**", s)
            } else {
                s
            }
        };

        agg_table.add_row(vec![
            format_bold(show_type, false),
            format_bold(show_container, false),
            format_bold(res.crate_name, false),
            format_bold(
                format!("{:.2} ± {:.2}", res.mean_error, res.std_dev_error),
                is_best_err,
            ),
            format_bold(format!("{:.2}", res.mean_time_per_elem), is_best_time),
        ]);
    }

    writeln!(file, "\n## Aggregated Results\n")?;
    writeln!(file, "{}", agg_table)?;
    println!("Full table and aggregated results saved to comparison_results.md");
    Ok(())
}
