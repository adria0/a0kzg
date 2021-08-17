use bls12_381::Scalar;

#[derive(Clone, Debug,PartialEq,Eq)]
pub struct Poly(pub Vec<Scalar>);

impl Poly {
    pub fn new(coeffs: Vec<Scalar>) -> Self {
        let mut poly = Poly(coeffs);
        poly.normalize();
        poly
    }

    pub fn lagrange(p : &[(Scalar,Scalar)]) -> Self {
        let k = p.len();
        let mut l = Poly::zero();
        for j in 0..k {
            let mut l_j = Poly::one();
            for i in 0..k {
                if i != j {
                   let c = (&p[j].0 - p[i].0).invert().unwrap();
                   l_j = &l_j * &Poly::new(vec![-(c*p[i].0),c]); 
                }
            }
            l += &(&l_j * &p[j].1);
        }
        l
    }

    pub fn eval(&self, x : Scalar) -> Scalar {
        let mut x_pow = Scalar::one();
        let mut y = self.0[0].clone(); 
        for i in 1..self.0.len() {
            x_pow = &x_pow * &x;
            y += &(&x_pow * &self.0[i]); 
        }
        y
    }
    
    pub fn eval_with_pows(&self, x_pow : &[Scalar]) -> Scalar {
        let mut y = self.0[0].clone(); 
        for i in 0..self.0.len() {
            y += &(&x_pow[i] * &self.0[i]); 
        }
        y
    }
    
    pub fn degree(&self) -> usize {
        self.0.len() - 1
    }
    pub fn from(coeffs: &[u64]) -> Self {
        Poly::new( 
            coeffs.iter()
            .map(|n| Scalar::from(*n))
            .collect::<Vec<Scalar>>())
    }
    fn normalize(&mut self) {
        if self.0.len() > 1 && self.0[self.0.len()-1] == Scalar::zero() {
            let zero = Scalar::zero();
            let first_non_zero = self.0.iter().rev().position(|p| p!=&zero);    
            if let Some(first_non_zero) = first_non_zero {
                self.0.resize(self.0.len()-first_non_zero, Scalar::zero());
            } else {
                self.0.resize(1, Scalar::zero()); 
            }
        }
    }
    pub fn is_zero(&self) -> bool {
       self.0.len()==1 && self.0[0] == Scalar::zero() 
    }
    pub fn zero() -> Self {
        Poly(vec![Scalar::zero()])
    }
    pub fn one() -> Self {
        Poly(vec![Scalar::one()])
    }
    pub fn set(&mut self, i: usize, p: Scalar) {
        if self.0.len() < i+1 {
            self.0.resize(i+1, Scalar::zero());
        }
        self.0[i]=p;
    }

    pub fn get(&mut self, i: usize) -> Option<&Scalar> {
        self.0.get(i)
    }

    pub fn div(self, d: &Poly) -> (Self, Self) {
        let (mut q, mut r) = (Poly::zero(), self);
        while !r.is_zero() && r.degree() >= d.degree() {
            let lead_r = r.0[r.0.len()-1];
            let lead_d = d.0[d.0.len()-1];
            let mut t = Poly::zero();
            t.set(r.0.len()-d.0.len(), lead_r * lead_d.invert().unwrap());
            q += &t;
            r -= &(d * &t);
        }

        (q,r)
    }
}
impl std::fmt::Display for Poly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first : bool = true;
        for i in (0..=self.degree()).rev() {
            let bi_n = num_bigint::BigUint::from_bytes_le(&self.0[i].to_bytes()).to_str_radix(10);
            let bi_inv = num_bigint::BigUint::from_bytes_le(&(-self.0[i]).to_bytes()).to_str_radix(10); 

            if bi_n == "0" {
                continue;
            }
            
            if bi_inv.len() < 20 && bi_n.len() > 20 {
                if bi_inv == "1" && i !=0  {
                    write!(f,"-")?;
                } else {
                    write!(f,"-{}",bi_inv)?;
                }
            } else {
                if !first {
                    write!(f,"+")?;
                }
                if i == 0 || bi_n != "1" {
                    write!(f,"{}",bi_n)?;
                }
            }
            if i >= 1 {
                write!(f,"x")?;
            }
            if i >= 2 {
                write!(f,"^{}",i)?;
            }
            first = false;
        }
        Ok(())
    }
}
impl std::ops::AddAssign<&Poly> for Poly {
    fn add_assign(&mut self, rhs: &Poly) {
        for n in 0..std::cmp::max(self.0.len(),rhs.0.len()) {
            if n >= self.0.len() {
                self.0.push(rhs.0[n].clone());
            } else if n < self.0.len() && n < rhs.0.len() {
                self.0[n] += rhs.0[n];
            }
        }
        self.normalize();
    }
} 

impl std::ops::AddAssign<&Scalar> for Poly {
    fn add_assign(&mut self, rhs: &Scalar) {
        self.0[0] += rhs;
    }
} 


impl std::ops::SubAssign<&Poly> for Poly {
    fn sub_assign(&mut self, rhs: &Poly) {
        for n in 0..std::cmp::max(self.0.len(),rhs.0.len()) {
            if n >= self.0.len() {
                self.0.push(rhs.0[n].clone());
            } else if n < self.0.len() && n < rhs.0.len() {
                self.0[n] -= rhs.0[n];
            }
        }
        self.normalize();
    }
} 

