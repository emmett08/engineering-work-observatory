import Mathlib.Data.Real.Basic
import Mathlib.Data.Nat.Choose.Basic
import Mathlib.Data.Fintype.BigOperators
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Positivity

/-!
Exact finite and algebraic claims for the effort–speed methodological article.
These declarations make no assertion that a company satisfies their premises.
-/
namespace EffortSpeed
open scoped BigOperators

/- prop:binomial: the four disjoint binomial masses, in exact rational arithmetic. -/
theorem binomial_correction :
    (((Nat.choose 10 0 + Nat.choose 10 1 + Nat.choose 10 9 + Nat.choose 10 10 : ℕ) : ℚ)
      / 2 ^ 10) = 11 / 512 := by norm_num

/- ex:variance: an actual four-point uniform probability space. -/
def sign (b : Bool) : ℚ := if b then 1 else -1
def mean4 (f : Bool × Bool → ℚ) : ℚ := (∑ ω, f ω) / 4
def cov4 (f g : Bool × Bool → ℚ) : ℚ :=
  mean4 (fun ω => (f ω - mean4 f) * (g ω - mean4 g))
def var4 (f : Bool × Bool → ℚ) : ℚ := cov4 f f
def score4 (ω : Bool × Bool) : ℚ := 2 + sign ω.1
def duration4 (ω : Bool × Bool) : ℚ := 10 + 4 * sign ω.1 + 3 * sign ω.2
def efficiency4 (ω : Bool × Bool) : ℚ := 3 * sign ω.1 + 3 * sign ω.2

theorem four_point_normalized : mean4 (fun _ => 1) = 1 := by
  norm_num [mean4, Fintype.sum_prod_type]

theorem covariance_counterexample :
    var4 score4 = 1 ∧ var4 duration4 = 25 ∧
    cov4 score4 duration4 = 4 ∧ var4 efficiency4 = 18 ∧
    (cov4 score4 duration4)^2 / (var4 score4 * var4 duration4) = 16/25 ∧
    var4 efficiency4 / var4 duration4 >
      1 - (cov4 score4 duration4)^2 / (var4 score4 * var4 duration4) := by
  norm_num [var4, cov4, mean4, score4, duration4, efficiency4, sign,
    Fintype.sum_prod_type]

theorem counterexample_positive_and_decomposed (ω : Bool × Bool) :
    0 < score4 ω ∧ 0 < duration4 ω ∧
    duration4 ω = 10 + sign ω.1 + efficiency4 ω := by
  rcases ω with ⟨x,z⟩
  cases x <;> cases z <;>
    norm_num [score4, duration4, efficiency4, sign]

/- prop:nonidentification: independently drawn hidden states and conditionally
independent emissions. The recursive marginal explicitly sums the two hidden
states at EVERY time step. This is a finite cylinder law, not a single product. -/
def bernoulliMass (p : ℚ) (b : Bool) : ℚ := if b then p else 1-p

def observedLaw (π c : ℚ) : List Bool → ℚ
  | [] => 1
  | o :: os =>
      (∑ z : Bool, bernoulliMass π z * bernoulliMass (if z then c else 0) o)
        * observedLaw π c os

def bernoulliLaw (p : ℚ) : List Bool → ℚ
  | [] => 1
  | b :: bs => bernoulliMass p b * bernoulliLaw p bs

theorem marginal_emission (π c : ℚ) (o : Bool) :
    (∑ z : Bool, bernoulliMass π z * bernoulliMass (if z then c else 0) o)
      = bernoulliMass (π*c) o := by
  cases o <;> simp [bernoulliMass] <;> ring

theorem observed_law_factorization (π c : ℚ) (os : List Bool) :
    observedLaw π c os = bernoulliLaw (π*c) os := by
  induction os with
  | nil => rfl
  | cons o os ih => rw [observedLaw, bernoulliLaw, marginal_emission, ih]

theorem observed_law_equal (os : List Bool) :
    observedLaw (1/2) (1/5) os = observedLaw (1/4) (2/5) os := by
  rw [observed_law_factorization, observed_law_factorization]
  norm_num

