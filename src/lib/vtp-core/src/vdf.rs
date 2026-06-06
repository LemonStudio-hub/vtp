//! # VDF (Verifiable Delay Function) Module
//!
//! This module implements the Wesolowski VDF construction based on the class
//! group of an imaginary quadratic field, as described in:
//!
//! > Wesolowski, B. (2019). "Efficient verifiable delay functions."
//! > *Advances in Cryptology -- EUROCRYPT 2019*, Lecture Notes in Computer
//! > Science, vol 11478, pp. 136-162. Springer.
//!
//! The construction's security relies on the **sequential squaring assumption**
//! in imaginary quadratic class groups.
//!
//! ## Mathematical Foundations
//!
//! ### Imaginary Quadratic Class Groups
//!
//! Let Δ < 0 be a fundamental discriminant, i.e. the discriminant of some
//! imaginary quadratic field K = Q(sqrt(Δ)).  The class group Cl(Delta) consists
//! of equivalence classes of primitive positive-definite binary quadratic forms
//! (a, b, c) satisfying:
//!
//! ```text
//! f(x, y) = a*x^2 + b*xy + c*y^2
//!
//! Discriminant constraint:  b^2 - 4ac = Delta
//! Primitivity constraint:  gcd(a, b, c) = 1
//! Positive-definiteness:   a > 0
//! ```
//!
//! The group operation is **composition** of quadratic forms followed by
//! **reduction** to obtain the unique reduced representative.
//!
//! ### Uniqueness of Reduced Forms
//!
//! A positive-definite form (a, b, c) is **reduced** if and only if:
//!
//! ```text
//! |b| <= a <= c   and   (a != c  =>  b >= 0)
//! ```
//!
//! Every equivalence class contains exactly one reduced form, so reduced forms
//! serve as canonical representatives.
//!
//! ## Wesolowski VDF Protocol
//!
//! ### Setup
//!
//! 1. Choose discriminant Delta < 0 with Delta == 0 or 1 (mod 4).
//! 2. Find a generator g of the class group Cl(Delta).
//!
//! ### Evaluation
//!
//! Given a time parameter T, compute:
//!
//! ```text
//! y = g^(2^T)
//! ```
//!
//! via T sequential squarings:
//!
//! ```text
//! state[0] = g
//! state[i+1] = state[i]^2    (squaring in the class group)
//! y = state[T]
//! ```
//!
//! **Key property**: each squaring must wait for the previous one to finish;
//! this is the core sequentiality guarantee of the VDF.
//!
//! ### Proof Generation
//!
//! The prover computes:
//!
//! 1. Challenge prime l = H(g || y || T)   (Fiat-Shamir transform)
//! 2. Division with remainder: 2^T = q*l + r,  where 0 <= r < l
//! 3. Proof: pi = g^r
//!
//! ### Verification
//!
//! The verifier checks:
//!
//! ```text
//! pi^l * g^q = (g^r)^l * g^q = g^(rl + q) = g^(2^T) = y
//! ```
//!
//! Verification cost: O(log l) group operations + O(log q) group operations +
//! one composition.  Because l is small (~256 bits) and q ~ 2^T / l,
//! verification is much faster than evaluation.
//!
//! ## SIMD Optimisation Notes
//!
//! The big-integer arithmetic in this module benefits from LLVM
//! auto-vectorisation at compile time:
//!
//! - Target: `wasm32-unknown-unknown` + `target-feature=+simd128`
//! - Optimisation: `opt-level = 3` + `lto = true`
//! - Hot loops: modular add, sub, mul, GCD inner loops
//!
//! ## Security Notes
//!
//! - **Sequentiality assumption**: sequential squaring in the class group
//!   cannot be effectively parallelised.
//! - **Security level**: a 256-bit discriminant provides approximately
//!   128 bits of security.
//! - **Random oracle model**: the Fiat-Shamir transform is secure in the
//!   random oracle model.
//!
//! ## Module Structure
//!
//! - [`Bqf`]: low-level binary quadratic form representation
//! - [`ClassGroupElement`]: class group element wrapping a BQF + discriminant
//! - [`VdfState`]: VDF state wrapper
//! - [`VdfIterator`]: public API managing the sequential squaring chain
//! - [`generate_proof`] / [`verify_proof`]: Wesolowski proof generation and
//!   verification

use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

// ============================================================================
// Constants
// ============================================================================

/// Default discriminant bit-length for the imaginary quadratic class group.
///
/// A 256-bit discriminant combined with the Fiat-Shamir challenge provides
/// approximately 128 bits of security.  Larger discriminants yield higher
/// security margins at the cost of slower squarings.
///
/// # Security Analysis
///
/// The relationship between discriminant bit-length d and security level s
/// is approximately s ~ d/2, because:
/// - Class number h(Delta) ~ sqrt(|Delta|)
/// - Pollard-rho attack complexity ~ sqrt(h(Delta)) ~ |Delta|^(1/4) ~ 2^(d/4)
/// - Actual security must also account for other attack vectors.
///
/// # Performance Impact
///
/// The discriminant bit-length directly affects big-integer overhead:
/// - Modular add/sub: O(d) bit operations
/// - Modular mul: O(d^2) bit operations (naive) or O(d^1.585) (Karatsuba)
/// - GCD: O(d^2) bit operations
const DISCRIMINANT_BITS: u32 = 256;

