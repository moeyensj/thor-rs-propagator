//! thor-rs-propagator: the propagator contract for THOR.
//!
//! The [`Propagator`] trait and the state / ephemeris / configuration types
//! every dynamical backend implements. This crate is deliberately tiny and
//! dependency-light (serde + thiserror): it is the semver-governed seam
//! between THOR's pipeline and its physics backends, so backends can live in
//! their own crates and repositories.
//!
//! Frames and conventions: states are heliocentric ecliptic J2000 unless a
//! type says otherwise on the field; epochs are MJD TDB. Backends signal
//! capabilities they lack with [`PropagatorError::Unsupported`] — loudly,
//! never by silent substitution.

mod api;
mod error;
mod types;

pub use api::Propagator;
pub use error::PropagatorError;
pub use types::{
    CartesianState, CovarianceKind, CovarianceMethod, CovarianceQuality, Ephemeris,
    EphemerisConfig, Frame, IntegratorProfile, ObserverState, Origin, PropagatedState,
    PropagationConfig, PropagationProfile, SphericalState, TestOrbit,
};
