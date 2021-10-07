extern crate powersoftau;
extern crate bellman;
extern crate memmap;
extern crate rand;
extern crate blake2;
extern crate byteorder;

//use powersoftau::bls12_381::{Bls12CeremonyParameters};
use powersoftau::small_bls12_381::{Bls12CeremonyParameters};
use powersoftau::batched_accumulator::{BachedAccumulator};
use powersoftau::keypair::{keypair};
use powersoftau::parameters::{UseCompression, CheckForCorrectness};

use std::fs::OpenOptions;
use bellman::pairing::bls12_381::Bls12;
use memmap::*;

use std::io::{Read, Write};

use powersoftau::parameters::PowersOfTauParameters;

const INPUT_IS_COMPRESSED: UseCompression = UseCompression::No;
const COMPRESS_THE_OUTPUT: UseCompression = UseCompression::Yes;
const CHECK_INPUT_CORRECTNESS: CheckForCorrectness = CheckForCorrectness::No;

fn main() {
    println!("Will contribute to accumulator for 2^{} powers of tau", Bls12CeremonyParameters::REQUIRED_POWER);
    println!("In total will generate up to {} powers", Bls12CeremonyParameters::TAU_POWERS_G1_LENGTH);
    
    // Create an RNG based on a mixture of system randomness and user provided randomness
    let mut rng = {
        use byteorder::{ReadBytesExt, BigEndian};
        use blake2::{Blake2b, Digest};
        use rand::{SeedableRng, Rng, OsRng};
        use rand::chacha::ChaChaRng;

        let h = {
            let mut system_rng = OsRng::new().unwrap();
            let mut h = Blake2b::default();

            // Gather 1024 bytes of entropy from the system
            for _ in 0..1024 {
                let r: u8 = system_rng.gen();
                h.input(&[r]);
            }

            // Ask the user to provide some information for additional entropy
            let mut user_input = String::new();
            println!("Type some random text and press [ENTER] to provide additional entropy...");
            std::io::stdin().read_line(&mut user_input).expect("expected to read some random text from the user");

            // Hash it all up to make a seed
            h.input(&user_input.as_bytes());
            h.result()
        };

        let mut digest = &h[..];

        // Interpret the first 32 bytes of the digest as 8 32-bit words
        let mut seed = [0u32; 8];
        for i in 0..8 {
            seed[i] = digest.read_u32::<BigEndian>().expect("digest is large enough for this to work");
        }

        ChaChaRng::from_seed(&seed)
    };

    // Try to load `./challenge` from disk.
    let reader = OpenOptions::new()
                            .read(true)
                            .open("challenge").expect("unable open `./challenge` in this directory");

    {
        let metadata = reader.metadata().expect("unable to get filesystem metadata for `./challenge`");
        let expected_challenge_length = match INPUT_IS_COMPRESSED {
            UseCompression::Yes => {
                Bls12CeremonyParameters::CONTRIBUTION_BYTE_SIZE
            },
            UseCompression::No => {
                Bls12CeremonyParameters::ACCUMULATOR_BYTE_SIZE
            }
        };

        if metadata.len() != (expected_challenge_length as u64) {
            panic!("The size of `./challenge` should be {}, but it's {}, so something isn't right.", expected_challenge_length, metadata.len());
        }
    }

    let readable_map = unsafe { MmapOptions::new().map(&reader).expect("unable to create a memory map for input") };

    // Create `./response` in this directory
    let writer = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create_new(true)
                            .open("response").expect("unable to create `./response` in this directory");

    let required_output_length = match COMPRESS_THE_OUTPUT {
        UseCompression::Yes => {
            Bls12CeremonyParameters::CONTRIBUTION_BYTE_SIZE
        },
        UseCompression::No => {
            Bls12CeremonyParameters::ACCUMULATOR_BYTE_SIZE + Bls12CeremonyParameters::PUBLIC_KEY_SIZE
        }
    };

    writer.set_len(required_output_length as u64).expect("must make output file large enough");

    let mut writable_map = unsafe { MmapOptions::new().map_mut(&writer).expect("unable to create a memory map for output") };
    
    println!("Calculating previous contribution hash...");

    assert!(UseCompression::No == INPUT_IS_COMPRESSED, "Hashing the compressed file in not yet defined");
    let current_accumulator_hash = BachedAccumulator::<Bls12, Bls12CeremonyParameters>::calculate_hash(&readable_map);

    {
        println!("`challenge` file contains decompressed points and has a hash:");
        for line in current_accumulator_hash.as_slice().chunks(16) {
            print!("\t");
            for section in line.chunks(4) {
                for b in section {
                    print!("{:02x}", b);
                }
                print!(" ");
            }
            println!("");
        }

        (&mut writable_map[0..]).write(current_accumulator_hash.as_slice()).expect("unable to write a challenge hash to mmap");

        writable_map.flush().expect("unable to write hash to `./response`");
    }

    {
        let mut challenge_hash = [0; 64];
        let memory_slice = readable_map.get(0..64).expect("must read point data from file");
        memory_slice.clone().read_exact(&mut challenge_hash).expect("couldn't read hash of challenge file from response file");

        println!("`challenge` file claims (!!! Must not be blindly trusted) that it was based on the original contribution with a hash:");
        for line in challenge_hash.chunks(16) {
            print!("\t");
            for section in line.chunks(4) {
                for b in section {
                    print!("{:02x}", b);
                }
                print!(" ");
            }
            println!("");
        }
    }

    // Construct our keypair using the RNG we created above
    let (pubkey, privkey) = keypair(&mut rng, current_accumulator_hash.as_ref());

    // Perform the transformation
    println!("Computing and writing your contribution, this could take a while...");

    // this computes a transformation and writes it
    BachedAccumulator::<Bls12, Bls12CeremonyParameters>::transform(
        &readable_map, 
        &mut writable_map, 
        INPUT_IS_COMPRESSED, 
        COMPRESS_THE_OUTPUT, 
        CHECK_INPUT_CORRECTNESS, 
        &privkey
    ).expect("must transform with the key");

    println!("Finihsing writing your contribution to `./response`...");

    // Write the public key
    pubkey.write::<Bls12CeremonyParameters>(&mut writable_map, COMPRESS_THE_OUTPUT).expect("unable to write public key");

    writable_map.flush().expect("must flush a memory map");

    // Get the hash of the contribution, so the user can compare later
    let output_readonly = writable_map.make_read_only().expect("must make a map readonly");
    let contribution_hash = BachedAccumulator::<Bls12, Bls12CeremonyParameters>::calculate_hash(&output_readonly);

    print!("Done!\n\n\
              Your contribution has been written to `./response`\n\n\
              The BLAKE2b hash of `./response` is:\n");

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

    println!("Thank you for your participation, much appreciated! :)");
}