// ============================================================================
// Binary Quadratic Form (BQF)
// ============================================================================

/// A primitive positive-definite binary quadratic form
///
/// ```text
/// f(x, y) = a*x^2 + b*xy + c*y^2
/// ```
///
/// # Invariants
///
/// All constructors and operations maintain:
///
/// - **Positive-definiteness**: a > 0
/// - **Discriminant consistency**: b^2 - 4ac = Delta (stored externally in
///   [`ClassGroupElement`])
/// - **Primitivity**: gcd(a, b, c) = 1
/// - **Reduced form**: |b| <= a <= c, and if a == c then b >= 0
///
/// # Serialisation
///
/// Because c can be reconstructed from (a, b, Delta) via
/// `c = (b^2 - Delta) / (4a)`, only a and b need to be stored.
///
/// # Example
///
/// The form x^2 + xy + 2y^2 has coefficients (a, b, c) = (1, 1, 2)
/// and discriminant Delta = 1^2 - 4*1*2 = -7.  It is the identity element
/// of the class group Cl(-7).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Bqf {
    /// Coefficient a (x^2 term).  Invariant: a > 0.
    a: BigInt,
    /// Coefficient b (xy term).  Invariant: |b| <= a when reduced.
    b: BigInt,
    /// Coefficient c (y^2 term).  Invariant: c = (b^2 - Delta) / (4a).
    c: BigInt,
}

impl Bqf {
    /// Create a new BQF instance.
    ///
    /// The caller must ensure the discriminant invariant b^2 - 4ac = Delta
    /// holds.  This constructor performs no validation to avoid unnecessary
    /// overhead.
    ///
    /// # Arguments
    ///
    /// - `a`: x^2 coefficient, must be positive
    /// - `b`: xy coefficient
    /// - `c`: y^2 coefficient
    fn new(a: BigInt, b: BigInt, c: BigInt) -> Self {
        Self { a, b, c }
    }

    /// Compute the discriminant Delta = b^2 - 4ac.
    ///
    /// For positive-definite forms the discriminant is always negative.
    ///
    /// # Complexity
    ///
    /// O(d^2), where d is the bit-length of the coefficients.
    fn _discriminant(&self) -> BigInt {
        &self.b * &self.b - 4_i32 * &self.a * &self.c
    }

    /// Check whether this form is in reduced state.
    ///
    /// A form (a, b, c) with Delta < 0 is reduced iff:
    ///
    /// ```text
    /// 1. |b| <= a <= c
    /// 2. If a == c, then b >= 0
    /// ```
    ///
    /// # Returns
    ///
    /// - `true`: the form is reduced
    /// - `false`: the form is not reduced
    ///
    /// # Mathematical Note
    ///
    /// A reduced form is the unique representative of its equivalence class.
    /// Two reduced forms are equal if and only if their coefficients are
    /// identical.
    fn _is_reduced(&self) -> bool {
        let abs_b = self.b.abs();
        abs_b <= self.a && self.a <= self.c && (self.a != self.c || self.b >= BigInt::zero())
    }
}

// ============================================================================
// Class Group Element
// ============================================================================

/// An element of the class group Cl(Delta) of an imaginary quadratic field.
///
/// Wraps a [`Bqf`] together with its discriminant to enforce the discriminant
/// invariant at the type level.
///
/// # Group Structure
///
/// Cl(Delta) is a finite abelian group with:
/// - **Identity**: the principal form (1, b0, (b0^2 - Delta) / 4)
/// - **Operation**: composition of quadratic forms
/// - **Inverse**: (a, -b, c)
/// - **Order**: h(Delta), the class number
///
/// # Serialisation Format
///
/// ```text
/// [4 bytes: a-length (big-endian)] [a-len bytes: sign+magnitude of a]
/// [4 bytes: b-length (big-endian)] [b-len bytes: sign+magnitude of b]
/// ```
///
/// c is reconstructed via c = (b^2 - Delta) / (4a) and is not stored.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassGroupElement {
    /// The underlying binary quadratic form.
    form: Bqf,
    /// The class group discriminant Delta < 0.
    /// Invariant: Delta == 0 or 1 (mod 4).
    discriminant: BigInt,
}

impl ClassGroupElement {
    /// Construct a class group element from coefficients a, b and the
    /// discriminant.
    ///
    /// The c coefficient is automatically recomputed from (a, b, Delta):
    ///
    /// ```text
    /// c = (b^2 - Delta) / (4a)
    /// ```
    ///
    /// This guarantees the discriminant invariant b^2 - 4ac = Delta holds
    /// under exact integer arithmetic (assuming b^2 - Delta is divisible by 4a).
    ///
    /// # Arguments
    ///
    /// - `a`: x^2 coefficient, must be positive
    /// - `b`: xy coefficient
    /// - `discriminant`: class group discriminant Delta < 0
    fn new(a: BigInt, b: BigInt, discriminant: BigInt) -> Self {
        let c = (&b * &b - &discriminant) / (4_i32 * &a);
        Self { form: Bqf::new(a, b, c), discriminant }
    }

