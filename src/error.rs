//! The error contract every propagator backend speaks.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PropagatorError {
    #[error("propagation failed: {0}")]
    PropagationFailed(String),

    #[error("light-time iteration did not converge")]
    LightTimeConvergence,

    #[error("invalid orbit: {0}")]
    InvalidOrbit(String),

    #[error("invalid observer: {0}")]
    InvalidObserver(String),

    #[error("frame/origin contract violated in {context}: expected {expected}, got {got}")]
    FrameOriginMismatch {
        context: String,
        expected: String,
        got: String,
    },

    /// A requested capability (covariance method, integrator, …) is not
    /// implemented by this backend. Backends MUST return this rather than
    /// silently substituting a different method — the caller decides
    /// whether a substitute is acceptable.
    #[error("unsupported by this propagator backend: {0}")]
    Unsupported(String),

    /// Covariance output was requested (`EphemerisConfig::compute_covariance`)
    /// but the input orbit carries no covariance to propagate. Fabricating
    /// one would silently poison every downstream gate, so this is a hard
    /// error at the call boundary.
    #[error("covariance requested but input orbit carries none: {0}")]
    MissingCovariance(String),

    #[error("{0}")]
    Other(String),
}
