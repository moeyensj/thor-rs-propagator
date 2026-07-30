//! Data types passed between THOR's pipeline and any [`super::Propagator`]
//! backend: states, ephemeris records, configuration structs.

use crate::error::PropagatorError;

/// Reference frame for coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Frame {
    /// J2000 ecliptic frame (inertial).
    EclipticJ2000,
    /// International Celestial Reference Frame (inertial, equatorial).
    Equatorial,
}

/// Origin (center body) of a coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Origin {
    /// Solar System Barycenter (default for propagation).
    SolarSystemBarycenter,
    /// Sun center.
    Sun,
    /// Earth center. Used by THOR's gnomonic projection when the
    /// tangent plane is built from a geocentric line-of-sight (CA-NEO and
    /// other near-Earth regimes).
    Earth,
}

/// Cartesian state in a specified frame and origin.
///
/// State vector \(\mathbf{s} = (x, y, z, \dot{x}, \dot{y}, \dot{z})\)
/// with position in AU, velocity in AU/day, and epoch as MJD TDB.
/// Optionally carries a \(6 \times 6\) covariance matrix in the same frame.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CartesianState {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub epoch: f64, // MJD TDB
    pub frame: Frame,
    pub origin: Origin,
    /// Optional \(6 \times 6\) covariance matrix (row-major) in the same frame.
    pub covariance: Option<[[f64; 6]; 6]>,
}

impl CartesianState {
    /// Loudly require this state to carry the given `frame` and `origin`.
    ///
    /// Returns [`PropagatorError::FrameOriginMismatch`] — never a silent
    /// fallback — when either tag differs. Boundary code that assumes a
    /// specific frame/origin should call this instead of trusting the
    /// doc-comment convention, mirroring the validation the assist backend
    /// already performs on its inputs (`assist::to_assist_orbit`). `context`
    /// names the call site so the error pinpoints which contract was broken.
    pub fn require(
        &self,
        frame: Frame,
        origin: Origin,
        context: &str,
    ) -> Result<(), PropagatorError> {
        if self.frame != frame || self.origin != origin {
            return Err(PropagatorError::FrameOriginMismatch {
                context: context.to_string(),
                expected: format!("{frame:?}/{origin:?}"),
                got: format!("{:?}/{:?}", self.frame, self.origin),
            });
        }
        Ok(())
    }

    /// Convenience wrapper for the THOR canonical state frame: heliocentric
    /// ecliptic J2000 ([`Frame::EclipticJ2000`] + [`Origin::Sun`]).
    pub fn require_helio_ecliptic(&self, context: &str) -> Result<(), PropagatorError> {
        self.require(Frame::EclipticJ2000, Origin::Sun, context)
    }
}

/// Spherical state in a specified frame and origin.
///
/// State vector \((\rho, \lambda, \beta, \dot\rho, \dot\lambda, \dot\beta)\)
/// with range in AU, angles in degrees, and rates in AU/day or degrees/day.
/// Optionally carries a \(6 \times 6\) covariance matrix in the same coordinates.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SphericalState {
    /// Range \(\rho\) (AU).
    pub rho: f64,
    /// Longitude \(\lambda\) — right ascension or ecliptic longitude (degrees).
    pub lon: f64,
    /// Latitude \(\beta\) — declination or ecliptic latitude (degrees).
    pub lat: f64,
    /// Range rate \(\dot\rho\) (AU/day).
    pub vrho: f64,
    /// Longitude rate \(\dot\lambda\) (degrees/day).
    pub vlon: f64,
    /// Latitude rate \(\dot\beta\) (degrees/day).
    pub vlat: f64,
    /// Epoch (MJD TDB).
    pub epoch: f64,
    pub frame: Frame,
    pub origin: Origin,
    /// Optional \(6 \times 6\) covariance matrix (row-major) in **raw**
    /// \((\rho, \lambda, \beta, \dot\rho, \dot\lambda, \dot\beta)\)
    /// coordinates — i.e., `cov[1][1]` is `Var(λ)` (longitude variance,
    /// not λ·cos β). Differs from
    /// the observation store's `ra_sigma_sq`, which stores
    /// the sky-projected `Var(α·cos δ)`. Pair with
    /// the residuals stage to project both
    /// Δ and Σ into the sky-uniform `(lon·cos(lat), lat)` frame for
    /// declination-invariant Mahalanobis distance.
    pub covariance: Option<[[f64; 6]; 6]>,
}