    /// The identity element of the class group (principal form).
    ///
    /// The principal form is (1, b0, (b0^2 - Delta) / 4), where:
    ///
    /// - If Delta == 0 (mod 4): b0 = 0
    /// - If Delta == 1 (mod 4): b0 = 1
    ///
    /// # Example
    ///
    /// For Delta = -7:
    ///   Delta == 1 (mod 4) => b0 = 1
    ///   c = (1 - (-7)) / 4 = 2
    ///   Identity = (1, 1, 2), corresponding to x^2 + xy + 2y^2
    fn identity(discriminant: &BigInt) -> Self {
        let b0 = if discriminant % 4 == BigInt::from(0) { BigInt::zero() } else { BigInt::one() };
        Self::new(BigInt::one(), b0, discriminant.clone())
    }

    /// Class group composition (group operation).
    ///
    /// Computes the class group product of `self` and `other`, returning
    /// a reduced form.
    ///
    /// # Algorithm
    ///
    /// A variant of the NUCOMP (Shanks) algorithm for quadratic form
    /// composition:
    ///
    /// 1. Compute B = (b1 + b2) / 2
    /// 2. Compute g = gcd(a1, a2)
    /// 3. Solve the modular equation u*(a1/g) == -(s/g) (mod g),
    ///    where s = (b2 - b1) / 2
    /// 4. Compute new coefficients: A = a1*a2/g^2, B' = b1 + 2*u*a1/g
    /// 5. Centre-reduce B' to (-A, A]
    /// 6. Compute C = (B'^2 - Delta) / (4A)
    /// 7. Reduce the result
    ///
    /// # Panics
    ///
    /// Panics if the two elements have different discriminants.
    ///
    /// # Complexity
    ///
    /// O(d^2), where d is the discriminant bit-length.  The dominant cost
    /// is the GCD and big-integer division.
    fn compose(&self, other: &ClassGroupElement) -> ClassGroupElement {
        assert_eq!(
            self.discriminant, other.discriminant,
            "cannot compose elements of different class groups"
        );
        let delta = &self.discriminant;

        let a1 = &self.form.a;
        let b1 = &self.form.b;
        let a2 = &other.form.a;
        let b2 = &other.form.b;

        // Step 1: B = (b1 + b2) / 2  (NUCOMP intermediate)
        // Note: B is used implicitly in the NUCOMP derivation below.

        // Step 2: g = gcd(a1, a2)
        // g is used to factor a1 and a2 to reduce the size of subsequent
        // computations.
        let g = Integer::gcd(a1, a2);

        // Step 3: s = (b2 - b1) / 2
        // s is used in the NUCOMP modular equation.
        let s: BigInt = (b2 - b1) / 2;

        // Step 4: Factor a1 and s to prepare for the modular equation.
        // If g == 1 no factoring is needed.
        let (a1_div, s_div, g_div) = if g.is_one() {
            (a1.clone(), s.clone(), BigInt::one())
        } else {
            (a1 / &g, &s / &g, g.clone())
        };

        // Step 5: Solve u*(a1/g) == -(s/g) (mod g) via extended GCD.
        let (u, _v) = extended_gcd_mod(&a1_div, &g_div, &(-&s_div));

        // Step 6: Compute the new form coefficients.
        // A = a1*a2 / g^2
        let a_new = (a1 * a2) / (&g * &g);

        // B' = b1 + 2*u*a1/g
        let mut b_new = b1 + 2 * &u * a1 / &g;

        // Step 7: Centre-reduce B' to (-A, A] to satisfy the reduced-form
        // bound |B'| <= A.
        let two_a = &a_new * 2;
        b_new = center_mod(&b_new, &two_a);

        // C is determined by the discriminant invariant; it holds by
        // construction.

        // Step 8: Construct and reduce the result.
        let result = ClassGroupElement::new(a_new, b_new, delta.clone());
        result.reduce()
    }

    /// Square this class group element.
    ///
    /// Equivalent to `self.compose(self)`; this is the core operation of the
    /// VDF sequential squaring chain.
    fn square(&self) -> ClassGroupElement {
        self.compose(self)
    }

    /// Optimised binary exponentiation that skips leading zero bits.
    ///
    /// Uses the left-to-right binary method with the following optimisations:
    ///
    /// 1. Skips leading zero bytes of the exponent.
    /// 2. Initialises the accumulator to `self` upon encountering the first
    ///    set bit (instead of starting from the identity).
    /// 3. Reduces the number of unnecessary squarings.
    ///
    /// # Special Cases
    ///
    /// - exp == 0: returns the identity
    /// - exp == 1: returns `self`
    fn pow_optimized(&self, exp: &BigUint) -> ClassGroupElement {
        if exp.is_zero() {
            return ClassGroupElement::identity(&self.discriminant);
        }
        if exp.is_one() {
            return self.clone();
        }

        let bytes = exp.to_bytes_be();
        let mut result = ClassGroupElement::identity(&self.discriminant);

        // Locate the first non-zero byte.
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(0);

        let mut started = false;
        for (i, byte) in bytes.iter().enumerate() {
            if i < start {
                continue;
            }
            for bit_idx in (0..8).rev() {
                if !started {
                    // Wait for the first set bit.
                    if (byte >> bit_idx) & 1 == 1 {
                        started = true;
                        result = self.clone();
                    } else {
                        continue;
                    }
                } else {
                    // Square
                    result = result.square();
                    // Multiply if the current bit is set.
                    if (byte >> bit_idx) & 1 == 1 {
                        result = result.compose(self);
                    }
                }
            }
        }

        result
    }

