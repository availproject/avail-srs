extern crate dusk_plonk;
extern crate hex;
extern crate merlin;
extern crate num_cpus;
extern crate rand;

use dusk_plonk::bls12_381::{BlsScalar, G1Affine, G2Affine, G2Prepared};
use dusk_plonk::commitment_scheme::kzg10::{CommitKey, OpeningKey, PublicParameters};
use dusk_plonk::fft::Polynomial;
use merlin::Transcript;
use std::convert::TryInto;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Instant;

const N: usize = 1 << 16;

fn read_points(path: &str) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let mut fd = OpenOptions::new().read(true).open(path)?;
    let meta = fd.metadata()?;
    assert_eq!(
        meta.len(),
        expected_size() as u64,
        "{}",
        format!(
            "expected to be {}b, found to be {}b",
            expected_size(),
            meta.len(),
        ),
    );

    let mut hash: [u8; 64] = [0; 64];
    fd.read_exact(&mut hash[..])?;
    println!("hash: {}", hex::encode(hash));

    let mut g1s: Vec<u8> = vec![0; 48 * N];
    for i in 0..N {
        let start = i * 48;
        fd.read_exact(&mut g1s[start..(start + 48)])?;
    }

    let mut g2s: Vec<u8> = vec![0; 96 * 2];
    for i in 0..2 {
        let start = i * 96;
        fd.read_exact(&mut g2s[start..(start + 96)])?;
    }

    Ok((g1s, g2s))
}

fn convert_g1s(g1s: Vec<u8>) -> std::io::Result<Vec<G1Affine>> {
    let mut cpu_count = num_cpus::get();
    while cpu_count > 0 && N % cpu_count != 0 {
        cpu_count -= 1;
    }
    println!("deserialising G1s, using {} CPUs", cpu_count);

    let span = N / cpu_count;
    let (tx, rx) = sync_channel::<(usize, G1Affine)>(cpu_count * 32);
    let start_tm = Instant::now();

    for i in 0..cpu_count {
        let g1s_ = g1s[(i * span * 48)..((i + 1) * span * 48)].to_vec();
        let tx_ = tx.clone();

        thread::spawn(move || {
            for k in 0..span {
                let start = k * 48;
                let elm = G1Affine::from_compressed(&g1s_[start..(start + 48)].try_into().unwrap())
                    .unwrap();
                let gid = i * span + k;
                tx_.send((gid, elm)).unwrap();
            }
        });
    }

    let mut g1s_: Vec<G1Affine> = vec![G1Affine::identity(); N];
    for _ in 0..N {
        let (gid, elm) = rx.recv().unwrap();
        g1s_[gid] = elm;
    }
    println!("deserialised G1s, in {:?}", start_tm.elapsed());

    Ok(g1s_)
}

fn convert_g2s(g2s: Vec<u8>) -> std::io::Result<Vec<G2Affine>> {
    let mut g2s_: Vec<G2Affine> = vec![G2Affine::identity(); 2];

    for i in 0..2 {
        let start = i * 96;
        g2s_[i] = G2Affine::from_compressed(&g2s[start..(start + 96)].try_into().unwrap()).unwrap();
    }

    Ok(g2s_)
}

fn get_public_params(g1s: Vec<G1Affine>, g2s: Vec<G2Affine>) -> std::io::Result<PublicParameters> {
    let okey = OpeningKey {
        g: g1s[0],
        h: g2s[0],
        beta_h: g2s[1],
        prepared_h: G2Prepared::from(g2s[0]),
        prepared_beta_h: G2Prepared::from(g2s[1]),
    };
    let ckey = CommitKey { powers_of_g: g1s };
    let pp = PublicParameters {
        commit_key: ckey,
        opening_key: okey,
    };

    Ok(pp)
}

fn main() -> std::io::Result<()> {
    let cli_args: Vec<String> = env::args().collect();
    assert_eq!(
        cli_args.len(),
        2,
        "invoke program using `cargo run <abs-path-to-extracted.data>`"
    );

    let (g1s, g2s) = read_points(&cli_args[1])?;
    let g1s = convert_g1s(g1s)?;
    let g2s = convert_g2s(g2s)?;
    let pp = get_public_params(g1s, g2s)?;

    println!(
        "public params of max degree: {}\t[OBTAINED]",
        pp.max_degree()
    );

    test_serialise_deserialise(&pp);
    test_basic_commitment(&pp);
    test_batch_verification(&pp);
    test_aggregate_witness(&pp);
    test_batch_with_aggregation(&pp);

    let pp_bytes = pp.to_bytes();
    let mut serialised_pp = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("serialised_pp.data")?;

    serialised_pp.set_len(pp_bytes.len() as u64)?;
    serialised_pp.write(&pp_bytes)?;
    serialised_pp.flush()?;
    println!("exported serialised public params to `./serialised_pp.data`");

    Ok(())
}

fn test_serialise_deserialise(pp: &PublicParameters) {
    let pp_bytes = pp.to_bytes();
    let d_pp = PublicParameters::from_bytes(&pp_bytes).unwrap();

    assert_eq!(d_pp.commit_key.powers_of_g, pp.commit_key.powers_of_g);
    assert_eq!(d_pp.opening_key.g, pp.opening_key.g);
    assert_eq!(d_pp.opening_key.h, pp.opening_key.h);
    assert_eq!(d_pp.opening_key.beta_h, pp.opening_key.beta_h);

    println!("serialise-deserialise\t✅\t[TEST]");
}

