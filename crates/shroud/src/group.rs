//! The group operations Shroud needs. The template backs these with engine intrinsics and the
//! client backs them with curve25519-dalek.

pub type Pt = [u8; 32];
pub type Sc = [u8; 32];

/// The compressed Ristretto255 basepoint.
pub const GENERATOR: Pt = [
    0xe2, 0xf2, 0xae, 0x0a, 0x6a, 0xbc, 0x4e, 0x71, 0xa8, 0x84, 0xa9, 0x61, 0xc5, 0x00, 0x51, 0x5f,
    0x58, 0xe3, 0x0b, 0x6a, 0xa5, 0x82, 0xdd, 0x8d, 0xb6, 0xa6, 0x59, 0x45, 0xe0, 0x8d, 0x2d, 0x76,
];

/// The compressed Ristretto255 identity.
pub const IDENTITY: Pt = [0u8; 32];

pub trait Group {
    fn msm(points: &[Pt], scalars: &[Sc]) -> Pt;
    fn add(a: &Pt, b: &Pt) -> Pt;
    fn sub(a: &Pt, b: &Pt) -> Pt;
    fn sc_add(a: &Sc, b: &Sc) -> Sc;
    fn sc_sub(a: &Sc, b: &Sc) -> Sc;
    fn sc_mul(a: &Sc, b: &Sc) -> Sc;
    /// SHA-512 of the concatenated input, reduced to a scalar.
    fn hash_scalar(data: &[u8]) -> Sc;
    fn point_ok(p: &Pt) -> bool;
    fn scalar_ok(s: &Sc) -> bool;

    fn mul(p: &Pt, s: &Sc) -> Pt {
        Self::msm(&[*p], &[*s])
    }
    fn mul_base(s: &Sc) -> Pt {
        Self::mul(&GENERATOR, s)
    }
}

/// A small scalar, for the fixed card and vote plaintexts.
pub const fn small(v: u8) -> Sc {
    let mut s = [0u8; 32];
    s[0] = v;
    s
}