/// A test orbit to be processed by the pipeline.
///
/// Ecliptic J2000 Cartesian coordinates. Covariance, if present,
/// is carried inside `state.covariance`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestOrbit {
    pub id: String,
    /// Ground-truth object ID (for analysis/validation).
    pub object_id: Option<String>,
    /// Bundle ID grouping this test orbit with others (e.g., HEALPix pixel).
    /// Used for tiered merge-and-extend: orbits within a bundle are
    /// deduplicated together before cross-bundle merging.
    pub bundle_id: Option<String>,
    /// HEALPix nside at which `bundle_id` is the pixel. Required for
    /// healpix-nested split (`pipeline::split::split_orbit_healpixel`)
    /// to know what depth to split from. Set to 0 for derived test orbits
    /// (e.g. constructed from FittedOrbit during M&E) where the source
    /// healpix grid is unknown — those won't be split.
    pub nside: u32,
    pub state: CartesianState,
}

/// An observer's heliocentric state at a given epoch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObserverState {
    /// MPC observatory code (e.g., "I11", "W84", "500").
    pub code: String,
    /// Heliocentric ecliptic J2000 position and velocity.
    pub state: CartesianState,
}

/// How a propagated covariance was derived — the resolved kind at the
/// output epoch. Mirrors the empyrean engine's provenance tags so the
/// escalated (close-approach) covariance is distinguishable from the
/// bare linear \(\Phi \Sigma_0 \Phi^T\) mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CovarianceKind {
    /// Linear STM mapping \(\Phi \Sigma_0 \Phi^T\).
    Linear,
    /// Second-order (STM + STT) correction.
    SecondOrder,
    /// Third-order extension.
    ThirdOrder,
    /// Adaptive Gaussian mixture, moment-collapsed to a single second
    /// moment by the engine. NOT the full mixture — treat as a better
    /// Gaussian, not as mixture-aware clustering input.
    Mixture,
    /// Monte Carlo sample covariance.
    MonteCarlo,
    /// Sigma-point sample covariance: second moment of the propagated
    /// canonical 2N+1 sigma-point set (empyrean ≥ 0.8.1). Deterministic
    /// and parameter-free.
    SigmaPoint,
}

/// Definiteness of a propagated covariance matrix, as reported by the
/// backend. `Repaired`/`Indefinite` carry the most-negative eigenvalue
/// so consumers (adaptive clustering) can widen or reject.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CovarianceQuality {
    /// All eigenvalues positive within round-off.
    PositiveDefinite,
    /// Explicitly repaired to PSD; `min_eig` is the value *before* repair.
    Repaired { min_eig: f64 },
    /// At least one meaningfully negative eigenvalue.
    Indefinite { min_eig: f64 },
}

/// Result of propagating an orbit to a single epoch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropagatedState {
    /// Heliocentric ecliptic J2000 state at the target epoch.
    pub state: CartesianState,
    /// \(6 \times 6\) state transition matrix \(\Phi(t, t_0)\), row-major.
    ///
    /// Maps initial state perturbations to propagated state perturbations:
    /// \(\delta\mathbf{x}(t) = \Phi(t, t_0) \, \delta\mathbf{x}_0\).
    ///
    /// Only populated when `PropagationConfig::compute_stm` is true.
    pub stm: Option<[[f64; 6]; 6]>,
    /// Provenance of `state.covariance`: how it was derived. `Some` iff
    /// the covariance is populated. ASSIST always tags [`CovarianceKind::Linear`]
    /// (its only machinery); empyrean reports the resolved kind from its
    /// `TaggedCovariance` readback, which escalates through close approaches
    /// under the Auto uncertainty method.
    #[serde(default)]
    pub covariance_kind: Option<CovarianceKind>,
    /// Definiteness of `state.covariance`. `Some` iff the covariance is
    /// populated. Backends must never emit an `Indefinite` covariance
    /// silently — the empyrean adapter hard-errors on it; ASSIST tags
    /// honestly after a Cholesky check.
    #[serde(default)]
    pub covariance_quality: Option<CovarianceQuality>,
}

