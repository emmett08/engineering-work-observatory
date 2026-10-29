use observatory::inference::{
    LabourEstimateInput, LabourFrameUnit, LabourObservation, PredictionProvenance, SamplingDesign,
    VarianceStatus, estimate_labour,
};
use serde_json::json;

fn srs(outcomes: &[f64], predictions: &[f64], selected: &[usize]) -> LabourEstimateInput {
    LabourEstimateInput {
        schema: "observatory.labour-estimate-input/1".into(),
        frame_id: "synthetic-finite-population".into(),
        frame_complete: true,
        predictions: PredictionProvenance {
            frozen_before_sample: true,
            provenance: "Synthetic fixed vector; enumerated before sample selection".into(),
        },
        design: SamplingDesign::SimpleRandomWithoutReplacement {
            sample_size: selected.len(),
        },
        units: outcomes
            .iter()
            .zip(predictions)
            .enumerate()
            .map(
                |(i, (&person_hours, &predicted_person_hours))| LabourFrameUnit {
                    id: format!("unit-{i}"),
                    predicted_person_hours,
                    inclusion_probability: selected.len() as f64 / outcomes.len() as f64,
                    observation: if selected.contains(&i) {
                        LabourObservation::Observed { person_hours }
                    } else {
                        LabourObservation::Unselected {}
                    },
                },
            )
            .collect(),
    }
}

#[test]
fn enumerate_all_srs_samples_unbiasedness_and_variance_benefit_and_harm() {
    // Exhaust all C(4,2)=6 equiprobable samples. All totals and squared deviations
    // below are exactly representable integers; no Monte Carlo coverage claim.
    let outcomes = [0.0, 2.0, 4.0, 6.0];
    let useful = [1.0, 3.0, 3.0, 5.0];
    let harmful = [6.0, 4.0, 2.0, 0.0];
    let samples = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];
    let mut ht_total = 0.0;
    let mut useful_total = 0.0;
    let mut harmful_total = 0.0;
    let mut ht_squared_error = 0.0;
    let mut useful_squared_error = 0.0;
    let mut harmful_squared_error = 0.0;
    let mut ht_variance_sum = 0.0;
    let mut useful_variance_sum = 0.0;
    let mut harmful_variance_sum = 0.0;
    for sample in samples {
        let good = estimate_labour(&srs(&outcomes, &useful, &sample)).unwrap();
        let bad = estimate_labour(&srs(&outcomes, &harmful, &sample)).unwrap();
        assert_eq!(good.variance.status, VarianceStatus::EstimatedSrs);
        ht_total += good.ht_total_person_hours;
        useful_total += good.difference_total_person_hours;
        harmful_total += bad.difference_total_person_hours;
        ht_squared_error += (good.ht_total_person_hours - 12.0).powi(2);
        useful_squared_error += (good.difference_total_person_hours - 12.0).powi(2);
        harmful_squared_error += (bad.difference_total_person_hours - 12.0).powi(2);
        ht_variance_sum += good.variance.ht_variance_person_hours_squared.unwrap();
        useful_variance_sum += good
            .variance
            .difference_variance_person_hours_squared
            .unwrap();
        harmful_variance_sum += bad
            .variance
            .difference_variance_person_hours_squared
            .unwrap();
    }
    // Exact finite enumeration: all three means are 12, and the unbiased SRS
    // variance estimates average to the corresponding repeated-sampling variance.
    assert_eq!([ht_total, useful_total, harmful_total], [72.0; 3]);
    assert_eq!(ht_squared_error, 160.0);
    assert_eq!(useful_squared_error, 32.0);
    assert_eq!(harmful_squared_error, 640.0);
    assert_eq!(ht_variance_sum, ht_squared_error);
    assert_eq!(useful_variance_sum, useful_squared_error);
    assert_eq!(harmful_variance_sum, harmful_squared_error);
    assert_eq!(5.0 * useful_squared_error, ht_squared_error);
    assert_eq!(harmful_squared_error, 4.0 * ht_squared_error);
}

