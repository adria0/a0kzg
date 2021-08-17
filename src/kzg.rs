use super::poly::Poly;
pub use bls12_381::Scalar;
use bls12_381::*;
use rand::Rng;

pub struct Kzg {
    pow_tau_g1: Vec<G1Projective>,
    pow_tau_g2: Vec<G2Projective>,
}

pub type Proof = G1Projective;
pub type Commitment = G1Projective;

impl Kzg {
    fn eval_at_tau_g1(&self, poly: &Poly) -> G1Projective {
        poly.0
            .iter()
            .enumerate()
            .fold(G1Projective::identity(), |acc, (n, k)| {
                acc + self.pow_tau_g1[n] * k
            })
    }

    fn eval_at_tau_g2(&self, poly: &Poly) -> G2Projective {
        poly.0
            .iter()
            .enumerate()
            .fold(G2Projective::identity(), |acc, (n, k)| {
                acc + self.pow_tau_g2[n] * k
            })
    }

    fn z_poly_of(points: &[(Scalar, Scalar)]) -> Poly {
        points.iter().fold(Poly::one(), |acc, (z, _y)| {
            &acc * &Poly::new(vec![-z, Scalar::one()])
        })
    }

    pub fn trusted_setup(n: usize) -> Self {
        let mut rng = rand::thread_rng();
        let rnd: [u64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
        let tau = Scalar::from_raw(rnd);

        let pow_tau_g1: Vec<G1Projective> = (0..n)
            .into_iter()
            .scan(Scalar::one(), |acc, _| {
                let v = *acc;
                *acc *= tau;
                Some(v)
            })
            .map(|tau_pow| G1Affine::generator() * tau_pow)
            .collect();

        let pow_tau_g2: Vec<G2Projective> = (0..n)
            .into_iter()
            .scan(Scalar::one(), |acc, _| {
                let v = *acc;
                *acc *= tau;
                Some(v)
            })
            .map(|tau_pow| G2Affine::generator() * tau_pow)
            .collect();

        Self {
            pow_tau_g1,
            pow_tau_g2,
        }
    }

    /// generate a polinomial and its commitment from a `set` of points
    #[allow(non_snake_case)]
    pub fn poly_commitment_from_set(&self, set: &[(Scalar, Scalar)]) -> (Poly, Commitment) {
        let poly = Poly::lagrange(set);
        let commitment = self.eval_at_tau_g1(&poly);

        (poly, commitment)
    }

    /// Generates a proof that `points` exists in `set`
    #[allow(non_snake_case)]
    pub fn prove(&self, poly: &Poly, points: &[(Scalar, Scalar)]) -> Proof {
        // compute a lagrange poliomial I that have all the points to proof that are in the set
        // compute the polinomial Z that has roots (y=0) in all x's of I,
        //   so this is I=(x-x0)(x-x1)...(x-xn)
        let I = Poly::lagrange(points);
        let Z = Self::z_poly_of(points);

        // now compute that Q = ( P - I(x) ) / Z(x)
        // also check that the division does not have remainder
        let mut poly = poly.clone();
        poly -= &I;
        let (Q, remainder) = poly.div(&Z);
        assert!(remainder.is_zero());

        // the proof is evaluating the Q at tau in G1
        self.eval_at_tau_g1(&Q)
    }

    /// Verifies that `points` exists in `proof`
    /// is the duty of the caller to check if `proof.commitment` belongs
    ///    to the full set
    #[allow(non_snake_case)]
    pub fn verify(
        &self,
        commitment: &G1Projective,
        points: &[(Scalar, Scalar)],
        proof: &G1Projective,
    ) -> bool {
        let I = Poly::lagrange(points);
        let Z = Self::z_poly_of(points);

        let e1 = pairing(&proof.into(), &self.eval_at_tau_g2(&Z).into());

        let e2 = pairing(
            &(commitment - self.eval_at_tau_g1(&I)).into(),
            &G2Affine::generator(),
        );
        e1 == e2
    }
}

#[test]
fn test_multi_proof() {
    let kzg = Kzg::trusted_setup(5);
    let set = vec![
        (Scalar::from(1), Scalar::from(2)),
        (Scalar::from(2), Scalar::from(3)),
        (Scalar::from(3), Scalar::from(4)),
        (Scalar::from(4), Scalar::from(57)),
    ];
    let (p, c) = kzg.poly_commitment_from_set(&set);

    let proof01 = kzg.prove(&p, &vec![set[0].clone(), set[1].clone()]);
    assert!(kzg.verify(&c, &vec![set[0].clone(), set[1].clone()], &proof01));
    assert!(!kzg.verify(&c, &vec![set[0].clone()], &proof01));
    assert!(!kzg.verify(&c, &vec![set[0].clone(), set[2].clone()], &proof01));

    let proof0123 = kzg.prove(&p, &set);
    assert!(kzg.verify(&c, &set, &proof0123));
}
