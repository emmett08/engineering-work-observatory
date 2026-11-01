import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Sum
import Mathlib.MeasureTheory.Measure.Typeclasses.Probability

/-!
Finite real linear-programme certificates and simultaneous interval coverage.
These results check given certificates; they do not assert strong-dual existence
or correctness of an external optimisation implementation.
-/

open scoped BigOperators

namespace LabourBounds.Certificates

variable {V C O Ω : Type*} [Fintype V] [Fintype C]

/-- Weighted target, with arbitrary real weights. -/
def objective (w x : V → ℝ) : ℝ := ∑ j, w j * x j

/-- Row action of a finite real matrix. -/
def matrixAction (A : C → V → ℝ) (x : V → ℝ) (i : C) : ℝ :=
  ∑ j, A i j * x j

/-- Transposed matrix action on constraint multipliers. -/
def transposeAction (A : C → V → ℝ) (lam : C → ℝ) (j : V) : ℝ :=
  ∑ i, lam i * A i j

/-- Nonnegative coordinates satisfying every row inequality. -/
def primalFeasible (A : C → V → ℝ) (b : C → ℝ) (x : V → ℝ) : Prop :=
  (∀ j, 0 ≤ x j) ∧ ∀ i, matrixAction A x i ≤ b i

def upperCertificate (A : C → V → ℝ) (w : V → ℝ) (lam : C → ℝ) : Prop :=
  (∀ i, 0 ≤ lam i) ∧ ∀ j, w j ≤ transposeAction A lam j

def lowerCertificate (A : C → V → ℝ) (w : V → ℝ) (mu : C → ℝ) : Prop :=
  upperCertificate A (fun j => -w j) mu

theorem transpose_objective (A : C → V → ℝ) (lam : C → ℝ) (x : V → ℝ) :
    objective (transposeAction A lam) x = objective lam (matrixAction A x) := by
  unfold objective transposeAction matrixAction
  simp only [Finset.sum_mul, Finset.mul_sum]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro i _
  apply Finset.sum_congr rfl
  intro j _
  exact mul_assoc (lam i) (A i j) (x j)

/-- Weak duality for a supplied upper certificate (paper Eq. bound-certificates). -/
theorem weak_dual_upper {A : C → V → ℝ} {b lam : C → ℝ} {w x : V → ℝ}
    (hx : primalFeasible A b x) (hlam : upperCertificate A w lam) :
    objective w x ≤ objective lam b := by
  calc
    objective w x ≤ objective (transposeAction A lam) x :=
      Finset.sum_le_sum fun j _ => mul_le_mul_of_nonneg_right (hlam.2 j) (hx.1 j)
    _ = objective lam (matrixAction A x) := transpose_objective A lam x
    _ ≤ objective lam b :=
      Finset.sum_le_sum fun i _ => mul_le_mul_of_nonneg_left (hx.2 i) (hlam.1 i)

theorem objective_neg (w x : V → ℝ) :
    objective (fun j => -w j) x = -objective w x := by
  simp [objective, Finset.sum_neg_distrib]

/-- Weak duality for the negated objective. -/
theorem weak_dual_lower {A : C → V → ℝ} {b mu : C → ℝ} {w x : V → ℝ}
    (hx : primalFeasible A b x) (hmu : lowerCertificate A w mu) :
    -objective mu b ≤ objective w x := by
  have h := weak_dual_upper hx hmu
  rw [objective_neg] at h
  simpa only [neg_neg] using neg_le_neg h

theorem weak_dual_bounds {A : C → V → ℝ} {b lam mu : C → ℝ} {w x : V → ℝ}
    (hx : primalFeasible A b x) (hlam : upperCertificate A w lam)
    (hmu : lowerCertificate A w mu) :
    -objective mu b ≤ objective w x ∧ objective w x ≤ objective lam b :=
  ⟨weak_dual_lower hx hmu, weak_dual_upper hx hlam⟩

/-- Matching feasible primal and upper-dual values certify an attained maximum. -/
theorem equality_certifies_maximum {A : C → V → ℝ} {b lam : C → ℝ}
    {w xStar : V → ℝ} (hxStar : primalFeasible A b xStar)
    (hlam : upperCertificate A w lam)
    (heq : objective w xStar = objective lam b) :
    primalFeasible A b xStar ∧
      ∀ x, primalFeasible A b x → objective w x ≤ objective w xStar := by
  refine ⟨hxStar, fun x hx => ?_⟩
  rw [heq]
  exact weak_dual_upper hx hlam

