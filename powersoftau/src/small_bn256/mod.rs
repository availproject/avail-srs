extern crate bellman;
extern crate blake2;
extern crate byteorder;
extern crate crossbeam;
extern crate generic_array;
extern crate num_cpus;
extern crate rand;
extern crate typenum;

use self::bellman::pairing::bn256::Bn256;
use self::bellman::pairing::ff::{Field, PrimeField};
use self::bellman::pairing::*;
use self::blake2::{Blake2b, Digest};
use self::byteorder::{BigEndian, ReadBytesExt};
use self::generic_array::GenericArray;
use self::rand::{Rand, Rng, SeedableRng};
use self::typenum::consts::U64;
use std::fmt;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

use crate::keypair::*;
use crate::parameters::*;
use crate::utils::*;

#[derive(Clone)]
pub struct Bn256CeremonyParameters {}

impl PowersOfTauParameters for Bn256CeremonyParameters {
    const REQUIRED_POWER: usize = 12; // generate to have roughly 2 million constraints

    // This ceremony is based on the BN256 elliptic curve construction.
    const G1_UNCOMPRESSED_BYTE_SIZE: usize = 64;
    const G2_UNCOMPRESSED_BYTE_SIZE: usize = 128;
    const G1_COMPRESSED_BYTE_SIZE: usize = 32;
    const G2_COMPRESSED_BYTE_SIZE: usize = 64;
}
