//! The two properties a contract crate must never break silently: the serde
//! shapes of the configuration/state types (serialization IS the contract),
//! and the trait's object safety with its defaulted methods.

use thor_rs_propagator::{
    CartesianState, Ephemeris, EphemerisConfig, Frame, ObserverState, Origin, PropagatedState,
    PropagationConfig, Propagator, PropagatorError, TestOrbit,
};

fn state() -> CartesianState {
    CartesianState {
        x: 2.3,
        y: 1.0,
        z: 0.1,
        vx: -0.005,
        vy: 0.009,
        vz: 0.0001,
        epoch: 60800.0,
        frame: Frame::EclipticJ2000,
        origin: Origin::Sun,
        covariance: None,
    }
}

#[test]
fn config_and_state_types_round_trip_through_serde() {
    let orbit = TestOrbit {
        id: "t".into(),
        object_id: Some("o".into()),
        bundle_id: None,
        nside: 32,
        state: state(),
    };
    let back: TestOrbit = serde_json::from_str(&serde_json::to_string(&orbit).unwrap()).unwrap();
    assert_eq!(back.id, "t");
    assert_eq!(back.state.epoch, 60800.0);
    let cfg: PropagationConfig =
        serde_json::from_str(&serde_json::to_string(&PropagationConfig::default()).unwrap())
            .unwrap();
    assert_eq!(cfg.compute_stm, PropagationConfig::default().compute_stm);
    let ecfg: EphemerisConfig =
        serde_json::from_str(&serde_json::to_string(&EphemerisConfig::default()).unwrap()).unwrap();
    assert_eq!(
        ecfg.compute_covariance,
        EphemerisConfig::default().compute_covariance
    );
}

/// A minimal impl proving the trait is object-safe and that the defaulted
/// methods behave as documented: force_model() = "unknown", body positions =
/// a loud typed error, never a silent answer.
struct Inert;

impl Propagator for Inert {
    fn propagate(
        &self,
        _orbit: &TestOrbit,
        epochs: &[f64],
        _config: &PropagationConfig,
    ) -> Result<Vec<PropagatedState>, PropagatorError> {
        Err(PropagatorError::Other(format!(
            "inert: {} epochs",
            epochs.len()
        )))
    }
    fn compute_observers(
        &self,
        _codes: &[String],
        _epochs: &[f64],
    ) -> Result<Vec<ObserverState>, PropagatorError> {
        Err(PropagatorError::Other("inert".into()))
    }
    fn transform_state(
        &self,
        _state: &CartesianState,
        _target_frame: Frame,
        _target_origin: Origin,
    ) -> Result<CartesianState, PropagatorError> {
        Err(PropagatorError::Other("inert".into()))
    }
    fn generate_ephemeris(
        &self,
        _orbit: &TestOrbit,
        _observers: &[ObserverState],
        _config: &EphemerisConfig,
    ) -> Result<Vec<Ephemeris>, PropagatorError> {
        Err(PropagatorError::Other("inert".into()))
    }
}

#[test]
fn trait_is_object_safe_with_documented_defaults() {
    let p: &dyn Propagator = &Inert;
    assert_eq!(p.force_model(), "unknown");
    assert!(p.compute_body_positions("earth", &[60800.0]).is_err());
}