/- Explicit expectation over all 2^n hidden histories, evaluated recursively.
Each branch receives its Bernoulli mass; no stochastic mean is assumed. -/
def historyExpectation (π : ℚ) : ℕ → (List Bool → ℚ) → ℚ
  | 0, f => f []
  | n+1, f =>
      (1-π) * historyExpectation π n (fun xs => f (false::xs)) +
      π * historyExpectation π n (fun xs => f (true::xs))

def activeCount : List Bool → ℚ
  | [] => 0
  | b::bs => (if b then 1 else 0) + activeCount bs

theorem history_mass_one (π : ℚ) (n : ℕ) :
    historyExpectation π n (fun _ => 1) = 1 := by
  induction n with
  | zero => rfl
  | succ n ih => simp [historyExpectation, ih]

theorem history_expectation_nonnegative (π : ℚ) (n : ℕ)
    (hπ : 0 ≤ π) (hπ1 : π ≤ 1) (f : List Bool → ℚ)
    (hf : ∀ xs, 0 ≤ f xs) : 0 ≤ historyExpectation π n f := by
  induction n generalizing f with
  | zero => exact hf []
  | succ n ih =>
    apply add_nonneg
    · exact mul_nonneg (by linarith) (ih _ (fun xs => hf (false::xs)))
    · exact mul_nonneg hπ (ih _ (fun xs => hf (true::xs)))

theorem nonidentification_parameters_valid :
    (0:ℚ) < 1/2 ∧ (1/2:ℚ) < 1 ∧ (0:ℚ) < 1/5 ∧ (1/5:ℚ) < 1 ∧
    (0:ℚ) < 1/4 ∧ (1/4:ℚ) < 1 ∧ (0:ℚ) < 2/5 ∧ (2/5:ℚ) < 1 ∧
    (1/2:ℚ) ≠ 1/4 := by norm_num

theorem expected_count_offset (π : ℚ) (n : ℕ) (k : ℚ) :
    historyExpectation π n (fun xs => k + activeCount xs) = k + (n:ℚ)*π := by
  induction n generalizing k with
  | zero => simp [historyExpectation, activeCount]
  | succ n ih =>
    simp only [historyExpectation, activeCount, Bool.false_eq_true, ↓reduceIte, zero_add]
    have ht : (fun xs => k + (1 + activeCount xs)) =
        (fun xs => (k+1) + activeCount xs) := by funext xs; ring
    rw [ht, ih, ih]
    push_cast
    ring

theorem latent_effort_mean (π : ℚ) (n : ℕ) :
    historyExpectation π n activeCount = (n:ℚ)*π := by
  simpa using expected_count_offset π n 0

theorem latent_effort_unequal (n : ℕ) (hn : 0 < n) :
    historyExpectation (1/2) n activeCount > historyExpectation (1/4) n activeCount := by
  rw [latent_effort_mean, latent_effort_mean]
  have hn' : (0:ℚ) < n := by exact_mod_cast hn
  linarith

/- prop:sampling: an actual finite expectation identity. Indicator moments,
not independence, are its hypotheses; d_m*a_m is supplied as a_m. -/
theorem sampling_unbiased_finite {Ω M : Type*} [Fintype Ω] [Fintype M]
    (w : Ω → ℝ) (I : M → Ω → ℝ) (a π : M → ℝ)
    (hπ : ∀ m, π m ≠ 0) (hI : ∀ m, ∑ ω, w ω * I m ω = π m) :
    (∑ ω, w ω * (∑ m, I m ω * a m / π m)) = ∑ m, a m := by
  simp_rw [Finset.mul_sum]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro m _
  calc
    (∑ ω, w ω * (I m ω * a m / π m)) =
        (∑ ω, w ω * I m ω) * (a m / π m) := by
      rw [Finset.sum_mul]
      apply Finset.sum_congr rfl
      intro ω _
      ring
    _ = a m := by rw [hI]; field_simp [hπ m]

