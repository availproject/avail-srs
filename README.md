# avail-srs

## Introduction

For Avail DA, we needed to have one publicly verifiable reference string, which can be used for constructing & verifying KZG polynomial commitment proofs, so we decided to make use of Filecoin's Powers of Tau, which also uses BLS12-381 curve.

---

Avail: [https://docs.availproject.org](https://docs.availproject.org)

Filecoin's Attestations: [github.com/filecoin-project/phase2-attestations](https://github.com/filecoin-project/phase2-attestations)

---

> Filecoin keeps both phase1 & phase2 files here: [trusted-setup.filecoin.io](https://trusted-setup.filecoin.io/)

> We use **challenge_19** of [phase1](https://trusted-setup.filecoin.io/phase1).

This repository contains programs required for extracting out N( <= 2**27 )-many parameters from `challenge_19`, which are eventually converted into desired serialised format, can be used by Avail DA validators / light clients. During extraction & serialisation, program also asserts public parameters for correctness by running some testcases. 

> We use N = 1 << 10 for constructing out reference string

If you follow steps below, you must have following files: `extracted.data`, `g1_g2_1024.txt`, `pp_1024.data` & `pp_raw_1024.data`.

file name | significance | sha256
--- | --- | ---
challenge_19 | downloaded phase1 **p**owers-**o**f-**t**au file, with N ( = 2 ** 27 ) many parameters | 7f311127652a83e3499e7d5e6c9a3dd78f6cb4bd27ea9ce8c1af3818a97adc8f
extracted.data | contains N ( = 1024 ) many parameters extracted from `challenge_19`, in compressed form | 942d0579b83c70dcec7eec2075ff5a13ff7d72a99c21bbcb96a4a1c1865d71fd
g1_g2_1024.txt | contains g1 & g2 points, which can be used to derive pp for [poly_multiproof](https://github.com/availproject/poly-multiproof/releases/tag/v0.0.1) | 942d0579b83c70dcec7eec2075ff5a13ff7d72a99c21bbcb96a4a1c1865d71fd
pp_1024.data | serialised reference strings, ready to be used by Avail validators/ light clients | 6f2a6fc74dd09fb70969a0843ca9fa971c26f224cb2bf11ce18d3c9c2b385a84

## Download

We serve aforementioned 4 static files from https://srs.availproject.org. Download them using

```bash
wget -v https://srs.availproject.org/{challenge_19, extracted.data, g1_g2_1024.txt, pp_1024.data}
```

> After download, make sure you match SHA256 hash with provided one, in above table.

## Requirements

- Make sure you've rust toolchain installed. You may take a look at: [rustup.rs](https://rustup.rs/)
- You also need to have `wget`, UNIX program.

## Usage

- Download phase1 powers-of-tau file

```bash
wget -v https://trusted-setup.filecoin.io/phase1/challenge_19

sha256sum challenge_19 # match with 👆 table
```

- Run parameter extractor, must generate `extracted.data`

```bash
pushd powersoftau
cargo run <absolute-path-to-challenge_19>
sha256sum extracted.data # match with 👆 table
popd
```

- Finally serialise into desired formats

```bash
pushd srs
cargo run <absolute-path-to-extracted.data>
sha256sum g1_g2_1024.txt # match with 👆 table
sha256sum pp_1024.data # match with 👆 table
popd
```

## Extra

Downloaded `challenge_19` file has 1 << 27 parameters, stored in uncompressed form.

> In uncompressed form G1 points: 96 bytes while G2 points: 192 bytes

```python3
num_of_tau_powers = 1 << 27
num_of_g1_tau_powers = (num_of_tau_powers << 1) - 1
```

File layout looks like

```
0           | hash                  <64                   bytes>
...
64          | g1 tau powers         <96 * ((2 ** 28) - 1) bytes>
...
25769803744 | g2 tau powers         <192 * (2 ** 27)      bytes>
...
51539607520 | alpha tau powers      <96 * (2 ** 27)       bytes>
...
64424509408 | beta tau powers       <96 * (2 ** 27)       bytes>
...
77309411296 | beta in g2            <192                  bytes>
...
```

`challenge_19` file size must be **77309411488** bytes.

---

After parameter extraction, generated `extracted.data` holds 1 << 10 parameters, stored in compressed form.

> In uncompressed form G1 points: 48 bytes while G2 points: 96 bytes

File layout looks like

```
0           | hash                  <64             bytes>
...
64          | g1 tau powers         <48 * (2 ** 16) bytes>
...
3145792     | g2 tau powers         <48 * (2 ** 16) bytes>
...
9437248     | alpha tau powers      <48 * (2 ** 16) bytes>
...
12582976    | beta tau powers       <48 * (2 ** 16) bytes>
...
15728704    | beta in g2            <96             bytes>
...
```

`extracted.data` file size must be **15728800** bytes.

---

After serialisation step, reference string holding file `pp_1024.data` has everything an Avail Validator/ Light Client wants to have for proof generation/ verification.

It holds commit key & opening key in compressed form. Use following code snippet for deserialising reference string from byte array.

```rust
let mut data: Vec<u8> = Vec::new();

let fd = OpenOptions::new().read(true).open("pp_1024.data")?;
fd.read_to_end(&mut data)?;

let degree = 1 << 8; // say 256 for this case

let pp = PublicParameters::from_bytes(&data[..]).unwrap();
let (proving_key, verification_key) = pp.trim(degree).unwrap(); // create/ verify proofs !
```

## Acknowledgement

- We make use of Filecoin hosted phase1 powers of tau file ( read `challenge_19` )
- We also use slightly modified https://github.com/arielgabizon/powersoftau, for extracting out parameters. Check [tree](./powersoftau) for more info.
- Test cases run when serialising extracted parameters to compressed byte array, are taken from [here](https://github.com/dusk-network/plonk/blob/36ee6cb1dd8973d7bccddcad688a8d7eec2fbb9f/src/commitment_scheme/kzg10/key.rs#L331-L465)