/-- Matching feasible primal and lower-dual values certify an attained minimum. -/
theorem equality_certifies_minimum {A : C → V → ℝ} {b mu : C → ℝ}
    {w xStar : V → ℝ} (hxStar : primalFeasible A b xStar)
    (hmu : lowerCertificate A w mu)
    (heq : objective w xStar = -objective mu b) :
    primalFeasible A b xStar ∧
      ∀ x, primalFeasible A b x → objective w xStar ≤ objective w x := by
  refine ⟨hxStar, fun x hx => ?_⟩
  rw [heq]
  exact weak_dual_lower hx hmu

/-- The matching upper certificate identifies the greatest attainable value. -/
theorem equality_certifies_isGreatest {A : C → V → ℝ} {b lam : C → ℝ}
    {w xStar : V → ℝ} (hxStar : primalFeasible A b xStar)
    (hlam : upperCertificate A w lam)
    (heq : objective w xStar = objective lam b) :
    IsGreatest (objective w '' {x | primalFeasible A b x}) (objective w xStar) := by
  refine ⟨⟨xStar, hxStar, rfl⟩, ?_⟩
  intro y hy
  rcases hy with ⟨x, hx, rfl⟩
  exact (equality_certifies_maximum hxStar hlam heq).2 x hx

/-- The matching lower certificate identifies the least attainable value. -/
theorem equality_certifies_isLeast {A : C → V → ℝ} {b mu : C → ℝ}
    {w xStar : V → ℝ} (hxStar : primalFeasible A b xStar)
    (hmu : lowerCertificate A w mu)
    (heq : objective w xStar = -objective mu b) :
    IsLeast (objective w '' {x | primalFeasible A b x}) (objective w xStar) := by
  refine ⟨⟨xStar, hxStar, rfl⟩, ?_⟩
  intro y hy
  rcases hy with ⟨x, hx, rfl⟩
  exact (equality_certifies_minimum hxStar hmu heq).2 x hx

/-- The same certificates apply to every point satisfying the relaxed RHS.
Nonnegativity of the perturbation is unnecessary for certificate validity. -/
theorem relaxed_bounds {A : C → V → ℝ} {b eps lam mu : C → ℝ} {w x : V → ℝ}
    (hx : primalFeasible A (fun i => b i + eps i) x)
    (hlam : upperCertificate A w lam) (hmu : lowerCertificate A w mu) :
    -objective mu (fun i => b i + eps i) ≤ objective w x ∧
      objective w x ≤ objective lam (fun i => b i + eps i) :=
  weak_dual_bounds hx hlam hmu

omit [Fintype C] in
/-- A nonnegative RHS change retains every previously feasible vector. -/
theorem feasible_after_relaxation {A : C → V → ℝ} {b eps : C → ℝ} {x : V → ℝ}
    (hx : primalFeasible A b x) (heps : ∀ i, 0 ≤ eps i) :
    primalFeasible A (fun i => b i + eps i) x :=
  ⟨hx.1, fun i => (hx.2 i).trans (le_add_of_nonneg_right (heps i))⟩

theorem relaxed_upper_ge {b eps lam : C → ℝ}
    (hlam : ∀ i, 0 ≤ lam i) (heps : ∀ i, 0 ≤ eps i) :
    objective lam b ≤ objective lam (fun i => b i + eps i) := by
  apply Finset.sum_le_sum
  intro i _
  exact mul_le_mul_of_nonneg_left (le_add_of_nonneg_right (heps i)) (hlam i)

theorem relaxed_lower_le {b eps mu : C → ℝ}
    (hmu : ∀ i, 0 ≤ mu i) (heps : ∀ i, 0 ≤ eps i) :
    -objective mu (fun i => b i + eps i) ≤ -objective mu b :=
  neg_le_neg (relaxed_upper_ge hmu heps)

section ObservationCoverage

variable [Fintype O]

/-- Every random observation interval contains its fixed true linear total. -/
def simultaneousObservationEvent (B : O → V → ℝ) (xTrue : V → ℝ)
    (lower upper : Ω → O → ℝ) : Set Ω :=
  {ω | ∀ o, lower ω o ≤ matrixAction B xTrue o ∧
    matrixAction B xTrue o ≤ upper ω o}

/-- Deterministic capacity rows and the two signed rows for each observation. -/
def intervalMatrix (A : C → V → ℝ) (B : O → V → ℝ) :
    (C ⊕ (O ⊕ O)) → V → ℝ
  | .inl i => A i
  | .inr (.inl o) => B o
  | .inr (.inr o) => fun j => -B o j