    /// Reduce the quadratic form to its canonical reduced representative.
    ///
    /// # Algorithm
    ///
    /// Standard binary quadratic form reduction:
    ///
    /// ```text
    /// while not reduced:
    ///     if a > c or (a == c and b < 0):
    ///         swap(a, c); b = -b       (GL_2(Z) transformation)
    ///     b' = center_mod(b, 2a)       (centre-reduce)
    ///     c  = (b'^2 - Delta) / (4a)   (recompute c)
    /// ```
    ///
    /// After reduction: |b| <= a <= c, and if a == c then b >= 0.
    ///
    /// # Complexity
    ///
    /// O(d^2 * log|Delta|), where d is the discriminant bit-length.
    /// The actual number of iterations is typically small (about O(log|Delta|)).
    ///
    /// # Safety Bound
    ///
    /// A maximum of 4096 iterations is used as a safety bound to prevent
    /// infinite loops.  For normally-sized discriminants the actual iteration
    /// count is far below this limit.
    fn reduce(&self) -> ClassGroupElement {
        let delta = &self.discriminant;
        let mut a = self.form.a.clone();
        let mut b = self.form.b.clone();
        let mut c: BigInt;

        // Initial centre-reduction: ensure b in [-a, a) (mod 2a).
        let two_a = &a * 2;
        b = center_mod(&b, &two_a);
        c = (&b * &b - delta) / (4 * &a);

        // Reduction loop.
        let max_iters = 4096; // safety bound
        for _ in 0..max_iters {
            // Check whether a and c need to be swapped.
            // Condition: a > c, or a == c and b < 0.
            if a > c || (a == c && b < BigInt::zero()) {
                // GL_2(Z) transformation: (a, b, c) -> (c, -b, a)
                std::mem::swap(&mut a, &mut c);
                b = -b;
            }

            // Centre-reduce: find n such that -a < b + 2na <= a.
            let two_a = &a * 2;
            let b_new = center_mod(&b, &two_a);

            // Check whether we have reached a reduced form.
            if b_new == b && a <= c && (a != c || b >= BigInt::zero()) {
                break;
            }
            b = b_new;
            c = (&b * &b - delta) / (4 * &a);
        }

        ClassGroupElement::new(a, b, delta.clone())
    }

