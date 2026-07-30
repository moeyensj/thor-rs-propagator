# thor-rs-propagator

The propagator contract for [THOR](https://github.com/moeyensj/thor) — the trait and state/ephemeris types every dynamical backend implements

<a href="https://github.com/moeyensj/thor-rs-propagator/actions/workflows/rust.yml"><img src="https://github.com/moeyensj/thor-rs-propagator/actions/workflows/rust.yml/badge.svg" alt="CI"></a>
<a href="https://crates.io/crates/thor-rs-propagator"><img src="https://img.shields.io/crates/v/thor-rs-propagator.svg?style=flat-square&label=crates.io" alt="crates.io"></a>
<a href="https://docs.rs/thor-rs-propagator"><img src="https://img.shields.io/docsrs/thor-rs-propagator?style=flat-square&label=docs.rs" alt="docs.rs"></a>
<br>
<a href="Cargo.toml"><img src="https://img.shields.io/badge/rustc-1.94%2B-orange?style=flat-square&logo=rust" alt="MSRV 1.94"></a>
<a href="LICENSE.md"><img src="https://img.shields.io/badge/License-BSD--3--Clause-blue.svg?style=flat-square" alt="License: BSD 3-Clause"></a>
<br>
<a href="https://claude.ai"><img src="https://img.shields.io/badge/Built%20with-Claude%20Code-D97757?logo=anthropic&logoColor=white&style=flat-square" alt="Built with Claude Code"></a>
<a href="https://b612foundation.org/asteroid-institute/"><img src="https://img.shields.io/badge/Asteroid%20Institute-b612foundation.org-1a1a2e?style=flat-square" alt="Asteroid Institute"></a>
<a href="https://dirac.astro.washington.edu/"><img src="https://img.shields.io/badge/DIRAC%20Institute-dirac.astro.washington.edu-1a1a2e?style=flat-square" alt="DIRAC Institute"></a>

---

The semver-governed seam between THOR's pipeline and its physics backends:
the [`Propagator`] trait, the fifteen state / ephemeris / configuration
types in its signatures, and the [`PropagatorError`] contract. Deliberately
tiny (≈550 lines) and dependency-light (serde + thiserror) so that backends
can live in their own crates and repositories without inheriting THOR.

```toml
[dependencies]
thor-rs-propagator = "0.1"
```

Not yet on crates.io — until first publish, depend on the repository:

```toml
[dependencies]
thor-rs-propagator = { git = "https://github.com/moeyensj/thor-rs-propagator.git", rev = "<pin>" }
```

## What it provides

- **`Propagator`** — six methods: `propagate` (states + optional STM/covariance
  transport), `compute_observers` (heliocentric observer states from
  observatory codes), `transform_state` (frame/origin changes),
  `generate_ephemeris` (topocentric spherical coordinates with light-time,
  optional covariance and observation Jacobians), plus defaulted
  `force_model` provenance and `compute_body_positions`.
- **The types** — `CartesianState`, `TestOrbit`, `ObserverState`,
  `PropagatedState`, `Ephemeris`, `SphericalState`, `Frame`, `Origin`, and the
  propagation/ephemeris configuration surface (`PropagationConfig`,
  `PropagationProfile`, `IntegratorProfile`, `EphemerisConfig`,
  `CovarianceMethod`, `CovarianceKind`, `CovarianceQuality`). All fields
  public, serde on the config/state types.
- **The error contract** — [`PropagatorError`], including the rule that
  matters most across backends: a capability a backend lacks is a loud typed
  [`PropagatorError::Unsupported`], never a silent substitution.

Known implementors: THOR's in-tree empyrean backend,
[thor-rs-assist](https://github.com/moeyensj/thor-rs-assist) (the fully-open
ASSIST adapter), a planned in-tree 2-body backend, and the mock propagators
in THOR's test suite.

## Quick start

```rust,ignore
use thor_rs_propagator::{
    CartesianState, Ephemeris, EphemerisConfig, Frame, ObserverState, Origin,
    PropagatedState, PropagationConfig, Propagator, PropagatorError, TestOrbit,
};

struct MyBackend;

impl Propagator for MyBackend {
    fn propagate(
        &self,
        orbit: &TestOrbit,
        epochs: &[f64], // MJD TDB
        config: &PropagationConfig,
    ) -> Result<Vec<PropagatedState>, PropagatorError> {
        todo!("your physics here")
    }

    fn compute_observers(
        &self,
        codes: &[String],
        epochs: &[f64],
    ) -> Result<Vec<ObserverState>, PropagatorError> {
        todo!()
    }

    fn transform_state(
        &self,
        state: &CartesianState,
        target_frame: Frame,
        target_origin: Origin,
    ) -> Result<CartesianState, PropagatorError> {
        todo!()
    }

    fn generate_ephemeris(
        &self,
        orbit: &TestOrbit,
        observers: &[ObserverState],
        config: &EphemerisConfig,
    ) -> Result<Vec<Ephemeris>, PropagatorError> {
        todo!()
    }
}
```

## Design contract

- **Frames and units**: heliocentric ecliptic J2000 states in AU and AU/day,
  epochs in MJD TDB, unless a field documents otherwise. THOR's
  [conventions page](https://github.com/moeyensj/thor/blob/main/docs/conventions.md)
  is the authoritative statement.
- **Semver seriousness**: this crate is the contract between independently
  versioned repositories. Consumers pin exact versions; every change to a
  type in a trait signature is a breaking release, and 0.x minor bumps are
  treated as breaking by cargo anyway.
- **Capability honesty**: optional capabilities (fast integrators,
  sample-based covariances, body ephemerides) are expressed as runtime
  `Unsupported` errors from backends that lack them — the pattern the
  `compute_body_positions` default impl demonstrates.

## Provenance

Extracted 2026-07-30 from THOR v2's `src/propagator/{api,types}.rs` and the
`PropagatorError` half of `src/error.rs` (repository `moeyensj/thor_rust`),
where the surface stabilized between 2026-04 and 2026-07. THOR consumes this
crate and re-exports every item, so its internal call sites are unchanged.

## Acknowledgments

Developed with support from the [Asteroid Institute](https://b612foundation.org/asteroid-institute/)
(a program of the B612 Foundation) and the [DIRAC Institute](https://dirac.astro.washington.edu/)
at the University of Washington.

## License

BSD 3-Clause. See [LICENSE.md](LICENSE.md).
