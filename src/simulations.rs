//! Evaluate the paper's conditional retry mechanism without labelling it empirical evidence.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryScenario {
    pub human_production_hours: f64,
    pub agent_elapsed_hours: f64,
    pub verification_hours: f64,
    pub pass_probability: f64,
    pub retained_fraction: f64,
    pub reference_score: f64,
    pub initial_wait_hours: f64,
    pub attempt_cap: Option<u32>,
    pub defect_probability: Option<f64>,
    pub detection_sensitivity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryResult {
    pub evidence_class: String,
    pub human_hours_per_attempt: f64,
    pub elapsed_hours_per_attempt: f64,
    pub expected_attempts: f64,
    pub acceptance_probability: f64,
    pub expected_human_hours: f64,
    pub expected_elapsed_hours: f64,
    pub expected_retained_output: f64,
    pub output_per_human_hour: Option<f64>,
    pub gate_pass_probability: Option<f64>,
    pub correctness_given_gate_pass: Option<f64>,
    pub assumptions: Vec<String>,
}

pub fn retry(s: &RetryScenario) -> Result<RetryResult> {
    for value in [
        s.human_production_hours,
        s.agent_elapsed_hours,
        s.verification_hours,
        s.reference_score,
        s.initial_wait_hours,
    ] {
        ensure!(
            value.is_finite() && value >= 0.0,
            "resource and score values must be finite and nonnegative"
        );
    }
    for value in [s.pass_probability, s.retained_fraction] {
        ensure!(
            value.is_finite() && (0.0..=1.0).contains(&value),
            "probability/fraction outside [0,1]"
        );
    }
    ensure!(
        s.attempt_cap.is_some() || s.pass_probability > 0.0,
        "eventual acceptance requires positive pass probability"
    );
    let p = s.pass_probability;
    let (attempts, acceptance) = if let Some(cap) = s.attempt_cap {
        if p == 0.0 {
            (f64::from(cap), 0.0)
        } else {
            let accepted = if p == 1.0 {
                if cap == 0 { 0.0 } else { 1.0 }
            } else {
                -((f64::from(cap)) * (-p).ln_1p()).exp_m1()
            };
            (accepted / p, accepted)
        }
    } else {
        (1.0 / p, 1.0)
    };
    let h = s.human_production_hours + s.verification_hours;
    let t = s.human_production_hours.max(s.agent_elapsed_hours) + s.verification_hours;
    let human = h * attempts;
    let elapsed = if attempts == 0.0 {
        0.0
    } else {
        s.initial_wait_hours + t * attempts
    };
    let output = s.reference_score * s.retained_fraction * acceptance;
    let (gate, correctness) = match (s.defect_probability, s.detection_sensitivity) {
        (None, None) => (None, None),
        (Some(d), Some(sensitivity)) => {
            ensure!(
                d.is_finite() && (0.0..1.0).contains(&d),
                "defect probability must be in [0,1)"
            );
            ensure!(
                sensitivity.is_finite() && (0.0..=1.0).contains(&sensitivity),
                "sensitivity must be in [0,1]"
            );
            let pass = 1.0 - d * sensitivity;
            (Some(pass), Some((1.0 - d) / pass))
        }
        _ => anyhow::bail!("supply both defect probability and detection sensitivity"),
    };
    ensure!(
        [attempts, acceptance, h, t, human, elapsed, output]
            .iter()
            .all(|v| v.is_finite()),
        "scenario arithmetic overflow"
    );
    Ok(RetryResult {evidence_class:"synthetic_conditional_model".into(),human_hours_per_attempt:h,elapsed_hours_per_attempt:t,expected_attempts:attempts,acceptance_probability:acceptance,expected_human_hours:human,expected_elapsed_hours:elapsed,expected_retained_output:output,output_per_human_hour:if human>0.0 {Some(output/human)}else{None},gate_pass_probability:gate,correctness_given_gate_pass:correctness,assumptions:vec!["Independent identically distributed attempts; fixed attempt labour and duration".into(),"Human and agent production branches run independently in parallel before verification".into(),"No repair or shared queue costs; retained fraction is conditional on acceptance within the chosen cap".into(),"This mechanism is separate from finite-horizon cohort accounting and is not an estimated AI effect".into()]})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scenario() -> RetryScenario {
        RetryScenario {
            human_production_hours: 2.0,
            agent_elapsed_hours: 8.0,
            verification_hours: 1.5,
            pass_probability: 0.9,
            retained_fraction: 0.98,
            reference_score: 10.0,
            initial_wait_hours: 2.0,
            attempt_cap: None,
            defect_probability: None,
            detection_sensitivity: None,
        }
    }
    #[test]
    fn paper_slow_agent_example() {
        let r = retry(&scenario()).unwrap();
        assert!((r.expected_human_hours - 3.5 / 0.9).abs() < 1e-12);
        assert!((r.expected_elapsed_hours - (2.0 + 9.5 / 0.9)).abs() < 1e-12);
    }
    #[test]
    fn zero_success_cap_keeps_unsuccessful_effort() {
        let mut s = scenario();
        s.pass_probability = 0.0;
        s.attempt_cap = Some(3);
        let r = retry(&s).unwrap();
        assert_eq!(r.acceptance_probability, 0.0);
        assert_eq!(r.expected_human_hours, 10.5);
        assert_eq!(r.expected_retained_output, 0.0);
    }
    #[test]
    fn zero_attempts_has_no_ratio() {
        let mut s = scenario();
        s.attempt_cap = Some(0);
        let r = retry(&s).unwrap();
        assert_eq!(r.expected_elapsed_hours, 0.0);
        assert_eq!(r.output_per_human_hour, None);
    }
}