/// Ephemeris for a single (orbit, observer) pair at one epoch.
///
/// Contains the predicted on-sky spherical state
/// \((\rho, \alpha, \delta, \dot\rho, \dot\alpha, \dot\delta)\), the
/// light-time-corrected (aberrated) heliocentric state of the object,
/// and the observer's heliocentric state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ephemeris {
    /// Predicted on-sky spherical state (range, RA, Dec and rates).
    /// Covariance in spherical coordinates is carried inside this state.
    pub state: SphericalState,
    /// Aberrated heliocentric ecliptic state of the object
    /// (light-time corrected). This is the gnomonic projection center.
    ///
    /// **Epoch convention**: `aberrated_state.epoch = t_obs − τ` (the
    /// emission time), matching the position/velocity the state actually
    /// represents. Backends are responsible for setting this correctly so
    /// the CartesianState is internally self-consistent — re-propagators
    /// can start from `aberrated_state` without further bookkeeping.
    pub aberrated_state: CartesianState,
    /// Observer's heliocentric ecliptic state. Needed for ranging.
    pub observer_state: CartesianState,
    /// One-way light time \(\tau\) in days.
    pub light_time: Option<f64>,
    /// \(6 \times 6\) observation Jacobian, row-major:
    /// \[\frac{\partial(\rho, \alpha, \delta, \dot\rho, \dot\alpha, \dot\delta)}
    ///   {\partial(x_0, y_0, z_0, \dot{x}_0, \dot{y}_0, \dot{z}_0)}\]
    ///
    /// Composed from the state transition matrix and the Cartesian-to-spherical
    /// Jacobian. Needed for orbit determination and trail linking.
    pub observation_jacobian: Option<[[f64; 6]; 6]>,
    /// Second-order mean shift from STT covariance propagation:
    /// \(\delta\mu = \frac{1}{2} \mathrm{Tr}(T \cdot \Sigma_0)\).
    ///
    /// In heliocentric ecliptic Cartesian coordinates (AU, AU/day).
    /// Quantifies the expected displacement of the mean state due to
    /// nonlinear effects. When projected to the tangent plane and compared
    /// to the clustering radius, this measures whether the linear
    /// velocity-shift assumption is still valid.
    pub mean_shift: Option<[f64; 6]>,
}

/// Integrator selection within a [`PropagationProfile`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IntegratorProfile {
    /// The backend's tightest-accuracy integrator (empyrean: GR15;
    /// ASSIST: IAS15). Default — reproduces historical outputs.
    Accurate,
    /// A faster survey-grade integrator (empyrean: DOP853, ~1.4× faster
    /// at ~358 m vs ~35 m median Horizons error). Never an automatic
    /// downgrade: callers opt in explicitly. Backends without a fast
    /// integrator error loudly unless
    /// [`PropagationProfile::allow_accurate_substitute`] is set.
    Fast,
}

/// Per-call propagation profile: event detection, integrator choice, and
/// dense-step caching.
///
/// The default is the **Precision** profile, which reproduces the
/// pre-profile behavior bit-for-bit (events on, accurate integrator,
/// no dense caching). Hot survey paths that never read event output
/// opt in to [`PropagationProfile::survey`] explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PropagationProfile {
    /// Run the backend's event detectors (close approaches, impacts,
    /// atmospheric entry, possible impacts, shadow events). THOR reads
    /// none of their output on any path today, but the default stays ON
    /// so default behavior is unchanged; survey paths turn them off.
    /// Documented no-op on ASSIST (it has no event machinery).
    pub events: bool,
    /// Integrator selection. See [`IntegratorProfile`].
    pub integrator: IntegratorProfile,
    /// When `integrator` is [`IntegratorProfile::Fast`] and the backend
    /// has no fast integrator (ASSIST), substitute the accurate one
    /// instead of erroring. The substitution is logged so the run
    /// manifest reflects what actually integrated. Default `false`.
    pub allow_accurate_substitute: bool,
    /// Cache per-step integrator coefficients for fast interpolation
    /// (empyrean `cache_integrator_steps`). Documented no-op on ASSIST.
    pub cache_dense: bool,
}

impl Default for PropagationProfile {
    fn default() -> Self {
        // Precision: bit-for-bit today's outputs.
        Self {
            events: true,
            integrator: IntegratorProfile::Accurate,
            allow_accurate_substitute: false,
            cache_dense: false,
        }
    }
}

