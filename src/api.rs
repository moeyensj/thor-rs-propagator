//! [`Propagator`] trait — the dispatch surface every backend implements.
//!
//! Concrete implementations live in the consumers (THOR's empyrean backend,
//! the `thor-rs-assist` adapter, test mocks). A pipeline consumes a
//! `&dyn Propagator` and never depends on the underlying physics engine.

use crate::error::PropagatorError;

use crate::types::{
    CartesianState, Ephemeris, EphemerisConfig, Frame, ObserverState, Origin, PropagatedState,
    PropagationConfig, TestOrbit,
};

/// Trait for orbit propagation and ephemeris generation.
///
/// Implementors provide the physics (force models, light-time iteration,
/// coordinate transforms). THOR's pipeline consumes the output without
/// knowing propagator internals.
///
/// Two planned implementations:
/// - **empyrean**: full N-body with AD covariance (closed-source)
/// - **ASSIST**: REBOUND + ASSIST via C FFI (open-source)
pub trait Propagator: Send + Sync {
    /// Propagate an orbit to one or more target epochs.
    ///
    /// Returns one [`PropagatedState`] per epoch, in the same order as the
    /// input `epochs` slice. All states are heliocentric ecliptic J2000.
    fn propagate(
        &self,
        orbit: &TestOrbit,
        epochs: &[f64], // MJD TDB
        config: &PropagationConfig,
    ) -> Result<Vec<PropagatedState>, PropagatorError>;

    /// Compute heliocentric observer states from observatory codes and epochs.
    ///
    /// Returns one [`ObserverState`] per (code, epoch) pair. The propagator
    /// is responsible for looking up geodetic coordinates (e.g., from MPC
    /// observatory tables) and computing heliocentric positions using its
    /// ephemeris data (SPK/BPC files).
    fn compute_observers(
        &self,
        codes: &[String],
        epochs: &[f64], // MJD TDB
    ) -> Result<Vec<ObserverState>, PropagatorError>;

    /// Transform a Cartesian state to a different frame and/or origin.
    ///
    /// Origin translations (e.g., SSB → Sun) require ephemeris data to
    /// look up the offset between centers at the given epoch. Frame
    /// rotations (e.g., equatorial → ecliptic) are purely geometric.
    fn transform_state(
        &self,
        state: &CartesianState,
        target_frame: Frame,
        target_origin: Origin,
    ) -> Result<CartesianState, PropagatorError>;

    /// Generate ephemeris for a test orbit as seen from a set of observers.
    ///
    /// For each observer at epoch \(t_k\), the propagator:
    /// 1. Propagates the orbit to \(t_k\)
    /// 2. Applies light-time correction to find the aberrated state
    /// 3. Computes the topocentric spherical coordinates
    ///    \((\rho, \alpha, \delta, \dot\rho, \dot\alpha, \dot\delta)\)
    /// 4. Optionally propagates the covariance and computes the observation Jacobian
    ///
    /// Returns one [`Ephemeris`] per observer, in the same order as the
    /// input `observers` slice.
    fn generate_ephemeris(
        &self,
        orbit: &TestOrbit,
        observers: &[ObserverState],
        config: &EphemerisConfig,
    ) -> Result<Vec<Ephemeris>, PropagatorError>;

    /// Short label identifying the propagator's force model.
    ///
    /// Used to populate the fitted orbit's `force_model` column
    /// so downstream consumers can tell which physics produced each fit.
    /// Defaults to `"unknown"`; implementations should override.
    fn force_model(&self) -> &'static str {
        "unknown"
    }

    /// Heliocentric ecliptic J2000 positions of a named solar-system body
    /// at the requested epochs.
    ///
    /// `body` is one of `"earth"`, `"mars"`, `"venus"`, `"jupiter"`, …
    /// (case-insensitive); the propagator resolves the name to its
    /// underlying ephemeris source. Used by the 2-body coarse-cone
    /// pre-filter (the kepler coarse-cone pre-filter) to
    /// gate the NEO close-approach shortcut.
    ///
    /// Default impl errors with [`PropagatorError::Other`] — backends that
    /// can supply planet ephemerides should override.
    fn compute_body_positions(
        &self,
        body: &str,
        _epochs: &[f64],
    ) -> Result<Vec<[f64; 3]>, PropagatorError> {
        Err(PropagatorError::Other(format!(
            "compute_body_positions not implemented for this propagator (body = {body})"
        )))
    }
}
