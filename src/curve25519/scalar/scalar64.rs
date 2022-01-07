use super::super::fe::load::{load_3i, load_4i};

const M: [u64; 5] = [
    0x12631a5cf5d3ed,
    0xf9dea2f79cd658,
    0x000000000014de,
    0x00000000000000,
    0x00000010000000,
];

const MU: [u64; 5] = [
    0x9ce5a30a2c131b,
    0x215d086329a7ed,
    0xffffffffeb2106,
    0xffffffffffffff,
    0x00000fffffffff,
];

#[inline]
const fn lt_modm(a: u64, b: u64) -> u64 {
    (a.wrapping_sub(b)) >> 63
}

#[rustfmt::skip]
fn reduce256_modm(r: &mut [u64; 5]) {
    // t = r - m
    let mut t = [0u64; 5];
    let mut pb = 0;
    pb += M[0]; let b = lt_modm(r[0], pb); t[0] = r[0].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += M[1]; let b = lt_modm(r[1], pb); t[1] = r[1].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += M[2]; let b = lt_modm(r[2], pb); t[2] = r[2].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += M[3]; let b = lt_modm(r[3], pb); t[3] = r[3].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += M[4]; let b = lt_modm(r[4], pb); t[4] = r[4].wrapping_sub(pb).wrapping_add(b << 32);

    // keep r if r was smaller than m
    let mask = b.wrapping_sub(1);

    r[0] ^= mask & (r[0] ^ t[0]);
    r[1] ^= mask & (r[1] ^ t[1]);
    r[2] ^= mask & (r[2] ^ t[2]);
    r[3] ^= mask & (r[3] ^ t[3]);
    r[4] ^= mask & (r[4] ^ t[4]);
}

#[inline]
const fn mul128(a: u64, b: u64) -> u128 {
    a as u128 * b as u128
}

const fn shr128(value: u128, shift: usize) -> u64 {
    (value >> shift) as u64
}

const MASK16: u64 = 0x0000_0000_0000_ffff;
const MASK40: u64 = 0x0000_00ff_ffff_ffff;
const MASK56: u64 = 0x00ff_ffff_ffff_ffff;

#[rustfmt::skip]
fn barrett_reduce256_modm(out: &mut [u64; 5], q1: &[u64; 5], r1: &[u64; 5]) {
    let mut r2 = [0; 5];
    let mut q3 = [0; 5];
    let mut c : u128;
    let mut mul : u128;
    let mut f : u64;
    let mut b : u64;
    let mut pb : u64;

    // q1 = x >> 248 = 264 bits = 5 56 bit elements
    // q2 = mu * q1
    // q3 = (q2 / 256(32+1)) = q2 / (2^8)^(32+1) = q2 >> 264

    c = mul128(MU[0], q1[3]);                 mul = mul128(MU[3], q1[0]); c += mul; mul = mul128(MU[1], q1[2]); c += mul; mul = mul128(MU[2], q1[1]); c += mul; f = shr128(c, 56);
    c = mul128(MU[0], q1[4]); c += f as u128; mul = mul128(MU[4], q1[0]); c += mul; mul = mul128(MU[3], q1[1]); c += mul; mul = mul128(MU[1], q1[3]); c += mul; mul = mul128(MU[2], q1[2]); c += mul;
    f = c as u64; q3[0] = (f >> 40) & MASK16; f = shr128(c, 56);
    c = mul128(MU[4], q1[1]); c += f as u128; mul = mul128(MU[1], q1[4]); c += mul; mul = mul128(MU[2], q1[3]); c += mul; mul = mul128(MU[3], q1[2]); c += mul;
    f = c as u64; q3[0] |= (f << 16) & MASK56; q3[1] = (f >> 40) & MASK16; f = shr128(c, 56);
    c = mul128(MU[4], q1[2]); c += f as u128; mul = mul128(MU[2], q1[4]); c += mul; mul = mul128(MU[3], q1[3]); c += mul;
    f = c as u64; q3[1] |= (f << 16) & MASK56; q3[2] = (f >> 40) & MASK16; f = shr128(c, 56);
    c = mul128(MU[4], q1[3]); c += f as u128; mul = mul128(MU[3], q1[4]); c += mul;
    f = c as u64; q3[2] |= (f << 16) & MASK56; q3[3] = (f >> 40) & MASK16; f = shr128(c, 56);
    c = mul128(MU[4], q1[4]); c += f as u128;
    f = c as u64; q3[3] |= (f << 16) & MASK56; q3[4] = (f >> 40) & MASK16; f = shr128(c, 56);
    q3[4] |= f << 16;

    c = mul128(M[0], q3[0]);
    r2[0] = (c as u64) & MASK56;  f = shr128(c, 56);
    c = mul128(M[0], q3[1]); c += f as u128; mul = mul128(M[1], q3[0]); c += mul;
    r2[1] = (c as u64) & MASK56; f = shr128(c, 56);
    c = mul128(M[0], q3[2]); c += f as u128; mul = mul128(M[2], q3[0]); c += mul; mul = mul128(M[1], q3[1]); c += mul;
    r2[2] = (c as u64) & MASK56; f = shr128(c, 56);
    c = mul128(M[0], q3[3]); c += f as u128; mul = mul128(M[3], q3[0]); c += mul; mul = mul128(M[1], q3[2]); c += mul; mul = mul128(M[2], q3[1]); c += mul;
    r2[3] = (c as u64) & MASK56; f = shr128(c, 56);
    c = mul128(M[0], q3[4]); c += f as u128; mul = mul128(M[4], q3[0]); c += mul; mul = mul128(M[3], q3[1]); c += mul; mul = mul128(M[1], q3[3]); c += mul; mul = mul128(M[2], q3[2]); c += mul;
    r2[4] = (c as u64) & MASK40;

    pb = 0;
    pb += r2[0]; b = lt_modm(r1[0], pb); out[0] = r1[0].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += r2[1]; b = lt_modm(r1[1], pb); out[1] = r1[1].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += r2[2]; b = lt_modm(r1[2], pb); out[2] = r1[2].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += r2[3]; b = lt_modm(r1[3], pb); out[3] = r1[3].wrapping_sub(pb).wrapping_add(b << 56); pb = b;
    pb += r2[4]; b = lt_modm(r1[4], pb); out[4] = r1[4].wrapping_sub(pb).wrapping_add(b << 40);

    reduce256_modm(out);
    reduce256_modm(out);
}