/- prop:capacity: algebraic conclusion conditional on integrated resource and
path bounds; no DAG scheduling or integration theorem is claimed here. -/
theorem capacity_lower_bound (L C H G m k : ℝ)
    (hm : 0 < m) (hk : 0 < k) (hC : C ≤ L)
    (hH : H ≤ m*L) (hG : G ≤ k*L) :
    max C (max (H/m) (G/k)) ≤ L := by
  apply max_le hC
  apply max_le
  · exact (div_le_iff₀ hm).2 (by nlinarith)
  · exact (div_le_iff₀ hk).2 (by nlinarith)

/- cor:improvements: all denominators that are cross-multiplied are positive. -/
theorem ratio_improves_iff (h₀ h₁ p₀ p₁ : ℝ)
    (hh₀ : 0 < h₀) (hp₀ : 0 < p₀) (hp₁ : 0 < p₁) :
    h₁/p₁ < h₀/p₀ ↔ h₁/h₀ < p₁/p₀ := by
  rw [div_lt_div_iff₀ hp₁ hp₀, div_lt_div_iff₀ hh₀ hp₀]
  constructor <;> intro h <;> nlinarith

theorem productivity_improves_iff (S r₀ r₁ p₀ p₁ h₀ h₁ : ℝ)
    (hS : 0 < S) (hr₀ : 0 < r₀) (hp₀ : 0 < p₀)
    (hh₀ : 0 < h₀) (hh₁ : 0 < h₁) :
    S*r₀*p₀/h₀ < S*r₁*p₁/h₁ ↔ h₁/h₀ < (r₁*p₁)/(r₀*p₀) := by
  rw [div_lt_div_iff₀ hh₀ hh₁, div_lt_div_iff₀ hh₀ (mul_pos hr₀ hp₀)]
  constructor
  · intro h
    apply (mul_lt_mul_iff_right₀ hS).mp
    nlinarith only [h]
  · intro h
    have hm := mul_lt_mul_of_pos_left h hS
    nlinarith only [hm]

theorem retry_reversal :
    (1/2:ℚ) < 1 ∧ (1/2:ℚ)/(1/5) > 1/(9/10) := by norm_num

/- prop:capped: pointwise finite tail identity, valid also at K=0. -/
theorem min_eq_tail (n K : ℕ) :
    ((min n K : ℕ) : ℝ) = ∑ j ∈ Finset.range K, if j < n then (1:ℝ) else 0 := by
  induction K with
  | zero => simp
  | succ K ih =>
    rw [Finset.sum_range_succ, ← ih]
    by_cases hn : n ≤ K
    · simp [Nat.min_eq_left hn, Nat.min_eq_left (Nat.le_succ_of_le hn), Nat.not_lt.mpr hn]
    · have hKn : K < n := Nat.lt_of_not_ge hn
      rw [if_pos hKn, Nat.min_eq_right (Nat.le_of_lt hKn), Nat.min_eq_right hKn]
      push_cast
      ring

theorem finite_tail_sum {Ω : Type*} [Fintype Ω]
    (w : Ω → ℝ) (N : Ω → ℕ) (K : ℕ) :
    (∑ ω, w ω * ((min (N ω) K : ℕ) : ℝ)) =
      ∑ j ∈ Finset.range K, ∑ ω, if j < N ω then w ω else 0 := by
  simp_rw [min_eq_tail, Finset.mul_sum]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro j _
  apply Finset.sum_congr rfl
  intro ω _
  split_ifs <;> simp_all

def cappedTail (p : ℝ) (K : ℕ) : ℝ := ∑ j ∈ Finset.range K, (1-p)^j

theorem capped_tail_identity (p : ℝ) (K : ℕ) :
    p * cappedTail p K = 1 - (1-p)^K := by
  induction K with
  | zero => simp [cappedTail]
  | succ K ih =>
    simp only [cappedTail, Finset.sum_range_succ] at *
    rw [mul_add, ih, pow_succ]
    ring

theorem capped_tail_closed_form (p : ℝ) (K : ℕ) (hp : p ≠ 0) :
    cappedTail p K = (1-(1-p)^K)/p := by
  apply (eq_div_iff hp).2
  nlinarith [capped_tail_identity p K]