def intervalRhs (b : C → ℝ) (lower upper : Ω → O → ℝ) (ω : Ω) :
    (C ⊕ (O ⊕ O)) → ℝ
  | .inl i => b i
  | .inr (.inl o) => upper ω o
  | .inr (.inr o) => -lower ω o

omit [Fintype C] [Fintype O] in
/-- The feasible set contains the fixed true vector on simultaneous coverage.
The lower observation inequalities are represented by signed matrix rows. -/
theorem true_vector_feasible_on_simultaneous_event
    {A : C → V → ℝ} {B : O → V → ℝ} {b : C → ℝ} {xTrue : V → ℝ}
    {lower upper : Ω → O → ℝ} (hTrue : primalFeasible A b xTrue)
    {ω : Ω} (hω : ω ∈ simultaneousObservationEvent B xTrue lower upper) :
    primalFeasible (intervalMatrix A B) (intervalRhs b lower upper ω) xTrue := by
  refine ⟨hTrue.1, ?_⟩
  intro i
  rcases i with i | (o | o)
  · exact hTrue.2 i
  · exact (hω o).2
  · change (∑ j, -B o j * xTrue j) ≤ -lower ω o
    simpa only [matrixAction, neg_mul, Finset.sum_neg_distrib] using neg_le_neg (hω o).1

/-- Coverage includes nonemptiness, so an empty feasible set fails to cover. -/
def targetCoverageEvent (A : C → V → ℝ) (b : Ω → C → ℝ)
    (w xTrue : V → ℝ) (lo hi : Ω → ℝ) : Set Ω :=
  {ω | (∃ x, primalFeasible A (b ω) x) ∧
    lo ω ≤ objective w xTrue ∧ objective w xTrue ≤ hi ω}

omit [Fintype C] [Fintype O] in
/-- Given valid optimisation enclosures, simultaneous observation coverage
implies target coverage. No solver correctness or attainment is assumed silently. -/
theorem simultaneous_event_subset_target
    {A : C → V → ℝ} {B : O → V → ℝ} {b : C → ℝ} {w xTrue : V → ℝ}
    {lower upper : Ω → O → ℝ} {lo hi : Ω → ℝ}
    (hTrue : primalFeasible A b xTrue)
    (hEnclose : ∀ ω x,
      primalFeasible (intervalMatrix A B) (intervalRhs b lower upper ω) x →
      lo ω ≤ objective w x ∧ objective w x ≤ hi ω) :
    simultaneousObservationEvent B xTrue lower upper ⊆
      targetCoverageEvent (intervalMatrix A B) (intervalRhs b lower upper)
        w xTrue lo hi := by
  intro ω hω
  have hx := true_vector_feasible_on_simultaneous_event hTrue hω
  exact ⟨⟨xTrue, hx⟩, hEnclose ω xTrue hx⟩

/-- Checked random dual certificates supply the enclosure hypothesis themselves. -/
theorem simultaneous_event_subset_certificate_target
    {A : C → V → ℝ} {B : O → V → ℝ} {b : C → ℝ} {w xTrue : V → ℝ}
    {lower upper : Ω → O → ℝ}
    {lam mu : Ω → (C ⊕ (O ⊕ O)) → ℝ}
    (hTrue : primalFeasible A b xTrue)
    (hlam : ∀ ω, upperCertificate (intervalMatrix A B) w (lam ω))
    (hmu : ∀ ω, lowerCertificate (intervalMatrix A B) w (mu ω)) :
    simultaneousObservationEvent B xTrue lower upper ⊆
      targetCoverageEvent (intervalMatrix A B) (intervalRhs b lower upper) w xTrue
        (fun ω => -objective (mu ω) (intervalRhs b lower upper ω))
        (fun ω => objective (lam ω) (intervalRhs b lower upper ω)) :=
  simultaneous_event_subset_target hTrue
    (fun ω _ hx => weak_dual_bounds hx (hlam ω) (hmu ω))

