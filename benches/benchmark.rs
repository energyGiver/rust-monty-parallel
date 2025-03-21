// use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// // Import the num-bigint version (stable) and our modules.
// use num_bigint::BigUint as NumBigUint;
// use rust_monty_parallel::biguint::BigUint; // Our minimal BigUint implementation.
// use rust_monty_parallel::monty::monty_modpow;

// fn intense_benchmark(c: &mut Criterion) {
//     // Define benchmark inputs.
//     let base_val = NumBigUint::parse_bytes(b"171088328557451931063571308652120114352073905794908788381430617110957749454190528389629917946086803123143178449314464458240970872631292388793388287926424163820092972837946335119051903325297109239587817508192488443869072875097203143898795574862537362041896175868746593571322499183124039269353089307544958597653", 10).unwrap();
//     let exp_val = NumBigUint::parse_bytes(b"137052351665655252160751247807052127642384117920093987695539140976637104723245744103352105938796934189879604481906316886037821583838047023576485262495283808090874437291879396200090885594109715898924278246098399943822050423711918876609728746644081662419935385028655452259538927727018535785638406162303349147051", 10).unwrap();
//     let mod_val = NumBigUint::parse_bytes(b"169042463442127795748546525285533696137977819953417043406393940034034943536518426262238199174088596477319921011157872487182003513399738457398689653902607468475007530491048452293169900056985348497180881417065297587001350708343522362590521855245695130472536031816948825246464235178383382708324385451232472096631", 10).unwrap();

//     // Create inputs for num-bigint version.
//     let base_nb = base_val.clone();
//     let exp_nb = exp_val.clone();
//     let mod_nb = mod_val.clone();

//     // Create inputs for our Montgomery implementation.
//     let base = BigUint {
//         data: base_val.to_u64_digits(),
//     };
//     let exp = BigUint {
//         data: exp_val.to_u64_digits(),
//     };
//     let modulus = BigUint {
//         data: mod_val.to_u64_digits(),
//     };

//     let mut group = c.benchmark_group("Intense Benchmark");

//     group.bench_function(BenchmarkId::new("num-bigint_modpow", ""), |b| {
//         b.iter(|| {
//             let res = base_nb.modpow(black_box(&exp_nb), black_box(&mod_nb));
//             black_box(res)
//         })
//     });

//     group.bench_function(BenchmarkId::new("monty_modpow_single", ""), |b| {
//         b.iter(|| {
//             let res = monty_modpow(black_box(&base), black_box(&exp), black_box(&modulus));
//             black_box(res)
//         })
//     });

//     group.bench_function(BenchmarkId::new("monty_modpow_parallel", ""), |b| {
//         b.iter(|| {
//             let res = monty_modpow(black_box(&base), black_box(&exp), black_box(&modulus));
//             black_box(res)
//         })
//     });

//     group.finish();
// }

// criterion_group!(benches, intense_benchmark);
// criterion_main!(benches);

fn main() {
    return;
}
