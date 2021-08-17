# learning-kate
[![Crates.io](https://img.shields.io/crates/v/a0kzg.svg)](https://crates.io/crates/a0kzg)
[![CI](https://github.com/adria0/a0kzg/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/adria0/a0kzg/actions/workflows/ci.yml)

Kate-Zaverucha-Goldberg polynomial commitments in rust playground

This is just-for-learning, so **do not use in production** of KZG commitments in rust, the
idea is to extend it with other functions like plonk or verkle trees.

It uses the fantastic [bls12_381 crate](https://github.com/zkcrypto/bls12_381) 

## KZG Example

```rust
use a0kzg::{Scalar, Kzg};
// Create a trustd setup that allows maximum 4 points (degree+1)
let kzg = Kzg::trusted_setup(5);

// define the set of points (the "population"), and create a polinomial
// for them, as well its polinomial commitment, see the polinomial commitment
// like the "hash" of the polinomial
let set = vec![
  (Scalar::from(1), Scalar::from(2)),
  (Scalar::from(2), Scalar::from(3)),
  (Scalar::from(3), Scalar::from(4)),
  (Scalar::from(4), Scalar::from(57)),
];
let (p, c) = kzg.poly_commitment_from_set(&set);

// generate a proof that (1,2) and (2,3) are in the set
let proof01 = kzg.prove(&p, &vec![set[0].clone(), set[1].clone()]);

// prove that (1,2) and (2,3) are in the set
assert!(kzg.verify(&c, &vec![set[0].clone(), set[1].clone()], &proof01));
// other proofs will fail since the proof only works for exactly (1,2) AND (2,3)
assert!(!kzg.verify(&c, &vec![set[0].clone()], &proof01));
assert!(!kzg.verify(&c, &vec![set[0].clone(), set[2].clone()], &proof01));

// prove and verify that the whole set exists in the whole set
let proof0123 = kzg.prove(&p, &set);
assert!(kzg.verify(&c, &set, &proof0123));
```