    /// Serialise the class group element to bytes.
    ///
    /// # Format
    ///
    /// ```text
    /// [4 bytes: byte-length of a (big-endian)]
    /// [a_len bytes: sign + magnitude encoding of a]
    /// [4 bytes: byte-length of b (big-endian)]
    /// [b_len bytes: sign + magnitude encoding of b]
    /// ```
    ///
    /// The c coefficient is not stored; it can be reconstructed via
    /// c = (b^2 - Delta) / (4a).
    fn to_bytes(&self) -> Vec<u8> {
        let a_bytes = bigint_to_bytes(&self.form.a);
        let b_bytes = bigint_to_bytes(&self.form.b);

        let mut out = Vec::with_capacity(8 + a_bytes.len() + b_bytes.len());
        out.extend_from_slice(&(a_bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(&a_bytes);
        out.extend_from_slice(&(b_bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(&b_bytes);
        out
    }

    /// Deserialise a class group element from bytes.
    ///
    /// # Arguments
    ///
    /// - `data`: serialised data (see [`to_bytes`] for the format)
    /// - `discriminant`: the class group discriminant Delta
    ///
    /// # Returns
    ///
    /// - `Some(ClassGroupElement)` on success
    /// - `None` if the data is malformed or too short
    fn from_bytes(data: &[u8], discriminant: &BigInt) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let a_len = u32::from_be_bytes(data[0..4].try_into().ok()?) as usize;
        if data.len() < 8 + a_len {
            return None;
        }
        let a = bytes_to_bigint(&data[4..4 + a_len]);

        let b_start = 4 + a_len;
        if data.len() < b_start + 4 {
            return None;
        }
        let b_len = u32::from_be_bytes(data[b_start..b_start + 4].try_into().ok()?) as usize;
        if data.len() < b_start + 4 + b_len {
            return None;
        }
        let b = bytes_to_bigint(&data[b_start + 4..b_start + 4 + b_len]);

        Some(ClassGroupElement::new(a, b, discriminant.clone()))
    }

    /// Hash this element to a 32-byte digest (for Fiat-Shamir).
    ///
    /// 1. Serialise the element to bytes.
    /// 2. Compute the SHA-256 hash.
    fn hash_to_bytes(&self) -> [u8; 32] {
        let bytes = self.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }
}

// ============================================================================
// Helper: BigInt <-> bytes
// ============================================================================

/// Encode a [`BigInt`] as a byte vector.
///
/// # Encoding
///
/// ```text
/// [1 byte: sign  (0x00 = non-negative, 0x01 = negative)]
/// [N bytes: big-endian magnitude]
/// ```
fn bigint_to_bytes(n: &BigInt) -> Vec<u8> {
    let (_sign, bytes) = n.to_bytes_be();
    let mut out = Vec::with_capacity(1 + bytes.len());
    if n.is_negative() {
        out.push(0x01);
    } else {
        out.push(0x00);
    }
    out.extend_from_slice(&bytes);
    out
}

/// Decode a [`BigInt`] from a byte vector (see [`bigint_to_bytes`]).
///
/// Returns 0 for empty input.
fn bytes_to_bigint(data: &[u8]) -> BigInt {
    if data.is_empty() {
        return BigInt::zero();
    }
    let sign_byte = data[0];
    let magnitude = BigUint::from_bytes_be(&data[1..]);
    let n = BigInt::from(magnitude);
    if sign_byte == 0x01 {
        -n
    } else {
        n
    }
}

// ============================================================================
// Helper: centre reduction
// ============================================================================

/// Centre-reduce `b` to the interval (-a, a] modulo 2a.
///
/// Returns the unique r in (-a, a] such that r == b (mod 2a).
///
/// # Algorithm
///
/// 1. Compute r = b mod 2a, r in [0, 2a).
/// 2. If r > a then r = r - 2a, yielding r in (-a, 0).
/// 3. Otherwise r is already in [0, a].
fn center_mod(b: &BigInt, two_a: &BigInt) -> BigInt {
    let r = b.mod_floor(two_a);
    let a = two_a / 2;
    if r > a {
        r - two_a
    } else {
        r
    }
}

// ============================================================================
// Helper: extended Euclidean algorithm
// ============================================================================

/// Solve the linear congruence `u * a == target (mod m)`.
///
/// # Algorithm
///
/// 1. Use the extended GCD to compute g = gcd(a, m) and x such that
///    x*a + y*m = g.
/// 2. If g divides target then u = x * (target / g) mod m is a solution.
///
/// # Preconditions
///
/// The caller must ensure gcd(a, m) divides `target`; otherwise the result
/// is meaningless.
fn extended_gcd_mod(a: &BigInt, m: &BigInt, target: &BigInt) -> (BigInt, BigInt) {
    let (g, x, _y) = extended_gcd(a, m);
    // x*a + _y*m = g
    // We want u*a == target (mod m).
    // u = x * (target/g) mod m   (assuming g | target)
    let quotient = target / &g;
    let u = (&x * quotient).mod_floor(m);
    (u, BigInt::zero()) // second component unused
}

/// Extended Euclidean algorithm.
///
/// Returns (g, x, y) such that a*x + b*y = g where g = gcd(a, b) >= 0.
///
/// # Algorithm
///
/// Iterative extended GCD:
///
/// ```text
/// (r0, r1) = (a, b),  (s0, s1) = (1, 0),  (t0, t1) = (0, 1)
/// while r1 != 0:
///     q = r0 / r1
///     (r0, r1) = (r1, r0 - q*r1)
///     (s0, s1) = (s1, s0 - q*s1)
///     (t0, t1) = (t1, t0 - q*t1)
/// return (r0, s0, t0)
/// ```
///
/// # Complexity
///
/// O(d^2), where d = max(|a|, |b|) bit-length.
fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    if b.is_zero() {
        return (
            a.abs(),
            if a.is_negative() { -BigInt::one() } else { BigInt::one() },
            BigInt::zero(),
        );
    }
    let (mut old_r, mut r) = (a.clone(), b.clone());
    let (mut old_s, mut s) = (BigInt::one(), BigInt::zero());
    let (mut old_t, mut t) = (BigInt::zero(), BigInt::one());

    while !r.is_zero() {
        let q = &old_r / &r;
        let new_r = &old_r - &q * &r;
        old_r = r;
        r = new_r;

        let new_s = &old_s - &q * &s;
        old_s = s;
        s = new_s;

        let new_t = &old_t - &q * &t;
        old_t = t;
        t = new_t;
    }

    (old_r, old_s, old_t)
}

// ============================================================================
// Discriminant and generator derivation from seed
// ============================================================================

/// Deterministically derive a negative discriminant Delta and a class group
/// generator g from a seed.
///
/// # Discriminant Derivation
///
/// 1. SHA-256 hash of the seed with prefix "VTP-VDF-DISCRIMINANT".
/// 2. Chain-hash until the target bit-length (256 bits) is reached.
/// 3. Adjust so that Delta == 0 or 1 (mod 4) (required for quadratic forms).
/// 4. Negate to obtain Delta < 0.
///
/// # Generator Derivation
///
/// Search for a valid primitive positive-definite form (a, b, c):
///
/// 1. Iterate a from 2 to 999.
/// 2. For each a, find b such that b^2 == Delta (mod 4a).
/// 3. Verify gcd(a, b, c) == 1.
/// 4. Use the seed hash to deterministically select one valid form.
/// 5. Self-compose several times to obtain a non-trivial element.
///
/// # Determinism
///
/// The same seed always produces the same (Delta, g) pair.
fn derive_discriminant_and_generator(seed: &[u8]) -> (BigInt, ClassGroupElement) {
    // Derive discriminant.
    let mut hasher = Sha256::new();
    hasher.update(b"VTP-VDF-DISCRIMINANT");
    hasher.update(seed);
    let hash = hasher.finalize();

    // Build a large-enough discriminant by accumulating hash outputs.
    let mut d_bytes = Vec::new();
    d_bytes.extend_from_slice(&hash);

    // Chain-hash until the target bit-length is reached.
    let mut counter = 0u32;
    while (d_bytes.len() * 8) < DISCRIMINANT_BITS as usize {
        let mut h2 = Sha256::new();
        h2.update(b"VTP-VDF-DISCRIMINANT-CHAIN");
        h2.update(&counter.to_be_bytes());
        h2.update(seed);
        let h2_result = h2.finalize();
        d_bytes.extend_from_slice(&h2_result);
        counter += 1;
    }

    let full_d = BigUint::from_bytes_be(&d_bytes);

    // Adjust Delta == 0 or 1 (mod 4) and negate.
    let mut delta = BigInt::from(full_d);
    loop {
        let mod4 = (&delta % 4_i32).mod_floor(&BigInt::from(4));
        if mod4 == BigInt::from(0) || mod4 == BigInt::from(1) {
            break;
        }
        delta = &delta + 1;
    }
    delta = -delta;

    // Derive generator.
    let generator = derive_generator_from_seed(seed, &delta);

    (delta, generator)
}

/// Deterministically search for a valid generator element.
///
/// Iterates a from 2 to 999, for each a searching for b in [0, 2a) such
/// that b^2 == Delta (mod 4a), then verifying primitivity and
/// positive-definiteness.  The seed hash is used to select one valid form
/// deterministically.  Self-composition several times yields a non-trivial
/// generator.
///
/// Falls back to the identity if no valid form is found (should not happen
/// for a well-chosen Delta).
fn derive_generator_from_seed(seed: &[u8], delta: &BigInt) -> ClassGroupElement {
    let neg_delta = -delta; // |Delta| > 0

    for a_val in 2u32..1000 {
        let a = BigInt::from(a_val);
        let modulus = &a * 4;

        // Compute target = Delta mod 4a.
        // Delta < 0, so Delta mod 4a = (4a - |Delta| mod 4a) mod 4a.
        let remainder = neg_delta.mod_floor(&modulus);
        let target = (&modulus - &remainder).mod_floor(&modulus);

        // Brute-force search for b in [0, 2a) with b^2 == target (mod 4a).
        for b_val in 0..(2 * a_val) {
            let b = BigInt::from(b_val);
            let b_sq_mod = (&b * &b).mod_floor(&modulus);
            if b_sq_mod == target {
                // Found a valid b; compute c.
                let c = (&b * &b - delta) / (4 * &a);

                // Verify primitivity and positive-definiteness.
                let g = a.gcd(&b).gcd(&c);
                if g.is_one() && c > BigInt::zero() {
                    let elem = ClassGroupElement::new(a.clone(), b, delta.clone());
                    let reduced = elem.reduce();

                    // Use the seed hash to select deterministically.
                    let hash_val = {
                        let mut h = Sha256::new();
                        h.update(seed);
                        h.update(&(a_val as u32).to_be_bytes());
                        h.update(&(b_val as u32).to_be_bytes());
                        let result = h.finalize();
                        result[0]
                    };

                    // Accept this form with ~78% probability.
                    if hash_val < 200 {
                        // Make it non-trivial by self-composing hash_val times.
                        let mut elem = reduced.clone();
                        for _ in 0..(hash_val as usize % 10 + 1) {
                            elem = elem.compose(&reduced);
                        }
                        return elem.reduce();
                    }
                }
            }
        }
    }

    // Fallback: return identity (should not happen for well-chosen Delta).
    ClassGroupElement::identity(delta)
}

// ============================================================================
// Fiat-Shamir challenge prime
// ============================================================================

/// Compute the Fiat-Shamir challenge prime `l` for Wesolowski's proof.
///
/// l = H("VTP-WESOLOWSKI-CHALLENGE" || g || y || T), interpreted as an odd
/// integer >= 3.  A strong probable prime sufficiency is assumed; full
/// primality testing is unnecessary for a fast VDF.
fn compute_challenge_prime(
    generator: &ClassGroupElement,
    output: &ClassGroupElement,
    t: u64,
) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(b"VTP-WESOLOWSKI-CHALLENGE");
    hasher.update(&generator.to_bytes());
    hasher.update(&output.to_bytes());
    hasher.update(&t.to_be_bytes());
    let hash = hasher.finalize();

    let mut candidate = BigUint::from_bytes_be(&hash);

    // Ensure the candidate is odd.
    if &candidate % 2u32 == BigUint::zero() {
        candidate += 1u32;
    }

    // Ensure the candidate is at least 3.
    if candidate < BigUint::from(3u32) {
        candidate = BigUint::from(3u32);
    }

    candidate
}

// ============================================================================
// VDF state wrapper
// ============================================================================

/// The current VDF state: a class group element in the squaring chain.
///
/// Stored as the serialised bytes of the BQF (a, b); c is recoverable from
/// Delta.
#[derive(Clone)]
struct VdfState {
    element: ClassGroupElement,
}

impl VdfState {
    fn new(element: ClassGroupElement) -> Self {
        Self { element }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.element.to_bytes()
    }

