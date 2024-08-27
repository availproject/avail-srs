extern crate bellman;
extern crate blake2;
extern crate byteorder;
extern crate hex;
extern crate memmap;
extern crate powersoftau;
extern crate rand;

use bellman::pairing::bls12_381::Bls12;
use bellman::pairing::*;
use memmap::*;
use powersoftau::batched_accumulator::BachedAccumulator;
use powersoftau::parameters::PowersOfTauParameters;
use powersoftau::parameters::{CheckForCorrectness, UseCompression};
use powersoftau::small_bls12_381::Bls12CeremonyParameters;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};

const N: usize = 1 << 10;
const MAX_PARAMS: usize = 1 << 27;
type B = BachedAccumulator<Bls12, Bls12CeremonyParameters>;

fn main() -> std::io::Result<()> {
    let cli_args: Vec<String> = env::args().collect();
    assert_eq!(
        cli_args.len(),
        2,
        "invoke program using `cargo run <abs-path-to-challenge-19>`"
    );

    let challenge_reader = OpenOptions::new().read(true).open(cli_args[1].clone())?;
    let metadata = challenge_reader.metadata()?;
    let expected_challenge_length = Bls12CeremonyParameters::ACCUMULATOR_BYTE_SIZE;
    assert_eq!(
        metadata.len(),
        expected_challenge_length as u64,
        "expected to be {}b, found to be {}b",
        expected_challenge_length,
        metadata.len()
    );

    let challenge_readable_map = unsafe { MmapOptions::new().map(&challenge_reader).unwrap() };

    let mut hash = [0; 64];
    let memory_slice = challenge_readable_map.get(0..64).unwrap();
    memory_slice.clone().read_exact(&mut hash)?;
    println!("hash: {}", hex::encode(hash));

    let mut out = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("extracted.data")?;
    out.set_len(expected_size() as u64)?;

    // first write hash of contribution chain
    out.write(&hash[..])?;
    out.flush()?;

    // then write N-many points
    extract(
        &challenge_readable_map,
        &mut out,
        CheckForCorrectness::No,
        UseCompression::No,
    )?;
    println!("extracted {} params into `./extracted.data`\t✅", N);

    Ok(())
}

fn extract(
    input: &Mmap,
    output: &mut std::fs::File,
    check_input_for_correctness: CheckForCorrectness,
    is_compressed: UseCompression,
) -> std::io::Result<()> {
    assert_eq!(N > MAX_PARAMS, false, "not those many params !");

    let mut accumulator = B::empty();
    accumulator
        .read_chunk(0, N, is_compressed, check_input_for_correctness, &input)
        .expect(&format!(
            "must read a chunk from {} to {} from source of decompression",
            0,
            N - 1
        ));

    println!(
        "\ntau_powers_g1: {}\ntau_powers_g2: {}\nalpha_tau_powers_g1: {}\nbeta_tau_powers_g1: {}\n",
        accumulator.tau_powers_g1.len(),
        accumulator.tau_powers_g2.len(),
        accumulator.alpha_tau_powers_g1.len(),
        accumulator.beta_tau_powers_g1.len()
    );

    for i in accumulator.tau_powers_g1.iter() {
        output.write(i.into_compressed().as_ref())?;
    }
    output.flush()?;

    for i in accumulator.tau_powers_g2.iter() {
        output.write(i.into_compressed().as_ref())?;
    }
    output.flush()?;

    for i in accumulator.alpha_tau_powers_g1.iter() {
        output.write(i.into_compressed().as_ref())?;
    }
    output.flush()?;

    for i in accumulator.beta_tau_powers_g1.iter() {
        output.write(i.into_compressed().as_ref())?;
    }
    output.flush()?;

    output.write(accumulator.beta_g2.into_compressed().as_ref())?;
    output.flush()?;

    Ok(())
}

fn expected_size() -> usize {
    N  * 48 + // g1 tau powers
    N * 96 + // g2 tau powers
    N * 48 + // alpha tau powers
    N * 48 // beta tau powers
    + 96 // beta in g2
    + 64 // hash of contribution chain
}