#[test]
fn enumerate_an_unequal_probability_general_design() {
    // A known design selects {0}, {1}, {0,2} with probabilities 1/2,1/4,1/4.
    // Unequal first-order probabilities are 3/4,1/4,1/4; dependence is unrestricted.
    let samples = [(vec![0], 0.5), (vec![1], 0.25), (vec![0, 2], 0.25)];
    let mut expected_ht = 0.0;
    let mut expected_difference = 0.0;
    for (selected, probability) in samples {
        let mut frame = srs(&[3.0, 0.0, 9.0], &[0.0, 3.0, 6.0], &selected);
        frame.design = SamplingDesign::General {};
        for (unit, inclusion_probability) in frame.units.iter_mut().zip([0.75, 0.25, 0.25]) {
            unit.inclusion_probability = inclusion_probability;
        }
        let report = estimate_labour(&frame).unwrap();
        expected_ht += probability * report.ht_total_person_hours;
        expected_difference += probability * report.difference_total_person_hours;
        assert_eq!(report.variance.status, VarianceStatus::Unavailable);
        assert!(
            report
                .variance
                .difference_variance_person_hours_squared
                .is_none()
        );
    }
    assert_eq!(expected_ht, 12.0);
    assert_eq!(expected_difference, 12.0);
}

#[test]
fn negative_difference_estimates_are_reported_without_clipping() {
    let report = estimate_labour(&srs(&[0.0, 0.0], &[10.0, 0.0], &[0])).unwrap();
    assert_eq!(report.prediction_total_person_hours, 10.0);
    assert_eq!(report.ht_total_person_hours, 0.0);
    assert_eq!(report.difference_total_person_hours, -10.0);
    assert_eq!(report.variance.status, VarianceStatus::Unavailable);
    assert!(
        report
            .variance
            .difference_standard_error_person_hours
            .is_none()
    );
    // Clipping the two possible estimates (-10, 10) yields a biased mean of 5.
    let other = estimate_labour(&srs(&[0.0, 0.0], &[10.0, 0.0], &[1])).unwrap();
    assert_eq!(other.difference_total_person_hours, 10.0);
    assert_eq!(
        (report.difference_total_person_hours.max(0.0)
            + other.difference_total_person_hours.max(0.0))
            / 2.0,
        5.0
    );
}

#[test]
fn signed_frozen_predictions_are_valid_auxiliary_values() {
    let report = estimate_labour(&srs(&[2.0, 4.0], &[-1.0, -2.0], &[0, 1])).unwrap();
    assert_eq!(report.prediction_total_person_hours, -3.0);
    assert_eq!(report.difference_total_person_hours, 6.0);
}

#[test]
fn large_common_prediction_offsets_preserve_small_labels_and_residual_variance() {
    let census = estimate_labour(&srs(&[1.0, 1.0], &[1e16, 1e16], &[0, 1])).unwrap();
    assert_eq!(census.difference_total_person_hours, 2.0);
    let unequal_census = estimate_labour(&srs(&[1.0, 0.0], &[0.0, 1e16], &[0, 1])).unwrap();
    assert_eq!(unequal_census.difference_total_person_hours, 1.0);
    let sample = estimate_labour(&srs(&[0.0, 2.0, 4.0, 6.0], &[1e16; 4], &[0, 1])).unwrap();
    assert_eq!(sample.difference_total_person_hours, 4.0);
    assert_eq!(
        sample.variance.difference_variance_person_hours_squared,
        Some(8.0)
    );
    assert_eq!(sample.variance.ht_variance_person_hours_squared, Some(8.0));
}

#[test]
fn srs_rounding_does_not_turn_constant_predictions_into_nonzero_corrections() {
    // 3/7 and its reciprocal round differently. Cancel the known SRS offset
    // algebraically instead of subtracting a rounded weighted 7e16 total.
    for offset in [0.0, 1e16, -1e16] {
        let report = estimate_labour(&srs(&[0.0; 7], &[offset; 7], &[0, 1, 2])).unwrap();
        assert_eq!(report.difference_total_person_hours, 0.0);
        assert_eq!(
            report.variance.difference_variance_person_hours_squared,
            Some(0.0)
        );
    }
}