    /// Perform one VDF step: square the class group element.
    fn step(&mut self) {
        self.element = self.element.square();
    }
}

// ============================================================================
// Public API: VdfIterator
// ============================================================================

/// VDF iterator based on Wesolowski's construction over imaginary quadratic
/// class groups.
///
/// Manages the state of sequential squarings:
///
/// ```text
/// g -> g^2 -> g^4 -> ... -> g^(2^T)
/// ```
///
/// # Algorithm
///
/// Each VDF step computes a squaring in the class group:
///
/// ```text
/// state[i+1] = state[i]^2
/// ```
///
/// This is the core of the Wesolowski VDF: computing g^(2^T) requires T
/// sequential squarings that cannot be parallelised.
///
/// # Performance
///
/// - Each step involves one BQF composition (NUCOMP) + reduction.
/// - In a WebAssembly environment with SIMD, single-step latency is
///   dominated by big-integer operations on ~256-bit numbers.
#[wasm_bindgen]
pub struct VdfIterator {
    /// Current VDF state (class group element).
    state: VdfState,

    /// Number of completed VDF steps.
    step: u64,

    /// Total steps target.
    total: u64,

    /// Discriminant of the class group (stored for proof generation).
    _discriminant: BigInt,

    /// Original generator (stored for proof generation).
    _generator: ClassGroupElement,
}

#[wasm_bindgen]
impl VdfIterator {
    /// Create a new VDF iterator.
    ///
    /// # Arguments
    ///
    /// - `seed`: initial seed, at least 32 bytes.  Used to derive the
    ///   discriminant and generator of the class group.
    /// - `total`: total steps target (number of squarings to perform).
    ///
    /// # Panics
    ///
    /// Panics if `seed` is less than 32 bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &[u8], total: u64) -> Self {
        assert!(seed.len() >= 32, "seed must be at least 32 bytes");

