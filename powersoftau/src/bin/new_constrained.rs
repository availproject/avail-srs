extern crate bellman;
extern crate memmap;
extern crate powersoftau;

// use powersoftau::bls12_381::{Bls12CeremonyParameters};
use powersoftau::batched_accumulator::BachedAccumulator;
use powersoftau::parameters::UseCompression;
use powersoftau::small_bls12_381::Bls12CeremonyParameters;
use powersoftau::utils::blank_hash;

use bellman::pairing::bls12_381::Bls12;
use memmap::*;
use std::fs::OpenOptions;
use std::io::Write;

use powersoftau::parameters::PowersOfTauParameters;

const COMPRESS_NEW_CHALLENGE: UseCompression = UseCompression::No;

fn main() {
    println!(
        "Will generate an empty accumulator for 2^{} powers of tau",
        Bls12CeremonyParameters::REQUIRED_POWER
    );
    println!(
        "In total will generate up to {} powers",
        Bls12CeremonyParameters::TAU_POWERS_G1_LENGTH
    );

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open("challenge")
        .expect("unable to create `./challenge`");

    let expected_challenge_length = match COMPRESS_NEW_CHALLENGE {
        UseCompression::Yes => {
            Bls12CeremonyParameters::CONTRIBUTION_BYTE_SIZE
                - Bls12CeremonyParameters::PUBLIC_KEY_SIZE
        }
        UseCompression::No => Bls12CeremonyParameters::ACCUMULATOR_BYTE_SIZE,
    };

    file.set_len(expected_challenge_length as u64)
        .expect("unable to allocate large enough file");

    let mut writable_map = unsafe {
        MmapOptions::new()
            .map_mut(&file)
            .expect("unable to create a memory map")
    };

    // Write a blank BLAKE2b hash:
    let hash = blank_hash();
    (&mut writable_map[0..])
        .write(hash.as_slice())
        .expect("unable to write a default hash to mmap");
    writable_map
        .flush()
        .expect("unable to write blank hash to `./challenge`");

    println!("Blank hash for an empty challenge:");
    for line in hash.as_slice().chunks(16) {
        print!("\t");
        for section in line.chunks(4) {
            for b in section {
                print!("{:02x}", b);
            }
            print!(" ");
        }
        println!("");
    }

    BachedAccumulator::<Bls12, Bls12CeremonyParameters>::generate_initial(
        &mut writable_map,
        COMPRESS_NEW_CHALLENGE,
    )
    .expect("generation of initial accumulator is successful");
    writable_map
        .flush()
        .expect("unable to flush memmap to disk");

    // Get the hash of the contribution, so the user can compare later
    let output_readonly = writable_map
        .make_read_only()
        .expect("must make a map readonly");
    let contribution_hash =
        BachedAccumulator::<Bls12, Bls12CeremonyParameters>::calculate_hash(&output_readonly);

    println!("Empty contribution is formed with a hash:");

    for line in contribution_hash.as_slice().chunks(16) {
        print!("\t");
        for section in line.chunks(4) {
            for b in section {
                print!("{:02x}", b);
            }
            print!(" ");
        }
        println!("");
    }

    println!("Wrote a fresh accumulator to `./challenge`");
}