#[test]
fn every_srs_sample_preserves_predictions_with_representable_common_offsets() {
    // Exhaust all C(7,3)=35 samples and compare against an integer numerator
    // oracle: 3*sum(f) + 7*sum_s(y-f), divided by 3 only at the end.
    // Even offsets are exactly representable at magnitude 1e16; sub-ULP input
    // differences cannot be recovered and are deliberately not assumed here.
    let outcomes = [1.0, 4.0, 5.0, 9.0, 9.0, 10.0, 13.0];
    let variations = [0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0];
    for a in 0..5 {
        for b in a + 1..6 {
            for c in b + 1..7 {
                let selected = [a, b, c];
                let expected_numerator = 3 * 42
                    + 7 * selected
                        .iter()
                        .map(|&i| outcomes[i] as i64 - variations[i] as i64)
                        .sum::<i64>();
                let baseline = estimate_labour(&srs(&outcomes, &variations, &selected)).unwrap();
                for offset in [0.0, 1e16, -1e16] {
                    let predictions = variations.map(|value| offset + value);
                    let report = estimate_labour(&srs(&outcomes, &predictions, &selected)).unwrap();
                    assert!(
                        (report.difference_total_person_hours - expected_numerator as f64 / 3.0)
                            .abs()
                            < 1e-12
                    );
                    assert_eq!(
                        report.difference_total_person_hours,
                        baseline.difference_total_person_hours
                    );
                    assert_eq!(report.ht_total_person_hours, baseline.ht_total_person_hours);
                    assert_eq!(
                        report.variance.difference_variance_person_hours_squared,
                        baseline.variance.difference_variance_person_hours_squared
                    );
                }
            }
        }
    }
}

#[test]
fn sample_fitting_counterexample_defeats_fixed_prediction_guarantee() {
    // For N=2,n=1,y=(0,2), memorise the sampled label and predict zero elsewhere.
    // Each sample residual is zero, so the difference estimate is the fitted
    // vector's total: 0 or 2, with mean 1 instead of population total 2.
    let mut estimates = Vec::new();
    for selected in 0..2 {
        let outcomes = [0.0, 2.0];
        let mut predictions = [0.0; 2];
        predictions[selected] = outcomes[selected];
        let mut frame = srs(&outcomes, &predictions, &[selected]);
        frame.predictions.frozen_before_sample = false;
        assert!(
            estimate_labour(&frame)
                .unwrap_err()
                .to_string()
                .contains("frozen")
        );
        estimates.push(predictions.iter().sum::<f64>());
    }
    assert_eq!(estimates, [0.0, 2.0]);
    assert_eq!(estimates.iter().sum::<f64>() / 2.0, 1.0);
}

#[test]
fn census_variance_is_zero_including_a_single_unit() {
    for (y, f, selected, total) in [
        (vec![0.0], vec![10.0], vec![0], 0.0),
        (vec![0.0, 4.0], vec![10.0, -3.0], vec![0, 1], 4.0),
    ] {
        let report = estimate_labour(&srs(&y, &f, &selected)).unwrap();
        assert_eq!(report.difference_total_person_hours, total);
        assert_eq!(report.variance.status, VarianceStatus::KnownZeroCensus);
        assert_eq!(
            report.variance.difference_variance_person_hours_squared,
            Some(0.0)
        );
        assert_eq!(report.variance.ht_variance_person_hours_squared, Some(0.0));
    }
}

#[test]
fn zero_observation_unselected_and_nonresponse_are_distinct() {
    let mut frame = srs(&[0.0, 4.0], &[1.0, 1.0], &[0]);
    frame.design = SamplingDesign::General {};
    let observed_zero = estimate_labour(&frame).unwrap();
    assert_eq!(observed_zero.observed_sample_size, 1);
    assert_eq!(observed_zero.difference_total_person_hours, 0.0);
    frame.units[0].observation = LabourObservation::Unselected {};
    let empty_sample = estimate_labour(&frame).unwrap();
    assert_eq!(empty_sample.observed_sample_size, 0);
    assert_eq!(empty_sample.difference_total_person_hours, 2.0);
    frame.units[0].observation = LabourObservation::Nonresponse {
        reason: "Selected participant did not answer".into(),
    };
    assert!(
        estimate_labour(&frame)
            .unwrap_err()
            .to_string()
            .contains("nonresponse")
    );
}