theorem capped_expectation_of_tails {Ω : Type*} [Fintype Ω]
    (w : Ω → ℝ) (N : Ω → ℕ) (K : ℕ) (p : ℝ)
    (h : ∀ j < K, (∑ ω, if j < N ω then w ω else 0) = (1-p)^j) :
    (∑ ω, w ω * ((min (N ω) K : ℕ) : ℝ)) = cappedTail p K := by
  rw [finite_tail_sum]
  apply Finset.sum_congr rfl
  intro j hj
  exact h j (Finset.mem_range.mp hj)

theorem capped_zero_probability (K : ℕ) :
    cappedTail 0 K = K ∧ 1-(1-(0:ℝ))^K = 0 := by simp [cappedTail]

theorem capped_zero_cap (p : ℝ) :
    cappedTail p 0 = 0 ∧ 1-(1-p)^0 = 0 := by simp [cappedTail]

theorem capped_one_probability (K : ℕ) (hK : 0 < K) :
    cappedTail 1 K = 1 ∧ 1-(1-(1:ℝ))^K = 1 := by
  rcases K with _ | K
  · omega
  · simp [cappedTail, Finset.sum_range_succ']

/- Finite joint probability table: rows are defect status, columns gate pass. -/
def detectionJoint (d s : ℝ) (defective passed : Bool) : ℝ :=
  if defective then d * (if passed then 1-s else s)
  else if passed then 1-d else 0

theorem detection_table (d s : ℝ) :
    (∑ a : Bool, ∑ b : Bool, detectionJoint d s a b) = 1 ∧
    (∑ a : Bool, detectionJoint d s a true) = 1-d*s ∧
    detectionJoint d s false true = 1-d ∧ detectionJoint d s false false = 0 := by
  simp [detectionJoint]
  constructor <;> ring

theorem detection_joint_nonnegative (d s : ℝ) (hd : 0 ≤ d) (hd1 : d ≤ 1)
    (hs : 0 ≤ s) (hs1 : s ≤ 1) (a b : Bool) : 0 ≤ detectionJoint d s a b := by
  cases a <;> cases b <;> dsimp [detectionJoint] <;> positivity

theorem detection_conditional_correct (d s : ℝ) :
    detectionJoint d s false true / (∑ a : Bool, detectionJoint d s a true) =
      (1-d)/(1-d*s) := by
  rw [(detection_table d s).2.1, (detection_table d s).2.2.1]

/- prop:detection: algebraic rate cancellation and monotonic tradeoff. -/
theorem detection_pass_positive (d s : ℝ) (hd : 0 ≤ d) (hd1 : d < 1)
    (hs1 : s ≤ 1) : 0 < 1-d*s := by nlinarith

theorem detection_correct_rate (S d s t : ℝ) (hd : 0 ≤ d) (hd1 : d < 1)
    (hs1 : s ≤ 1) :
    S * ((1-d)/(1-d*s)) * (1-d*s) / t = S*(1-d)/t := by
  have hp := ne_of_gt (detection_pass_positive d s hd hd1 hs1)
  congr 1
  field_simp

theorem detection_tradeoff (S d s₀ s₁ t : ℝ)
    (hS : 0 < S) (hd : 0 < d) (hd1 : d < 1) (ht : 0 < t)
    (hs : s₀ < s₁) (hs₁ : s₁ ≤ 1) :
    S*(1-d*s₁)/t < S*(1-d*s₀)/t ∧
    (1-d)/(1-d*s₀) < (1-d)/(1-d*s₁) := by
  have h₀ : 0 < 1-d*s₀ := detection_pass_positive d s₀ (le_of_lt hd) hd1 (le_trans (le_of_lt hs) hs₁)
  have h₁ : 0 < 1-d*s₁ := detection_pass_positive d s₁ (le_of_lt hd) hd1 hs₁
  constructor
  · apply (div_lt_div_iff_of_pos_right ht).2
    apply mul_lt_mul_of_pos_left _ hS
    nlinarith
  · apply (div_lt_div_iff₀ h₀ h₁).2
    apply mul_lt_mul_of_pos_left _ (by linarith : 0 < 1-d)
    nlinarith

theorem zero_defect_no_detection_effect (s : ℝ) :
    1-(0:ℝ)*s = 1 ∧ (1-(0:ℝ))/(1-(0:ℝ)*s) = 1 := by norm_num

end EffortSpeed