const fn from_bytes(bytes: &[u8; 32]) -> [u64; 5] {
    // load 8 bytes from input[ofs..ofs+7] as little endian u64
    #[inline]
    const fn load(bytes: &[u8; 32], ofs: usize) -> u64 {
        (bytes[ofs] as u64)
            | ((bytes[ofs + 1] as u64) << 8)
            | ((bytes[ofs + 2] as u64) << 16)
            | ((bytes[ofs + 3] as u64) << 24)
            | ((bytes[ofs + 4] as u64) << 32)
            | ((bytes[ofs + 5] as u64) << 40)
            | ((bytes[ofs + 6] as u64) << 48)
            | ((bytes[ofs + 7] as u64) << 56)
    }

    let x0 = load(bytes, 0);
    let x1 = load(bytes, 8);
    let x2 = load(bytes, 16);
    let x3 = load(bytes, 24);

    let out0 = x0 & MASK56;
    let out1 = (x0 >> 56 | x1 << 8) & MASK56;
    let out2 = (x1 >> 48 | x2 << 16) & MASK56;
    let out3 = (x2 >> 40 | x3 << 24) & MASK56;
    let out4 = x3 >> 32;
    [out0, out1, out2, out3, out4]
}

const fn to_bytes(limbs: &[u64; 5]) -> [u8; 32] {
    // contract limbs into saturated limbs
    let c0 = limbs[1] << 56 | limbs[0];
    let c1 = limbs[2] << 48 | limbs[1] >> 8;
    let c2 = limbs[3] << 40 | limbs[2] >> 16;
    let c3 = limbs[4] << 32 | limbs[3] >> 24;

    // write saturated limbs into little endian bytes
    let mut out = [0u8; 32];
    macro_rules! write8 {
        ($ofs:literal, $v:ident) => {
            let x = $v.to_le_bytes();
            out[$ofs] = x[0];
            out[$ofs + 1] = x[1];
            out[$ofs + 2] = x[2];
            out[$ofs + 3] = x[3];
            out[$ofs + 4] = x[4];
            out[$ofs + 5] = x[5];
            out[$ofs + 6] = x[6];
            out[$ofs + 7] = x[7];
        };
    }
    write8!(0, c0);
    write8!(8, c1);
    write8!(16, c2);
    write8!(24, c3);
    out
}

