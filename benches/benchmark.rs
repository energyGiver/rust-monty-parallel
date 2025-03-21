use criterion::{black_box, criterion_group, criterion_main, Criterion};
use num_traits::Num;
use rust_monty_parallel::BigUint;

static BIG_B: &str = "\
efac3c0a_0de55551_fee0bfe4_67fa017a_1a898fa1_6ca57cb1\
ca9e3248_cacc09a9_b99d6abc_38418d0f_82ae4238_d9a68832\
aadec7c1_ac5fed48_7a56a71b_67ac59d5_afb28022_20d9592d\
247c4efc_abbd9b75_586088ee_1dc00dc4_232a8e15_6e8191dd\
675b6ae0_c80f5164_752940bc_284b7cee_885c1e10_e495345b\
8fbe9cfd_e5233fe1_19459d0b_d64be53c_27de5a02_a829976b\
33096862_82dad291_bd38b6a9_be396646_ddaf8039_a2573c39\
1b14e8bc_2cb53e48_298c047e_d9879e9c_5a521076_f0e27df3\
990e1659_d3d8205b_6443ebc0_9918ebee_6764f668_9f2b2be3\
b59cbc76_d76d0dfc_d737c3ec_0ccf9c00_ad0554bf_17e776ad\
b4edf9cc_6ce540be_76229093_5c53893b";

static BIG_E: &str = "\
be0e6ea6_08746133_e0fbc1bf_82dba91e_e2b56231_a81888d2\
a833a1fc_f7ff002a_3c486a13_4f420bf3_a5435be9_1a5c8391\
774d6e6c_085d8357_b0c97d4d_2bb33f7c_34c68059_f78d2541\
eacc8832_426f1816_d3be001e_b69f9242_51c7708e_e10efe98\
449c9a4a_b55a0f23_9d797410_515da00d_3ea07970_4478a2ca\
c3d5043c_bd9be1b4_6dce479d_4302d344_84a939e6_0ab5ada7\
12ae34b2_30cc473c_9f8ee69d_2cac5970_29f5bf18_bc8203e4\
f3e895a2_13c94f1e_24c73d77_e517e801_53661fdd_a2ce9e47\
a73dd7f8_2f2adb1e_3f136bf7_8ae5f3b8_08730de1_a4eff678\
e77a06d0_19a522eb_cbefba2a_9caf7736_b157c5c6_2d192591\
17946850_2ddb1822_117b68a0_32f7db88";

static BIG_M: &str = "\
FFFFFFFF_FFFFFFFF_C90FDAA2_2168C234_C4C6628B_80DC1CD1\
29024E08_8A67CC74_020BBEA6_3B139B22_514A0879_8E3404DD\
EF9519B3_CD3A431B_302B0A6D_F25F1437_4FE1356D_6D51C245\
E485B576_625E7EC6_F44C42E9_A637ED6B_0BFF5CB6_F406B7ED\
EE386BFB_5A899FA5_AE9F2411_7C4B1FE6_49286651_ECE45B3D\
C2007CB8_A163BF05_98DA4836_1C55D39A_69163FA8_FD24CF5F\
83655D23_DCA3AD96_1C62F356_208552BB_9ED52907_7096966D\
670C354E_4ABC9804_F1746C08_CA18217C_32905E46_2E36CE3B\
E39E772C_180E8603_9B2783A2_EC07A28F_B5C55DF0_6F4C52C9\
DE2BCBF6_95581718_3995497C_EA956AE5_15D22618_98FA0510\
15728E5A_8AACAA68_FFFFFFFF_FFFFFFFF";

fn bench_modpow_big(c: &mut Criterion) {
    let b = BigUint::from_str_radix(BIG_B, 16).unwrap();
    let e = BigUint::from_str_radix(BIG_E, 16).unwrap();
    let m = BigUint::from_str_radix(BIG_M, 16).unwrap();

    c.bench_function("modpow_big", |bencher| {
        bencher.iter(|| {
            let res = b.modpow(&e, &m);
            black_box(res);
        })
    });
}

criterion_group!(benches, bench_modpow_big);
criterion_main!(benches);
