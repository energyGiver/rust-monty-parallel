use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::env;
use std::fs::File;
use std::io::{Result as IoResult, Write};

// Import the num-bigint version (stable) and our modules.
use num_bigint::BigUint as NumBigUint;
use num_traits::One;
use rust_monty_parallel::biguint::BigUint; // Our minimal BigUint implementation.
use rust_monty_parallel::monty::monty_modpow;

fn intense_benchmark(c: &mut Criterion) {
    // Define benchmark inputs.
    // For demonstration, we use small numbers. For more intense benchmarks,
    // replace these with larger (e.g., 1024-bit or 2048-bit) numbers.
    let base_val: u32 = 3;
    let exp_val: u32 = 13;
    let mod_val: u32 = 17;

    // Create inputs for num-bigint version.
    let base_nb = NumBigUint::from(base_val);
    let exp_nb = NumBigUint::from(exp_val);
    let mod_nb = NumBigUint::from(mod_val);

    // Create inputs for our Montgomery implementation.
    // Our minimal BigUint is represented as a vector of u64 limbs.
    let base = BigUint {
        data: vec![base_val as u64],
    };
    let exp = BigUint {
        data: vec![exp_val as u64],
    };
    let modulus = BigUint {
        data: vec![mod_val as u64],
    };

    // Create a benchmark group.
    let mut group = c.benchmark_group("Intense Benchmark");

    group.bench_function(BenchmarkId::new("num-bigint_modpow", ""), |b| {
        b.iter(|| {
            let res = base_nb.modpow(black_box(&exp_nb), black_box(&mod_nb));
            black_box(res)
        })
    });

    group.bench_function(BenchmarkId::new("monty_modpow_single", ""), |b| {
        b.iter(|| {
            let res = monty_modpow(black_box(&base), black_box(&exp), black_box(&modulus));
            black_box(res)
        })
    });

    group.bench_function(BenchmarkId::new("monty_modpow_parallel", ""), |b| {
        b.iter(|| {
            let res = monty_modpow(black_box(&base), black_box(&exp), black_box(&modulus));
            black_box(res)
        })
    });

    group.finish();

    // Write CSV results to file.
    // This is a simple example; in a production setting you might want to
    // parse Criterion's output for real numbers.
    if let Err(e) = write_csv_results() {
        eprintln!("Failed to write CSV results: {}", e);
    }
}

fn write_csv_results() -> IoResult<()> {
    // Determine current working directory.
    let cwd = env::current_dir()?;
    eprintln!("Current working directory: {}", cwd.display());

    let mut file = File::create("benchmark_results.csv")?;
    writeln!(file, "Algorithm,Average Time (ns)")?;
    // Placeholder values; in practice, you would extract these from Criterion's measurements.
    writeln!(file, "num-bigint_modpow,XYZ")?;
    writeln!(file, "monty_modpow_single,ABC")?;
    writeln!(file, "monty_modpow_parallel,DEF")?;
    file.flush()?;
    eprintln!("CSV file 'benchmark_results.csv' written successfully.");
    Ok(())
}

criterion_group!(benches, intense_benchmark);
criterion_main!(benches);
