//! The client-side group, on curve25519-dalek.

use curve25519_dalek::{
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::{Identity, MultiscalarMul},
};
use sha2::{Digest, Sha512};

use crate::group::{Group, Pt, Sc};

pub struct Native;

fn point(p: &Pt) -> RistrettoPoint {
    CompressedRistretto(*p).decompress().expect("canonical point")
}

fn scalar(s: &Sc) -> Scalar {
    Option::from(Scalar::from_canonical_bytes(*s)).expect("canonical scalar")
}

impl Group for Native {
    fn msm(points: &[Pt], scalars: &[Sc]) -> Pt {
        if points.is_empty() {
            return RistrettoPoint::identity().compress().to_bytes();
        }
        RistrettoPoint::multiscalar_mul(scalars.iter().map(scalar), points.iter().map(point))
            .compress()
            .to_bytes()
    }
    fn add(a: &Pt, b: &Pt) -> Pt {
        (point(a) + point(b)).compress().to_bytes()
    }
    fn sub(a: &Pt, b: &Pt) -> Pt {
        (point(a) - point(b)).compress().to_bytes()
    }
    fn sc_add(a: &Sc, b: &Sc) -> Sc {
        (scalar(a) + scalar(b)).to_bytes()
    }
    fn sc_sub(a: &Sc, b: &Sc) -> Sc {
        (scalar(a) - scalar(b)).to_bytes()
    }
    fn sc_mul(a: &Sc, b: &Sc) -> Sc {
        (scalar(a) * scalar(b)).to_bytes()
    }
    fn hash_scalar(data: &[u8]) -> Sc {
        let digest = Sha512::digest(data);
        let mut wide = [0u8; 64];
        wide.copy_from_slice(&digest);
        Scalar::from_bytes_mod_order_wide(&wide).to_bytes()
    }
    fn point_ok(p: &Pt) -> bool {
        CompressedRistretto(*p).decompress().is_some()
    }
    fn scalar_ok(s: &Sc) -> bool {
        bool::from(Scalar::from_canonical_bytes(*s).is_some())
    }
}