/*
Input:
    s[0]+256*s[1]+...+256^63*s[63] = s

Output:
    s[0]+256*s[1]+...+256^31*s[31] = s mod l
    where l = 2^252 + 27742317777372353535851937790883648493.
*/
#[rustfmt::skip]
#[must_use]
pub(crate) fn reduce(s: &[u8; 64]) -> [u8; 32] {
        // load 8 bytes from input[ofs..ofs+7] as little endian u64
        #[inline]
        const fn load(bytes: &[u8; 64], ofs: usize) -> u64 {
            (bytes[ofs] as u64)
                | ((bytes[ofs + 1] as u64) << 8)
                | ((bytes[ofs + 2] as u64) << 16)
                | ((bytes[ofs + 3] as u64) << 24)
                | ((bytes[ofs + 4] as u64) << 32)
                | ((bytes[ofs + 5] as u64) << 40)
                | ((bytes[ofs + 6] as u64) << 48)
                | ((bytes[ofs + 7] as u64) << 56)
        }

        // load little endian bytes to u64
        let x0 = load(s, 0);
        let x1 = load(s, 8);
        let x2 = load(s, 16);
        let x3 = load(s, 24);
        let x4 = load(s, 32);
        let x5 = load(s, 40);
        let x6 = load(s, 48);
        let x7 = load(s, 56);

        /* r1 = (x mod 256^(32+1)) = x mod (2^8)(31+1) = x & ((1 << 264) - 1) */
        let mut out = [0; 5];
        out[0] = (                     x0) & MASK56;
        out[1] = ((x0 >> 56) | (x1 <<  8)) & MASK56;
        out[2] = ((x1 >> 48) | (x2 << 16)) & MASK56;
        out[3] = ((x2 >> 40) | (x3 << 24)) & MASK56;
        out[4] = ((x3 >> 32) | (x4 << 32)) & MASK40;

        /*
        /* under 252 bits, no need to reduce */
        if (len < 32)
                return;
        */

        /* q1 = x >> 248 = 264 bits */
        let mut q1 = [0; 5];
        q1[0] = ((x3 >> 56) | (x4 <<  8)) & MASK56;
        q1[1] = ((x4 >> 48) | (x5 << 16)) & MASK56;
        q1[2] = ((x5 >> 40) | (x6 << 24)) & MASK56;
        q1[3] = ((x6 >> 32) | (x7 << 32)) & MASK56;
        q1[4] = x7 >> 24;

        let mut out2 = [0; 5];
        barrett_reduce256_modm(&mut out2, &q1, &out);

        to_bytes(&out2)
}

#[rustfmt::skip]
fn add(x: &[u64; 5], y: &[u64; 5]) -> [u64; 5] {
    let mut c;
    let mut r = [0; 5];

	c  = x[0] + y[0]; r[0] = c & MASK56; c >>= 56;
	c += x[1] + y[1]; r[1] = c & MASK56; c >>= 56;
	c += x[2] + y[2]; r[2] = c & MASK56; c >>= 56;
	c += x[3] + y[3]; r[3] = c & MASK56; c >>= 56;
	c += x[4] + y[4]; r[4] = c;

	reduce256_modm(&mut r);
    r
}

#[rustfmt::skip]
fn mul(x: &[u64; 5], y: &[u64; 5]) -> [u64; 5] {
    let mut q1 = [0; 5];
    let mut r1 = [0; 5];

	let c = mul128( x[0], y[0]);
	let f = c as u64; r1[0] = f & MASK56; let f = shr128(c, 56);
	let c = mul128( x[0], y[1]); let c = c + (f as u128); let mul = mul128(x[1], y[0]); let c = c + mul;
	let f = c as u64; r1[1] = f & MASK56; let f = shr128(c, 56);
	let c = mul128( x[0], y[2]); let c = c + (f as u128); let mul = mul128(x[2], y[0]); let c = c + mul; let mul = mul128(x[1], y[1]); let c = c + mul; 
	let f = c as u64; r1[2] = f & MASK56; let f = shr128(c, 56);
	let c = mul128( x[0], y[3]); let c = c + (f as u128); let mul = mul128(x[3], y[0]); let c = c + mul; let mul = mul128(x[1], y[2]); let c = c + mul; let mul = mul128(x[2], y[1]); let c = c + mul; 
	let f = c as u64; r1[3] = f & MASK56; let f = shr128(c, 56);
	let c = mul128( x[0], y[4]); let c = c + (f as u128); let mul = mul128(x[4], y[0]); let c = c + mul; let mul = mul128(x[3], y[1]); let c = c + mul; let mul = mul128(x[1], y[3]); let c = c + mul; let mul = mul128(x[2], y[2]); let c = c + mul; 
	let f = c as u64; r1[4] = f & MASK40; q1[0] = (f >> 24) & 0xffffffff; let f = shr128(c, 56);
	let c = mul128( x[4], y[1]); let c = c + (f as u128); let mul = mul128(x[1], y[4]); let c = c + mul; let mul = mul128(x[2], y[3]); let c = c + mul; let mul = mul128(x[3], y[2]); let c = c + mul; 
	let f = c as u64; q1[0] |= (f << 32) & MASK56; q1[1] = (f >> 24) & 0xffffffff; let f = shr128(c, 56);
	let c = mul128( x[4], y[2]); let c = c + (f as u128); let mul = mul128(x[2], y[4]); let c = c + mul; let mul = mul128(x[3], y[3]); let c = c + mul; 
	let f = c as u64; q1[1] |= (f << 32) & MASK56; q1[2] = (f >> 24) & 0xffffffff; let f = shr128(c, 56);
	let c = mul128( x[4], y[3]); let c = c + (f as u128); let mul = mul128(x[3], y[4]); let c = c + mul;
	let f = c as u64; q1[2] |= (f << 32) & MASK56; q1[3] = (f >> 24) & 0xffffffff; let f = shr128(c, 56);
	let c = mul128( x[4], y[4]); let c = c + (f as u128);
	let f = c as u64; q1[3] |= (f << 32) & MASK56; q1[4] = (f >> 24) & 0xffffffff; let f = shr128(c, 56);
	q1[4] |= f << 32;

    let mut out = [0;5];
	barrett_reduce256_modm(&mut out, &q1, &r1);
    out
}

