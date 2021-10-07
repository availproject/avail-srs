# avail-srs

## Introduction

For Polygon Avail project, we needed to have one publicly verifiable reference string, which can be used for constructing & verifying KZG polynomial commitment proofs, so we decided to make use of Filecoin's Powers of Tau, which also uses BLS12-381 curve.

---

Polygon Avail: [avail-docs.matic.today](https://avail-docs.matic.today/)

Filecoin's Attestations: [github.com/filecoin-project/phase2-attestations](https://github.com/filecoin-project/phase2-attestations)

---

> Filecoin keeps both phase1 & phase2 files here: [trusted-setup.filecoin.io](https://trusted-setup.filecoin.io/)

> We use **challenge_19** of [phase1](https://trusted-setup.filecoin.io/phase1).

This repository contains programs required for extracting out N( <= 2**27 )-many parameters from `challenge_19`, which are eventually converted into desired serialised format, can be used by Polygon Avail Validators/ Light Clients. During extraction & serialisation, program also asserts public parameters for correctness by running some testcases. 

> We use N = 1 << 16 for constructing out reference string

If you follow steps below, at last you must have `extracted.data` & `serialised_pp.data`.

file name | significance | sha256
--- | --- | ---
challenge_19 | downloaded phase1 **p**owers-**o**f-**t**au file, with N ( = 2 ** 27 ) many parameters |
extracted.data | contains N ( = 65536 ) many parameters extracted from `challenge_19`, in compressed form | eee6430020c96dccccc95ca7b433025e70b58359f400ddf06e4aba37a212afd6
serialised_pp.data | serialised reference strings, ready to be used by Avail validators/ light clients | 3857bb2ec085d4cb8a201c6e40a108870b817f539deadcc3e1e755138f715b10

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

- Finally serialise into desired format, must generate `serialised_pp.data`

```bash
pushd srs
cargo run <absolute-path-to-extracted.data>
sha256sum serialised_pp.data # match with 👆 table
popd
```
