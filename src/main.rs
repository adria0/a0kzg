mod poly;
mod kzg;

use kzg::KZG;
use bls12_381::Scalar;

fn main() {
    let kzg = KZG::trusted_setup(6);
    let proof = kzg.prove_one(Scalar::from(5), Scalar::from(9));
    println!("is ok {}",kzg.verify_one(Scalar::from(5), Scalar::from(9), &proof));
}