fn test_basic_commitment(pp: &PublicParameters) {
    let degree: usize = 25;
    let (trimmed_ckey, trimmed_okey) = pp.trim(degree).unwrap();

    let point = BlsScalar::from(10);

    let poly = Polynomial::rand(degree, &mut rand::thread_rng());
    let value = poly.evaluate(&point);

    let proof = trimmed_ckey.open_single(&poly, &value, &point).unwrap();
    let ok = trimmed_okey.check(point, proof);
    assert!(ok);

    println!("basic commitment\t✅\t[TEST]");
}

fn test_batch_verification(pp: &PublicParameters) {
    let degree: usize = 25;
    let (proving_key, vk) = pp.trim(degree).unwrap();

    let point_a = BlsScalar::from(10);
    let point_b = BlsScalar::from(11);

    // Compute secret polynomial a
    let poly_a = Polynomial::rand(degree, &mut rand::thread_rng());
    let value_a = poly_a.evaluate(&point_a);

    let proof_a = proving_key
        .open_single(&poly_a, &value_a, &point_a)
        .unwrap();
    assert!(vk.check(point_a, proof_a));

    // Compute secret polynomial b
    let poly_b = Polynomial::rand(degree, &mut rand::thread_rng());
    let value_b = poly_b.evaluate(&point_b);
    let proof_b = proving_key
        .open_single(&poly_b, &value_b, &point_b)
        .unwrap();
    assert!(vk.check(point_b, proof_b));

    assert!(vk
        .batch_check(
            &[point_a, point_b],
            &[proof_a, proof_b],
            &mut Transcript::new(b""),
        )
        .is_ok());

    println!("batch verification\t✅\t[TEST]");
}

fn test_aggregate_witness(pp: &PublicParameters) {
    let max_degree: usize = 27;
    let (proving_key, opening_key) = pp.trim(max_degree).unwrap();

    let point = BlsScalar::from(10);

    // Committer's View
    let aggregated_proof = {
        // Compute secret polynomials and their evaluations
        let poly_a = Polynomial::rand(25, &mut rand::thread_rng());
        let poly_a_eval = poly_a.evaluate(&point);

        let poly_b = Polynomial::rand(26 + 1, &mut rand::thread_rng());
        let poly_b_eval = poly_b.evaluate(&point);

        let poly_c = Polynomial::rand(27, &mut rand::thread_rng());
        let poly_c_eval = poly_c.evaluate(&point);

        proving_key
            .open_multiple(
                &[poly_a, poly_b, poly_c],
                vec![poly_a_eval, poly_b_eval, poly_c_eval],
                &point,
                &mut Transcript::new(b"agg_flatten"),
            )
            .unwrap()
    };

    // Verifier's View
    let ok = {
        let flattened_proof = aggregated_proof.flatten(&mut Transcript::new(b"agg_flatten"));
        opening_key.check(point, flattened_proof)
    };
    assert!(ok);

    println!("aggregate witness\t✅\t[TEST]");
}

fn test_batch_with_aggregation(pp: &PublicParameters) {
    let max_degree = 28;
    let (proving_key, opening_key) = pp.trim(max_degree).unwrap();

    let point_a = BlsScalar::from(10);
    let point_b = BlsScalar::from(11);

    // Committer's View
    let (aggregated_proof, single_proof) = {
        // Compute secret polynomial and their evaluations
        let poly_a = Polynomial::rand(25, &mut rand::thread_rng());
        let poly_a_eval = poly_a.evaluate(&point_a);

        let poly_b = Polynomial::rand(26, &mut rand::thread_rng());
        let poly_b_eval = poly_b.evaluate(&point_a);

        let poly_c = Polynomial::rand(27, &mut rand::thread_rng());
        let poly_c_eval = poly_c.evaluate(&point_a);

        let poly_d = Polynomial::rand(28, &mut rand::thread_rng());
        let poly_d_eval = poly_d.evaluate(&point_b);

        let aggregated_proof = proving_key
            .open_multiple(
                &[poly_a, poly_b, poly_c],
                vec![poly_a_eval, poly_b_eval, poly_c_eval],
                &point_a,
                &mut Transcript::new(b"agg_batch"),
            )
            .unwrap();

        let single_proof = proving_key
            .open_single(&poly_d, &poly_d_eval, &point_b)
            .unwrap();

        (aggregated_proof, single_proof)
    };

    // Verifier's View
    let ok = {
        let mut transcript = Transcript::new(b"agg_batch");
        let flattened_proof = aggregated_proof.flatten(&mut transcript);

        opening_key.batch_check(
            &[point_a, point_b],
            &[flattened_proof, single_proof],
            &mut transcript,
        )
    };
    assert!(ok.is_ok());

    println!("batch with aggregation\t✅\t[TEST]");
}

// expected size of `./extracted.data`
fn expected_size() -> usize {
    N  * 48 + // g1 tau powers
    N * 96 + // g2 tau powers
    N * 48 + // alpha tau powers
    N * 48 // beta tau powers
    + 96 // beta in g2
    + 64 // hash of contribution chain
}