/*
Input:
    a[0]+256*a[1]+...+256^31*a[31] = a
    b[0]+256*b[1]+...+256^31*b[31] = b
    c[0]+256*c[1]+...+256^31*c[31] = c

Output:
    s[0]+256*s[1]+...+256^31*s[31] = (ab+c) mod l
    where l = 2^252 + 27742317777372353535851937790883648493.
*/
#[rustfmt::skip]
pub(crate) fn muladd(s: &mut [u8; 32], a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]) {
    let a = from_bytes(a);
    let b = from_bytes(b);
    let c = from_bytes(c);

    let m = mul(&a, &b);
    let r = add(&m, &c);

    *s = to_bytes(&r);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve25519::testrng::{GeneratorOf, GeneratorOf2, GeneratorRaw};

    fn next_scalar(gen: &mut GeneratorRaw) -> [u64; 5] {
        let mut bytes = gen.bytes();
        bytes[31] &= 0x0f; // 2^252 max for simplicity
        from_bytes(&bytes)
    }

    #[test]
    fn serialization() {
        for scalar in GeneratorOf::new(0, 100, next_scalar) {
            let after_serialization = from_bytes(&to_bytes(&scalar));
            assert_eq!(scalar, after_serialization);
        }
    }

    #[test]
    fn add_iv() {
        struct Iv {
            a: [u8; 32],
            b: [u8; 32],
            r: [u8; 32],
        }

        let ivs = [Iv {
            a: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            b: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 15, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
            r: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 15, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
        }];

        for (i, iv) in ivs.iter().enumerate() {
            let a = from_bytes(&iv.a);
            let b = from_bytes(&iv.b);
            let r = add(&a, &b);
            assert_eq!(to_bytes(&r), iv.r, "iv test {} failed", i);
        }
    }

    #[test]
    fn add_commutes() {
        for (x, y) in GeneratorOf2::new(0, 100, next_scalar) {
            assert_eq!(add(&x, &y), add(&y, &x));
        }
    }

    #[test]
    fn mul_commutes() {
        for (x, y) in GeneratorOf2::new(0, 100, next_scalar) {
            assert_eq!(mul(&x, &y), mul(&y, &x));
        }
    }

    #[test]
    fn reduction() {
        assert_eq!(reduce(&[0; 64]), [0; 32]);
        assert_eq!(
            reduce(&[
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
            ]),
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        );

        assert_eq!(
            reduce(&[
                30, 1, 102, 252, 230, 223, 126, 62, 154, 62, 25, 173, 159, 16, 157, 227, 21, 140,
                223, 132, 84, 209, 86, 118, 35, 85, 26, 144, 12, 4, 76, 170, 93, 151, 77, 147, 32,
                213, 10, 135, 235, 26, 71, 94, 108, 45, 193, 229, 106, 233, 198, 109, 246, 81, 108,
                91, 63, 108, 220, 6, 119, 115, 9, 117
            ]),
            [
                4, 135, 152, 112, 4, 206, 189, 109, 105, 80, 162, 79, 191, 218, 37, 85, 225, 159,
                163, 149, 143, 3, 101, 222, 2, 81, 255, 223, 235, 242, 30, 12
            ]
        )
    }
}
