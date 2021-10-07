extern crate rand;
extern crate crossbeam;
extern crate num_cpus;
extern crate blake2;
extern crate generic_array;
extern crate typenum;
extern crate byteorder;
extern crate bellman;

use self::bellman::pairing::ff::{Field, PrimeField};
use self::byteorder::{ReadBytesExt, BigEndian};
use self::rand::{SeedableRng, Rng, Rand};
use self::rand::chacha::ChaChaRng;
use self::bellman::pairing::bls12_381::{Bls12};
use self::bellman::pairing::*;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use self::generic_array::GenericArray;
use self::typenum::consts::U64;
use self::blake2::{Blake2b, Digest};
use std::fmt;

use crate::parameters::*;
use crate::keypair::*;
use crate::utils::*;

#[derive(Clone)]
pub struct Bls12CeremonyParameters {

}

impl PowersOfTauParameters for Bls12CeremonyParameters {
    const REQUIRED_POWER: usize = 27; // generate to have roughly 2 million constraints

    // This ceremony is based on the BLS381 elliptic curve construction.
    const G1_UNCOMPRESSED_BYTE_SIZE: usize = 96;
    const G2_UNCOMPRESSED_BYTE_SIZE: usize = 192;
    const G1_COMPRESSED_BYTE_SIZE: usize = 48;
    const G2_COMPRESSED_BYTE_SIZE: usize = 96;
}
