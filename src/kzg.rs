use bls12_381::*;
use super::poly::Poly;
use rand::Rng;

pub struct KZG {
    pow_tau_g1 : Vec<G1Projective>,
    pow_tau_g2 : Vec<G2Projective>,
}

pub struct OnePointProof {
    commitment : G1Projective,
    pi : G1Projective,
}

impl KZG {
    fn rand_scalars(len: usize) -> Vec<Scalar> {
        let mut rng = rand::thread_rng();
        let mut v = Vec::new();
        for _ in 0..len {
            let r : [u64;4]=[rng.gen(), rng.gen(), rng.gen(), rng.gen()];
            v.push(Scalar::from_raw(r));
        }
        v
    }

    pub fn trusted_setup(n : usize) -> Self {
        
        let tau = Scalar::from(Self::rand_scalars(1)[0]);
        
        let pow_tau_g1 : Vec<G1Projective> = (0..n).into_iter()
            .scan(Scalar::one(), |acc,_| { let v = acc.clone(); *acc *= tau; Some(v) } )
            .map(|tau_pow|G1Affine::generator() * tau_pow)
            .collect();
       
        let pow_tau_g2 : Vec<G2Projective> = (0..n).into_iter()
            .scan(Scalar::one(), |acc,_| { let v = acc.clone(); *acc *= tau; Some(v) } )
            .map(|tau_pow|G2Affine::generator() * tau_pow)
            .collect();

        Self { pow_tau_g1, pow_tau_g2 }
    }

    pub fn prove_one(&self, z : Scalar, y : Scalar ) -> OnePointProof {
        let rand = Self::rand_scalars(2);
        let p = Poly::lagrange(&vec![
            (z,y),
            (rand[0],rand[1]),
        ]);    

        let commitment = p.0.iter()
            .enumerate()
            .fold(G1Projective::identity(), |acc, (n,p)| acc + self.pow_tau_g1[n]*p);
        
        // q_zy = ( p - y ) / (x-z)
        let mut p2 = p.clone();
        p2 -= &Poly::new(vec![y]);
        let (q, remainder) = p2.div(&Poly::new(vec![-z,Scalar::one()]));
        assert!(remainder.is_zero());

        let pi = q.0.iter()
            .enumerate()
            .fold(G1Projective::identity(), |acc, (n,k)| acc + self.pow_tau_g1[n]*k);

        OnePointProof{commitment, pi}
    }

    pub fn verify_one(&self, z: Scalar, y: Scalar, proof : &OnePointProof) -> bool {
        let e1 = pairing(
            &proof.pi.into(),
            &(self.pow_tau_g2[1] - G2Affine::generator() * z).into()
        );
        
        let e2 = pairing(
            &(proof.commitment - G1Affine::generator() * y).into(),
            &G2Affine::generator()
        );
        return e1 == e2;
    }
}

#[test]
fn test_simple_proof() {
    let kzg = KZG::trusted_setup(3);
    let proof = kzg.prove_one(Scalar::from(5), Scalar::from(9));
    assert!(kzg.verify_one(Scalar::from(5), Scalar::from(9), &proof));
    assert!(!kzg.verify_one(Scalar::from(6), Scalar::from(9), &proof));
    assert!(!kzg.verify_one(Scalar::from(5), Scalar::from(8), &proof));
    assert!(!kzg.verify_one(Scalar::from(51), Scalar::from(91), &proof));
}