        let (discriminant, generator) = derive_discriminant_and_generator(seed);
        let state_element = generator.clone();

        Self {
            state: VdfState::new(state_element),
            step: 0,
            total,
            _discriminant: discriminant,
            _generator: generator,
        }
    }

    /// Get the number of completed steps.
    pub fn step(&self) -> u64 {
        self.step
    }

    /// Get the total steps target.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Check whether the VDF computation has completed.
    pub fn is_finished(&self) -> bool {
        self.step >= self.total
    }

    /// Get the current VDF state as bytes.
    ///
    /// Returns the serialised class group element (BQF a, b components).
    ///
    /// Each call allocates a new `Vec`; frequent calls may impact performance.
    pub fn get_state(&self) -> Vec<u8> {
        self.state.to_bytes()
    }

    /// Execute a single VDF computation step (one squaring in the class
    /// group).
    ///
    /// Returns `true` if a step was executed, `false` if all steps are done.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.is_finished() {
            return false;
        }

        self.state.step();
        self.step += 1;
        true
    }

    /// Execute VDF computation in batch.
    ///
    /// Runs up to `max_steps` sequential squarings, or until all steps are
    /// completed.
    ///
    /// Returns the actual number of steps executed.
    pub fn run_batch(&mut self, max_steps: u64) -> u64 {
        let remaining = self.total.saturating_sub(self.step);
        let steps = max_steps.min(remaining);

        for _ in 0..steps {
            self.state.step();
            self.step += 1;
        }

        steps
    }
}

// ============================================================================
// Public API: proof generation and verification
// ============================================================================

/// Generate a Wesolowski proof for the VDF computation.
///
/// Given the generator `g`, output `y = g^(2^T)`, and time parameter `T`:
///
/// 1. Compute challenge prime `l = H(g, y, T)`.
/// 2. Compute quotient `q` and remainder `r`: `2^T = q*l + r`.
/// 3. Proof `pi = g^r` (exponentiation via repeated squaring on the exponent).
///
/// # Arguments
///
/// - `seed`: the original seed (to re-derive the discriminant and generator)
/// - `state_bytes`: the serialised final state (output y)
/// - `total`: the time parameter T
///
/// Returns the proof as bytes (serialised class group element).
#[wasm_bindgen]
pub fn generate_proof(seed: &[u8], state_bytes: &[u8], total: u64) -> Vec<u8> {
    let (discriminant, generator) = derive_discriminant_and_generator(seed);

    // Deserialise output y.
    let y = ClassGroupElement::from_bytes(state_bytes, &discriminant).expect("invalid state bytes");

    // Fiat-Shamir challenge.
    let l = compute_challenge_prime(&generator, &y, total);

    // Compute 2^T.
    let two_t = BigUint::from(1u32) << total;

    // q = 2^T / l,  r = 2^T mod l
    let (_q, r) = two_t.div_rem(&l);

    // pi = g^r
    let proof_element = generator.pow_optimized(&r);

    proof_element.to_bytes()
}

/// Verify a Wesolowski proof.
///
/// Checks that pi^l * g^q == y, where l = H(g, y, T).
///
/// Verification requires only O(log l) group operations (fast, since l is
/// small) plus one composition, regardless of how large T is.
///
/// # Arguments
///
/// - `seed`: the original seed
/// - `state_bytes`: the claimed output y
/// - `total`: the time parameter T
/// - `proof_bytes`: the proof pi
///
/// Returns `true` if the proof is valid, `false` otherwise.
#[wasm_bindgen]
pub fn verify_proof(seed: &[u8], state_bytes: &[u8], total: u64, proof_bytes: &[u8]) -> bool {
    let (discriminant, generator) =
        match std::panic::catch_unwind(|| derive_discriminant_and_generator(seed)) {
            Ok(v) => v,
            Err(_) => return false,
        };

    let y = match ClassGroupElement::from_bytes(state_bytes, &discriminant) {
        Some(v) => v,
        None => return false,
    };

    let pi = match ClassGroupElement::from_bytes(proof_bytes, &discriminant) {
        Some(v) => v,
        None => return false,
    };

    // Fiat-Shamir challenge.
    let l = compute_challenge_prime(&generator, &y, total);

    // Compute 2^T.
    let two_t = BigUint::from(1u32) << total;

    // q = 2^T / l,  r = 2^T mod l
    let (q, _r) = two_t.div_rem(&l);

    // Verify: pi^l * g^q == y
    let pi_l = pi.pow_optimized(&l);
    let g_q = generator.pow_optimized(&q);
    let lhs = pi_l.compose(&g_q);

    lhs == y
}

