import Mathlib.Analysis.SpecificLimits.Normed
import Mathlib.Data.Nat.Choose.Cast
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

/-! Exact infinite-series moments for the one-based geometric retry model.
The interpretation as retries requires the iid Bernoulli-attempt model.
These declarations establish convergence and the stated weighted sums. -/
namespace EffortSpeed

theorem retry_mass (p : ℝ) (hp : 0 < p) (hp1 : p ≤ 1) :
    HasSum (fun n : ℕ => (1 - p) ^ n * p) 1 := by
  have h := (hasSum_geometric_of_lt_one (by linarith : 0 ≤ 1-p)
    (by linarith : 1-p < 1)).mul_right p
  simpa [ne_of_gt hp] using h

theorem retry_first_moment (p : ℝ) (hp : 0 < p) (hp1 : p ≤ 1) :
    HasSum (fun n : ℕ => ((n : ℝ) + 1) * (1 - p) ^ n * p) (1 / p) := by
  have hnorm : ‖1 - p‖ < 1 := by
    rw [Real.norm_eq_abs, abs_of_nonneg (by linarith)]
    linarith
  have h1 := hasSum_coe_mul_geometric_of_norm_lt_one hnorm
  have h0 := hasSum_geometric_of_lt_one (by linarith : 0 ≤ 1-p)
    (by linarith : 1-p < 1)
  convert! (h1.add h0).mul_right p using 1
  · ext n
    ring
  · have hn : p ≠ 0 := ne_of_gt hp
    have he : 1 - (1 - p) = p := by ring
    rw [he]
    field_simp
    ring

theorem retry_labour_moment (p h : ℝ) (hp : 0 < p) (hp1 : p ≤ 1) :
    HasSum (fun n : ℕ => (h * ((n : ℝ) + 1)) * ((1 - p) ^ n * p)) (h / p) := by
  convert! (retry_first_moment p hp hp1).mul_left h using 1
  · ext n
    ring
  · ring

theorem retry_elapsed_moment (p w t : ℝ) (hp : 0 < p) (hp1 : p ≤ 1) :
    HasSum (fun n : ℕ => (w + t * ((n : ℝ) + 1)) * ((1 - p) ^ n * p))
      (w + t / p) := by
  convert! ((retry_mass p hp hp1).mul_left w).add
    (retry_labour_moment p t hp hp1) using 1
  · ext n
    ring
  · ring

theorem retry_second_moment (p : ℝ) (hp : 0 < p) (hp1 : p ≤ 1) :
    HasSum (fun n : ℕ => ((n : ℝ) + 1)^2 * ((1 - p)^n * p))
      ((2-p) / p^2) := by
  have hnorm : ‖1 - p‖ < 1 := by
    rw [Real.norm_eq_abs, abs_of_nonneg (by linarith)]
    linarith
  have h2 := (hasSum_choose_mul_geometric_of_norm_lt_one 2 hnorm).mul_right (2*p)
  have h1 := retry_first_moment p hp hp1
  convert! h2.sub h1 using 1
  · ext n
    rw [Nat.cast_choose_two]
    push_cast
    ring
  · have he : 1-(1-p) = p := by ring
    rw [he]
    have hn : p ≠ 0 := ne_of_gt hp
    field_simp
    ring

theorem retry_central_second_moment (p : ℝ) (hp : 0 < p) (hp1 : p ≤ 1) :
    HasSum (fun n : ℕ => (((n : ℝ) + 1) - 1/p)^2 * ((1-p)^n*p))
      ((1-p)/p^2) := by
  have h2 := retry_second_moment p hp hp1
  have h1 := (retry_first_moment p hp hp1).mul_left (2/p)
  have h0 := (retry_mass p hp hp1).mul_left (1/p^2)
  convert! (h2.sub h1).add h0 using 1
  · ext n
    ring
  · have hn : p ≠ 0 := ne_of_gt hp
    field_simp
    ring

theorem retry_labour_variance (p h : ℝ) (hp : 0 < p) (hp1 : p ≤ 1) :
    HasSum (fun n : ℕ => (h*((n : ℝ)+1)-h/p)^2 * ((1-p)^n*p))
      (h^2*(1-p)/p^2) := by
  convert! (retry_central_second_moment p hp hp1).mul_left (h^2) using 1
  · ext n
    ring
  · ring

end EffortSpeed