#[test]
fn incomplete_frame_duplicate_identity_and_missing_provenance_reject() {
    let base = srs(&[0.0, 4.0], &[1.0, 1.0], &[0]);
    let mut bad = base.clone();
    bad.frame_complete = false;
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("complete")
    );
    bad = base.clone();
    bad.units[1].id = bad.units[0].id.clone();
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("duplicate")
    );
    bad = base.clone();
    bad.predictions.provenance = " ".into();
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("provenance")
    );
    bad = base.clone();
    bad.units.clear();
    assert!(estimate_labour(&bad).is_err());
    bad = base;
    bad.schema = "observatory.labour-estimate-input/2".into();
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("schema")
    );
}

#[test]
fn srs_requires_the_declared_selected_count_and_probabilities() {
    let base = srs(&[0.0, 2.0, 4.0, 6.0], &[1.0; 4], &[0, 1]);
    let mut bad = base.clone();
    bad.units[1].observation = LabourObservation::Unselected {};
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("observed")
    );
    bad = base.clone();
    bad.units[2].inclusion_probability = 0.4;
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("sample_size/frame size")
    );
    for sample_size in [0, 5] {
        bad = base.clone();
        bad.design = SamplingDesign::SimpleRandomWithoutReplacement { sample_size };
        assert!(estimate_labour(&bad).is_err());
    }
    bad = base;
    bad.design = SamplingDesign::General {};
    bad.units[2].inclusion_probability = 1.0;
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("certainty")
    );
}

#[test]
fn invalid_probabilities_nonfinite_values_and_overflow_reject() {
    let base = srs(&[0.0, 4.0], &[1.0, 1.0], &[0]);
    for pi in [0.0, -0.5, 1.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut bad = base.clone();
        bad.units[0].inclusion_probability = pi;
        assert!(estimate_labour(&bad).is_err(), "probability {pi}");
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut bad = base.clone();
        bad.units[0].predicted_person_hours = value;
        assert!(estimate_labour(&bad).is_err());
        bad = base.clone();
        bad.units[0].observation = LabourObservation::Observed {
            person_hours: value,
        };
        assert!(estimate_labour(&bad).is_err());
    }
    let mut bad = base.clone();
    bad.units[0].observation = LabourObservation::Observed { person_hours: -1.0 };
    assert!(estimate_labour(&bad).is_err());
    bad = base;
    bad.units[0].predicted_person_hours = f64::MAX;
    bad.units[1].predicted_person_hours = f64::MAX;
    assert!(
        estimate_labour(&bad)
            .unwrap_err()
            .to_string()
            .contains("overflow")
    );
}

#[test]
fn serde_rejects_ambiguous_observation_and_unknown_fields() {
    let base = serde_json::to_value(srs(&[0.0, 4.0], &[1.0, 1.0], &[0])).unwrap();
    let mut bad = base.clone();
    bad["units"][0]["observation"] = json!({"status":"observed"});
    assert!(serde_json::from_value::<LabourEstimateInput>(bad).is_err());
    let mut bad = base.clone();
    bad["units"][0]["observation"] = json!({"status":"unselected","person_hours":0});
    assert!(serde_json::from_value::<LabourEstimateInput>(bad).is_err());
    let mut bad = base.clone();
    bad["design"] = json!({"kind":"general","sample_size":1});
    assert!(serde_json::from_value::<LabourEstimateInput>(bad).is_err());
    let mut bad = base.clone();
    bad["predictions"]["fitted_on_sample"] = json!(true);
    assert!(serde_json::from_value::<LabourEstimateInput>(bad).is_err());
    let mut bad = base;
    bad["frame_complet"] = json!(true);
    assert!(serde_json::from_value::<LabourEstimateInput>(bad).is_err());
}