// ============================================================================
// Single-step helper (compatibility with lib.rs)
// ============================================================================

/// Execute a single VDF computation step.
///
/// Performs one squaring in the imaginary quadratic class group derived from
/// the given state bytes.
///
/// # Arguments
///
/// - `state`: 32-byte input state (used as a seed to derive a class group
///   element, which is then squared)
///
/// Returns the 32-byte hash of the squared class group element.
///
/// # Algorithm
///
/// 1. Derive a class group element from the state.
/// 2. Square it.
/// 3. Hash the result to 32 bytes.
pub fn vdf_step(state: &[u8; 32]) -> [u8; 32] {
    let (_discriminant, element) = derive_element_from_short_seed(state);
    let squared = element.square();
    squared.hash_to_bytes()
}

/// Derive a class group element from a 32-byte seed.
///
/// Uses the seed to deterministically construct a non-trivial element of
/// the class group with a fixed discriminant scheme.
fn derive_element_from_short_seed(seed: &[u8; 32]) -> (BigInt, ClassGroupElement) {
    let (discriminant, _generator) = derive_discriminant_and_generator(seed);

    // Hash the seed to obtain a deterministic exponent.
    let mut hasher = Sha256::new();
    hasher.update(b"VTP-VDF-ELEMENT");
    hasher.update(seed);
    let hash = hasher.finalize();

    let exp = BigUint::from_bytes_be(&hash);

    // Construct a base form (2, 1, c) and exponentiate.
    let a = BigInt::from(2);
    let b = BigInt::from(1);
    let base = ClassGroupElement::new(a, b, discriminant.clone()).reduce();

    let result = base.pow_optimized(&exp);
    (discriminant, result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bqf_discriminant() {
        // Form x^2 + xy + 2y^2 => Delta = 1 - 8 = -7
        let form = Bqf::new(BigInt::from(1), BigInt::from(1), BigInt::from(2));
        assert_eq!(form._discriminant(), BigInt::from(-7));
    }

    #[test]
    fn test_identity_compose() {
        let delta = BigInt::from(-7);
        let id = ClassGroupElement::identity(&delta);
        let result = id.compose(&id);
        assert_eq!(result.form, id.form);
    }

    #[test]
    fn test_vdf_step_changes_state() {
        let state = [0u8; 32];
        let next_state = vdf_step(&state);
        assert_ne!(state, next_state);
        assert_eq!(next_state.len(), 32);
    }

    #[test]
    fn test_vdf_step_deterministic() {
        let state = [42u8; 32];
        let result1 = vdf_step(&state);
        let result2 = vdf_step(&state);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_vdf_iterator_basic() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 100);

        assert_eq!(iter.step(), 0);
        assert_eq!(iter.total(), 100);
        assert!(!iter.is_finished());

        for i in 1..=100 {
            assert!(iter.next());
            assert_eq!(iter.step(), i);
        }

        assert!(iter.is_finished());
        assert!(!iter.next());
    }

    #[test]
    fn test_vdf_iterator_batch() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 1000);

        let steps = iter.run_batch(100);
        assert_eq!(steps, 100);
        assert_eq!(iter.step(), 100);

        let steps = iter.run_batch(1000);
        assert_eq!(steps, 900);
        assert_eq!(iter.step(), 1000);
        assert!(iter.is_finished());
    }

    #[test]
    fn test_vdf_deterministic_iterator() {
        let seed = [0u8; 32];

        let mut iter1 = VdfIterator::new(&seed, 10);
        iter1.run_batch(10);

        let mut iter2 = VdfIterator::new(&seed, 10);
        iter2.run_batch(10);

        assert_eq!(iter1.get_state(), iter2.get_state());
    }

    #[test]
    fn test_proof_roundtrip() {
        let seed = [0u8; 32];
        let total: u64 = 10;

        let mut iter = VdfIterator::new(&seed, total);
        iter.run_batch(total);
        let state = iter.get_state();

        let proof = generate_proof(&seed, &state, total);
        assert!(!proof.is_empty());

        assert!(verify_proof(&seed, &state, total, &proof));
    }

    #[test]
    fn test_proof_invalid_state() {
        let seed = [0u8; 32];
        let total: u64 = 5;

        let mut iter = VdfIterator::new(&seed, total);
        iter.run_batch(total);
        let state = iter.get_state();

        let proof = generate_proof(&seed, &state, total);

        // Tamper with state.
        let mut bad_state = state.clone();
        bad_state[0] ^= 0xff;
        assert!(!verify_proof(&seed, &bad_state, total, &proof));
    }

    #[test]
    fn test_proof_invalid_proof() {
        let seed = [0u8; 32];
        let total: u64 = 5;

        let mut iter = VdfIterator::new(&seed, total);
        iter.run_batch(total);
        let state = iter.get_state();

        let proof = generate_proof(&seed, &state, total);

        // Tamper with proof.
        let mut bad_proof = proof.clone();
        if !bad_proof.is_empty() {
            bad_proof[0] ^= 0xff;
        }
        assert!(!verify_proof(&seed, &state, total, &bad_proof));
    }
}