impl std::ops::Mul<&Poly> for &Poly {
    type Output = Poly;
    fn mul(self, rhs: &Poly) -> Self::Output {
        let mut mul : Vec<Scalar> = std::iter::repeat(Scalar::zero())
            .take(self.0.len()+rhs.0.len()-1)
            .collect();
        for n in 0..self.0.len() {
            for m in 0..rhs.0.len() {
                mul[n+m] += self.0[n] * rhs.0[m];
            }
        }
        Poly(mul)
    }
}


impl std::ops::Mul<&Scalar> for &Poly {
    type Output = Poly;
    fn mul(self, rhs: &Scalar) -> Self::Output {
        if rhs == &Scalar::zero() {
            Poly::zero()
        } else {
            Poly(self.0.iter().map(|v| v*rhs).collect::<Vec<_>>())
        }
    }
}

#[test]
fn test_poly_add() {
    let mut p246 = Poly::from(&[1,2,3]);
    p246 += &Poly::from(&[1,2,3]);
    assert_eq!(p246,Poly::from(&[2,4,6])); 

    let mut p24645 = Poly::from(&[1,2,3]);
    p24645 += &Poly::from(&[1,2,3,4,5]);
    assert_eq!(p24645,Poly::from(&[2,4,6,4,5])); 

    let mut p24646 = Poly::from(&[1,2,3,4,6]);
    p24646 += &Poly::from(&[1,2,3]);
    assert_eq!(p24646,Poly::from(&[2,4,6,4,6])); 
}

#[test]
fn test_poly_sub() {
    let mut p0 = Poly::from(&[1,2,3]);
    p0 -= &Poly::from(&[1,2,3]);
    assert_eq!(p0,Poly::from(&[0])); 

    let mut p003 = Poly::from(&[1,2,3]);
    p003 -= &Poly::from(&[1,2]);
    assert_eq!(p003,Poly::from(&[0,0,3])); 
}

#[test]
fn test_poly_mul() {
    assert_eq!(&Poly::from(&[5,0,10,6]) * &Poly::from(&[1,2,4]),Poly::from(&[5,10,30,26,52,24]));
}

#[test]
fn test_normalize() {
    let mut p1 = Poly::from(&[1,0,0,0]);
    p1.normalize();
    assert_eq!(p1, Poly::from(&[1])); 
    p1.normalize();
    assert_eq!(p1, Poly::from(&[1])); 
}

#[test]
fn test_get_set() {
    let mut p007 = Poly::zero();
    p007.set(2, Scalar::from(7));
    assert_eq!(p007, Poly::from(&[0,0,7]));
    assert_eq!(p007.get(2), Some(&Scalar::from(7)));
}


#[test]
fn test_div() {
    fn do_test(n: Poly, d:Poly) {
       let (q,r) = n.clone().div(&d);
       let mut n2 = &q * &d;
       n2 += &r;
       assert_eq!(n, n2);
    }

    assert_eq!((Poly::from(&[1,1]),Poly::zero()), Poly::from(&[1,2,1]).div(&Poly::from(&[1,1]))); 
    do_test(Poly::from(&[1]),Poly::from(&[1,1]));
    do_test(Poly::from(&[1,1]),Poly::from(&[1,1]));
    do_test(Poly::from(&[1,2,1]),Poly::from(&[1,1]));
    do_test(Poly::from(&[1,2,1,2,5,8,1,9]),Poly::from(&[1,1,5,4]));
}

#[test]
fn test_eval() {
    assert_eq!(Poly::from(&[1,2,1]).eval(Scalar::from(2)), Scalar::from(9));
}

#[test]
fn test_print() {
    assert_eq!("x^2+2x+1",format!("{}",Poly::from(&[1,2,1])));
    assert_eq!("x^2+1",format!("{}",Poly::from(&[1,0,1])));
    assert_eq!("x^2",format!("{}",Poly::from(&[0,0,1])));
    assert_eq!("2x^2",format!("{}",Poly::from(&[0,0,2])));
    assert_eq!("-4",format!("{}",Poly::new(vec![-Scalar::from(4)])));
    assert_eq!("-4x",format!("{}",Poly::new(vec![Scalar::zero(),-Scalar::from(4)])));
    assert_eq!("-x-2",format!("{}",Poly::new(vec![-Scalar::from(2),-Scalar::from(1)])));
    assert_eq!("x-2",format!("{}",Poly::new(vec![-Scalar::from(2),Scalar::from(1)])));
    
}

#[test]
fn test_lagrange_minimal() {
    assert_eq!(
        Poly::lagrange(&vec![
            (Scalar::from(1), Scalar::from(1)),
            (Scalar::from(2), Scalar::from(2))
        ]),
        Poly::from(&[0,1])
    );
}

#[test]
fn test_lagrange_multi() {
     let points = vec![
            (Scalar::from(12342), Scalar::from(22342)),
            (Scalar::from(2234), Scalar::from(22222)),
            (Scalar::from(3982394), Scalar::from(111114)),
            (Scalar::from(483838), Scalar::from(444444))
    ];
    let l = Poly::lagrange(&points);
    points.iter().for_each(|p| assert_eq!(l.eval(p.0),p.1));
}