impl PropagationProfile {
    /// Survey profile: event detection off (THOR never reads event
    /// output), accurate integrator, no dense caching. Trajectory output
    /// is identical to Precision — only the dead event-detection work is
    /// dropped. DOP853 remains a separate explicit opt-in on top.
    pub fn survey() -> Self {
        Self {
            events: false,
            ..Self::default()
        }
    }
}

/// Configuration for orbit propagation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PropagationConfig {
    /// Whether to compute the state transition matrix \(\Phi(t, t_0)\).
    pub compute_stm: bool,
    /// Propagation profile (events / integrator / caching). Defaults to
    /// the Precision profile, which reproduces historical behavior.
    #[serde(default)]
    pub profile: PropagationProfile,
}

/// Configuration for ephemeris generation.
///
/// # Covariance contract
///
/// `compute_covariance` is a real contract, honored by every backend:
///
/// - `true` + input orbit carries a covariance → the backend MUST
///   populate both `Ephemeris::aberrated_state.covariance` (Cartesian,
///   heliocentric ecliptic, at the emission epoch) and
///   `Ephemeris::state.covariance` (spherical, raw
///   \((\rho, \lambda, \beta, \dot\rho, \dot\lambda, \dot\beta)\), deg²
///   angular blocks), or error loudly.
/// - `true` + no input covariance →
///   [`crate::PropagatorError::MissingCovariance`].
///   Backends never fabricate a covariance.
/// - `false` → no covariance propagation, even when the orbit carries
///   one; both output covariance slots are `None`. This is the cheap
///   hot path (thor_dc, phase-2 light-time calls).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EphemerisConfig {
    /// Whether to propagate the input covariance to the output ephemeris
    /// (Cartesian at emission epoch + on-sky spherical). See the
    /// covariance contract in the struct docs.
    pub compute_covariance: bool,
    /// Method for covariance computation. Honored by the backends:
    /// requesting a method a backend cannot provide is a loud
    /// [`crate::PropagatorError::Unsupported`],
    /// never a silent substitution. See [`CovarianceMethod`] for the
    /// per-backend capability statements.
    pub covariance_method: CovarianceMethod,
    /// Number of samples for Monte Carlo or sigma-point methods.
    /// MonteCarlo: total sample count. SigmaPoint: total budget spread
    /// over the 15 coordinate planes of the 6-D state
    /// (`samples_per_plane = max(1, num_samples / 15)`). Ignored by
    /// Auto/Analytic.
    pub num_samples: usize,
    /// Populate `Ephemeris::observation_jacobian` analytically from the
    /// backend's STM and the Cartesian-to-spherical Jacobian. Skips the
    /// covariance pipeline so callers that only need the Jacobian (e.g.
    /// thor_dc LM step) avoid the extra cost.
    #[serde(default)]
    pub compute_jacobian: bool,
}

impl Default for EphemerisConfig {
    fn default() -> Self {
        Self {
            compute_covariance: false,
            covariance_method: CovarianceMethod::Auto,
            num_samples: 1000,
            compute_jacobian: false,
        }
    }
}

/// Method for propagating covariance through the ephemeris computation.
///
/// Per-backend capability statement:
/// - **empyrean**: `Auto` (adaptive escalation with engine-default
///   thresholds — load-bearing for near-Earth covariance inflation),
///   `Analytic` (first-order STM), and `SigmaPoint` (0.8.1 surfaces the
///   sigma-point sample covariance through the tagged readback).
///   `MonteCarlo` remains
///   [`crate::PropagatorError::Unsupported`]:
///   THOR has no seed policy for a reproducible MC covariance and no
///   consumer that wants one (see `map_covariance_method` in the adapter).
/// - **ASSIST**: `Analytic` natively (finite-difference STM,
///   \(\Phi \Sigma_0 \Phi^T\)); `Auto` resolves to the same first-order
///   machinery — that is ASSIST's best method, stated here explicitly,
///   not a silent downgrade; `SigmaPoint`/`MonteCarlo` are
///   [`crate::PropagatorError::Unsupported`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CovarianceMethod {
    /// Let the propagator choose the best method it has.
    Auto,
    /// First-order analytic propagation (STM, \(\Phi \Sigma_0 \Phi^T\)).
    Analytic,
    /// Monte Carlo sampling of perturbed initial conditions.
    MonteCarlo,
    /// Deterministic sigma-point sampling.
    SigmaPoint,
}