omit [Fintype C] [Fintype O] in
/-- Probability transfer derives event inclusion from the fixed true vector and
all random observation bounds; the lower bound is not assumed for the target. -/
theorem simultaneous_coverage_probability
    [MeasurableSpace Ω] (P : MeasureTheory.Measure Ω)
    [MeasureTheory.IsProbabilityMeasure P]
    {A : C → V → ℝ} {B : O → V → ℝ} {b : C → ℝ} {w xTrue : V → ℝ}
    {lower upper : Ω → O → ℝ} {lo hi : Ω → ℝ} {alpha : ℝ}
    (hTrue : primalFeasible A b xTrue)
    (hEnclose : ∀ ω x,
      primalFeasible (intervalMatrix A B) (intervalRhs b lower upper ω) x →
      lo ω ≤ objective w x ∧ objective w x ≤ hi ω)
    (_hAlpha : 0 < alpha ∧ alpha < 1)
    (_hObservationMeasurable : MeasurableSet
      (simultaneousObservationEvent B xTrue lower upper))
    (_hTargetMeasurable : MeasurableSet
      (targetCoverageEvent (intervalMatrix A B) (intervalRhs b lower upper)
        w xTrue lo hi))
    (hCoverage : ENNReal.ofReal (1 - alpha) ≤
      P (simultaneousObservationEvent B xTrue lower upper)) :
    ENNReal.ofReal (1 - alpha) ≤
      P (targetCoverageEvent (intervalMatrix A B) (intervalRhs b lower upper)
        w xTrue lo hi) :=
  hCoverage.trans (MeasureTheory.measure_mono
    (simultaneous_event_subset_target hTrue hEnclose))

/-- Certificate-derived simultaneous target coverage, with empty-set failure. -/
theorem certificate_coverage_probability
    [MeasurableSpace Ω] (P : MeasureTheory.Measure Ω)
    [MeasureTheory.IsProbabilityMeasure P]
    {A : C → V → ℝ} {B : O → V → ℝ} {b : C → ℝ} {w xTrue : V → ℝ}
    {lower upper : Ω → O → ℝ}
    {lam mu : Ω → (C ⊕ (O ⊕ O)) → ℝ} {alpha : ℝ}
    (hTrue : primalFeasible A b xTrue)
    (hlam : ∀ ω, upperCertificate (intervalMatrix A B) w (lam ω))
    (hmu : ∀ ω, lowerCertificate (intervalMatrix A B) w (mu ω))
    (_hAlpha : 0 < alpha ∧ alpha < 1)
    (_hObservationMeasurable : MeasurableSet
      (simultaneousObservationEvent B xTrue lower upper))
    (_hTargetMeasurable : MeasurableSet
      (targetCoverageEvent (intervalMatrix A B) (intervalRhs b lower upper) w xTrue
        (fun ω => -objective (mu ω) (intervalRhs b lower upper ω))
        (fun ω => objective (lam ω) (intervalRhs b lower upper ω))))
    (hCoverage : ENNReal.ofReal (1 - alpha) ≤
      P (simultaneousObservationEvent B xTrue lower upper)) :
    ENNReal.ofReal (1 - alpha) ≤
      P (targetCoverageEvent (intervalMatrix A B) (intervalRhs b lower upper) w xTrue
        (fun ω => -objective (mu ω) (intervalRhs b lower upper ω))
        (fun ω => objective (lam ω) (intervalRhs b lower upper ω))) :=
  hCoverage.trans (MeasureTheory.measure_mono
    (simultaneous_event_subset_certificate_target hTrue hlam hmu))

end ObservationCoverage

end LabourBounds.Certificates

#print axioms LabourBounds.Certificates.transpose_objective
#print axioms LabourBounds.Certificates.weak_dual_upper
#print axioms LabourBounds.Certificates.objective_neg
#print axioms LabourBounds.Certificates.weak_dual_lower
#print axioms LabourBounds.Certificates.weak_dual_bounds
#print axioms LabourBounds.Certificates.equality_certifies_maximum
#print axioms LabourBounds.Certificates.equality_certifies_minimum
#print axioms LabourBounds.Certificates.equality_certifies_isGreatest
#print axioms LabourBounds.Certificates.equality_certifies_isLeast
#print axioms LabourBounds.Certificates.relaxed_bounds
#print axioms LabourBounds.Certificates.feasible_after_relaxation
#print axioms LabourBounds.Certificates.relaxed_upper_ge
#print axioms LabourBounds.Certificates.relaxed_lower_le
#print axioms LabourBounds.Certificates.true_vector_feasible_on_simultaneous_event
#print axioms LabourBounds.Certificates.simultaneous_event_subset_target
#print axioms LabourBounds.Certificates.simultaneous_event_subset_certificate_target
#print axioms LabourBounds.Certificates.simultaneous_coverage_probability
#print axioms LabourBounds.Certificates.certificate_coverage_probability
