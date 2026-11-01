import Mathlib.MeasureTheory.Integral.IntervalIntegral.Basic
import Mathlib.MeasureTheory.Integral.Bochner.Set
import Mathlib.Tactic.Linarith

/-!
# Interval observations, cell integrals and consecutive binary allocations

Time and labour are real valued. Integrals use Lebesgue measure on `ℝ`.
The finite-cell observation bridge is the actual integral-additivity theorem,
not an assumption that the observation totals have already been split.
-/

open scoped BigOperators
open MeasureTheory Set

noncomputable section

namespace LabourBounds

theorem bounded_measurable_intervalIntegrable (a : ℝ → ℝ) (ha : AEStronglyMeasurable a volume)
    (hnonneg : ∀ᵐ t ∂volume, 0 ≤ a t) (hupper : ∀ᵐ t ∂volume, a t ≤ 1)
    (left right : ℝ) : IntervalIntegrable a volume left right := by
  have hnorm : ∀ᵐ t ∂volume, ‖a t‖ ≤ 1 := by
    filter_upwards [hnonneg, hupper] with t ht₀ ht₁
    simpa [Real.norm_eq_abs, abs_of_nonneg ht₀] using ht₁
  constructor
  · exact Measure.integrableOn_of_bounded (by simp) ha
      (ae_restrict_of_ae hnorm)
  · exact Measure.integrableOn_of_bounded (by simp) ha
      (ae_restrict_of_ae hnorm)

def cellMass (a : ℝ → ℝ) (b : ℕ → ℝ) (k : ℕ) : ℝ :=
  ∫ t in b k..b (k + 1), a t

theorem allocation_integrable_of_capacity {S : Type*} [Fintype S]
    (a : S → ℝ → ℝ) (ha : ∀ i, AEStronglyMeasurable (a i) volume)
    (hnonneg : ∀ i t, 0 ≤ a i t)
    (hcapacity : ∀ᵐ t ∂volume, (∑ i : S, a i t) ≤ 1)
    (i : S) (left right : ℝ) : IntervalIntegrable (a i) volume left right := by
  apply bounded_measurable_intervalIntegrable (a i) (ha i)
  · exact Filter.Eventually.of_forall (hnonneg i)
  · filter_upwards [hcapacity] with t ht
    exact le_trans (Finset.single_le_sum (fun j _ => hnonneg j t) (Finset.mem_univ i)) ht

theorem history_to_cell_constraints {S : Type*} [Fintype S]
    (a : S → ℝ → ℝ) (ha : ∀ i, AEStronglyMeasurable (a i) volume)
    (hnonneg : ∀ i t, 0 ≤ a i t)
    (hcapacity : ∀ᵐ t ∂volume, (∑ i : S, a i t) ≤ 1)
    (left right : ℝ) (hlr : left ≤ right) :
    (∀ i, 0 ≤ ∫ t in left..right, a i t) ∧
      (∑ i : S, ∫ t in left..right, a i t) ≤ right - left := by
  have hi : ∀ i : S, IntervalIntegrable (a i) volume left right :=
    fun i => allocation_integrable_of_capacity a ha hnonneg hcapacity i left right
  constructor
  · intro i
    exact intervalIntegral.integral_nonneg_of_forall hlr (hnonneg i)
  · rw [← intervalIntegral.integral_finsetSum (fun i _ => hi i)]
    calc
      (∫ t in left..right, ∑ i : S, a i t) ≤ ∫ _ in left..right, (1 : ℝ) := by
        apply intervalIntegral.integral_mono_ae hlr
        · have hfun : (fun t => ∑ i : S, a i t) = ∑ i : S, a i := by
            funext t
            simp
          rw [hfun]
          exact IntervalIntegrable.sum Finset.univ fun i _ => hi i
        · exact intervalIntegrable_const
        · exact hcapacity
      _ = right - left := by simp

theorem zero_extension_measurable (D : Set ℝ) (hD : MeasurableSet D)
    (a : ℝ → ℝ) (ha : AEStronglyMeasurable a (volume.restrict D)) :
    AEStronglyMeasurable (D.indicator a) volume :=
  (aestronglyMeasurable_indicator_iff hD).mpr ha

theorem zero_extension_nonnegative (D : Set ℝ) (a : ℝ → ℝ)
    (ha : ∀ t ∈ D, 0 ≤ a t) : ∀ t, 0 ≤ D.indicator a t := by
  intro t
  by_cases ht : t ∈ D
  · simpa [ht] using ha t ht
  · simp [ht]

theorem zero_extension_capacity {S : Type*} [Fintype S]
    (D : Set ℝ) (hD : MeasurableSet D) (a : S → ℝ → ℝ)
    (hcap : ∀ᵐ t ∂volume.restrict D, (∑ i : S, a i t) ≤ 1) :
    ∀ᵐ t ∂volume, (∑ i : S, D.indicator (a i) t) ≤ 1 := by
  filter_upwards [(ae_restrict_iff' hD).mp hcap] with t ht
  by_cases htD : t ∈ D
  · simpa [htD] using ht htD
  · simp [htD]

theorem zero_extension_preserves_integral (D : Set ℝ) (a : ℝ → ℝ)
    (left right : ℝ) (hlr : left ≤ right) (hs : Icc left right ⊆ D) :
    (∫ t in left..right, D.indicator a t) = ∫ t in left..right, a t := by
  apply intervalIntegral.integral_congr
  intro t ht
  have htD : t ∈ D := hs (by simpa [uIcc_of_le hlr] using ht)
  simp [htD]

theorem localized_history_intervalIntegrable {S : Type*} [Fintype S]
    (D : Set ℝ) (hD : MeasurableSet D) (a : S → ℝ → ℝ)
    (ha : ∀ i, AEStronglyMeasurable (a i) (volume.restrict D))
    (hnonneg : ∀ i t, t ∈ D → 0 ≤ a i t)
    (hcapacity : ∀ᵐ t ∂volume.restrict D, (∑ i : S, a i t) ≤ 1)
    (i : S) (left right : ℝ) (hlr : left ≤ right) (hs : Icc left right ⊆ D) :
    IntervalIntegrable (a i) volume left right := by
  have hi := allocation_integrable_of_capacity
    (fun i => D.indicator (a i))
    (fun i => zero_extension_measurable D hD (a i) (ha i))
    (fun i => zero_extension_nonnegative D (a i) (hnonneg i))
    (zero_extension_capacity D hD a hcapacity) i left right
  apply hi.congr
  intro t ht
  have htD : t ∈ D := hs (Ioc_subset_Icc_self (by simpa [uIoc_of_le hlr] using ht))
  simp [htD]

theorem localized_history_to_cell_constraints {S : Type*} [Fintype S]
    (D : Set ℝ) (hD : MeasurableSet D) (a : S → ℝ → ℝ)
    (ha : ∀ i, AEStronglyMeasurable (a i) (volume.restrict D))
    (hnonneg : ∀ i t, t ∈ D → 0 ≤ a i t)
    (hcapacity : ∀ᵐ t ∂volume.restrict D, (∑ i : S, a i t) ≤ 1)
    (left right : ℝ) (hlr : left ≤ right) (hs : Icc left right ⊆ D) :
    (∀ i, 0 ≤ ∫ t in left..right, a i t) ∧
      (∑ i : S, ∫ t in left..right, a i t) ≤ right - left := by
  have hresult := history_to_cell_constraints
    (fun i => D.indicator (a i))
    (fun i => zero_extension_measurable D hD (a i) (ha i))
    (fun i => zero_extension_nonnegative D (a i) (hnonneg i))
    (zero_extension_capacity D hD a hcapacity) left right hlr
  simpa only [zero_extension_preserves_integral D _ left right hlr hs] using hresult

theorem cell_integrals_recover_interval (a : ℝ → ℝ) (b : ℕ → ℝ)
    (p q : ℕ) (hpq : p ≤ q)
    (hi : ∀ k ∈ Finset.Ico p q,
      IntervalIntegrable a volume (b k) (b (k + 1))) :
    (∑ k ∈ Finset.Ico p q, cellMass a b k) = ∫ t in b p..b q, a t := by
  exact intervalIntegral.sum_integral_adjacent_intervals_Ico hpq fun k hk =>
    hi k (Finset.mem_Ico.mpr hk)

theorem grouped_observation_from_cells {S : Type*} [DecidableEq S]
    (G : Finset S) (a : S → ℝ → ℝ) (b : ℕ → ℝ)
    (p q : ℕ) (hpq : p ≤ q)
    (hi : ∀ s ∈ G, ∀ k ∈ Finset.Ico p q,
      IntervalIntegrable (a s) volume (b k) (b (k + 1))) :
    (∑ s ∈ G, ∫ t in b p..b q, a s t) =
      ∑ s ∈ G, ∑ k ∈ Finset.Ico p q, cellMass (a s) b k := by
  apply Finset.sum_congr rfl
  intro s hs
  exact (cell_integrals_recover_interval (a s) b p q hpq (hi s hs)).symm

theorem interval_observation_iff_cells {S : Type*} [DecidableEq S]
    (G : Finset S) (a : S → ℝ → ℝ) (b : ℕ → ℝ)
    (p q : ℕ) (hpq : p ≤ q) (lo hiBound : ℝ)
    (hi : ∀ s ∈ G, ∀ k ∈ Finset.Ico p q,
      IntervalIntegrable (a s) volume (b k) (b (k + 1))) :
    (lo ≤ ∑ s ∈ G, ∫ t in b p..b q, a s t) ∧
      (∑ s ∈ G, ∫ t in b p..b q, a s t) ≤ hiBound ↔
    (lo ≤ ∑ s ∈ G, ∑ k ∈ Finset.Ico p q, cellMass (a s) b k) ∧
      (∑ s ∈ G, ∑ k ∈ Finset.Ico p q, cellMass (a s) b k) ≤ hiBound := by
  rw [grouped_observation_from_cells G a b p q hpq hi]

def prefixMass (x : ℕ → ℝ) (i : ℕ) : ℝ := ∑ j ∈ Finset.range i, x j

def serialBlock (left : ℝ) (x : ℕ → ℝ) (i : ℕ) : Set ℝ :=
  Ico (left + prefixMass x i) (left + prefixMass x (i + 1))

def serialAllocation (left : ℝ) (x : ℕ → ℝ) (i : ℕ) (t : ℝ) : ℝ :=
  (serialBlock left x i).indicator (fun _ => 1) t

theorem prefixMass_succ (x : ℕ → ℝ) (i : ℕ) :
    prefixMass x (i + 1) = prefixMass x i + x i := by
  exact Finset.sum_range_succ x i

theorem prefixMass_nonneg {x : ℕ → ℝ} {n k : ℕ} (hk : k ≤ n)
    (hx : ∀ i < n, 0 ≤ x i) : 0 ≤ prefixMass x k := by
  exact Finset.sum_nonneg fun i hi => hx i (lt_of_lt_of_le (Finset.mem_range.mp hi) hk)

theorem prefixMass_mono {x : ℕ → ℝ} {n i j : ℕ} (hij : i ≤ j) (hjn : j ≤ n)
    (hx : ∀ k < n, 0 ≤ x k) : prefixMass x i ≤ prefixMass x j := by
  apply Finset.sum_le_sum_of_subset_of_nonneg (Finset.range_mono hij)
  intro k hk _
  exact hx k (lt_of_lt_of_le (Finset.mem_range.mp hk) hjn)

theorem serialBlock_subset_cell {left right : ℝ} {x : ℕ → ℝ} {n i : ℕ}
    (hi : i < n) (hx : ∀ k < n, 0 ≤ x k)
    (hcap : prefixMass x n ≤ right - left) :
    serialBlock left x i ⊆ Ico left right := by
  intro t ht
  have hzero := prefixMass_nonneg (Nat.le_of_lt hi) hx
  have hlast := prefixMass_mono (Nat.succ_le_of_lt hi) (le_refl n) hx
  constructor
  · exact le_trans (by linarith) ht.1
  · exact lt_of_lt_of_le ht.2 (by linarith)

theorem serialBlock_disjoint {left : ℝ} {x : ℕ → ℝ} {n i j : ℕ}
    (hi : i < n) (hj : j < n) (hne : i ≠ j)
    (hx : ∀ k < n, 0 ≤ x k) :
    Disjoint (serialBlock left x i) (serialBlock left x j) := by
  apply Set.disjoint_left.mpr
  intro t hti htj
  rcases lt_or_gt_of_ne hne with hij | hji
  · have horder := prefixMass_mono (Nat.succ_le_of_lt hij) (Nat.le_of_lt hj) hx
    exact (not_lt_of_ge (le_trans (by linarith) htj.1)) hti.2
  · have horder := prefixMass_mono (Nat.succ_le_of_lt hji) (Nat.le_of_lt hi) hx
    exact (not_lt_of_ge (le_trans (by linarith) hti.1)) htj.2

theorem serialAllocation_binary (left : ℝ) (x : ℕ → ℝ) (i : ℕ) (t : ℝ) :
    serialAllocation left x i t = 0 ∨ serialAllocation left x i t = 1 := by
  by_cases ht : t ∈ serialBlock left x i
  · exact Or.inr (by simp [serialAllocation, ht])
  · exact Or.inl (by simp [serialAllocation, ht])

theorem serialAllocation_measurable (left : ℝ) (x : ℕ → ℝ) (i : ℕ) :
    Measurable (serialAllocation left x i) := by
  exact measurable_const.indicator measurableSet_Ico

theorem serialAllocation_capacity (left : ℝ) (x : ℕ → ℝ) (n : ℕ)
    (hx : ∀ k < n, 0 ≤ x k) (t : ℝ) :
    (∑ i : Fin n, serialAllocation left x i t) ≤ 1 := by
  classical
  by_cases h : ∃ i : Fin n, t ∈ serialBlock left x i
  · obtain ⟨i, hi⟩ := h
    have hsum : (∑ j : Fin n, serialAllocation left x j t) = 1 := by
      calc
        (∑ j : Fin n, serialAllocation left x j t) = serialAllocation left x i t := by
          apply Finset.sum_eq_single i
          · intro j _ hji
            have hdis := serialBlock_disjoint (left := left) j.isLt i.isLt
              (fun he => hji (Fin.ext he)) hx
            have hj : t ∉ serialBlock left x j := by
              intro ht
              exact Set.disjoint_left.mp hdis ht hi
            simp [serialAllocation, hj]
          · simp
        _ = 1 := by simp [serialAllocation, hi]
    exact le_of_eq hsum
  · have hsum : (∑ j : Fin n, serialAllocation left x j t) = 0 := by
      apply Finset.sum_eq_zero
      intro j _
      have hj : t ∉ serialBlock left x j := fun hj => h ⟨j, hj⟩
      simp [serialAllocation, hj]
    linarith

theorem serialAllocation_integral {left : ℝ} {x : ℕ → ℝ} {n i : ℕ}
    (hi : i < n) (hx : ∀ k < n, 0 ≤ x k) :
    (∫ t, serialAllocation left x i t) = x i := by
  change (∫ t, (Ico (left + prefixMass x i)
    (left + prefixMass x (i + 1))).indicator (1 : ℝ → ℝ) t) = x i
  rw [MeasureTheory.integral_indicator_one measurableSet_Ico]
  rw [Real.volume_real_Ico_of_le]
  · rw [prefixMass_succ]
    linarith
  · have hxi := hx i hi
    rw [prefixMass_succ]
    linarith

theorem serialAllocation_cell_integral {left right : ℝ} {x : ℕ → ℝ} {n i : ℕ}
    (hi : i < n) (hx : ∀ k < n, 0 ≤ x k)
    (hcap : prefixMass x n ≤ right - left) :
    (∫ t in Ico left right, serialAllocation left x i t) = x i := by
  have hsub := serialBlock_subset_cell hi hx hcap
  calc
    (∫ t in Ico left right, serialAllocation left x i t) =
        ∫ t, serialAllocation left x i t := by
      rw [← MeasureTheory.integral_indicator measurableSet_Ico]
      congr 1
      ext t
      by_cases ht : t ∈ serialBlock left x i
      · simp [serialAllocation, ht, hsub ht]
      · simp [serialAllocation, ht]
    _ = x i := serialAllocation_integral hi hx

/- `C` indexes calendar cells, while the first `n` naturals index streams of
one person. Other people are reconstructed independently by the same theorem. -/
def stitchedAllocation {C : Type*} [Fintype C] (left : C → ℝ)
    (x : C → ℕ → ℝ) (i : ℕ) (t : ℝ) : ℝ :=
  ∑ c : C, serialAllocation (left c) (x c) i t

theorem stitchedAllocation_measurable {C : Type*} [Fintype C]
    (left : C → ℝ) (x : C → ℕ → ℝ) (i : ℕ) :
    Measurable (stitchedAllocation left x i) := by
  exact Finset.measurable_sum _ fun c _ => serialAllocation_measurable (left c) (x c) i

theorem stitchedAllocation_on_cell {C : Type*} [Fintype C] [DecidableEq C]
    (left right : C → ℝ) (x : C → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c, prefixMass (x c) n ≤ right c - left c)
    (hdis : Pairwise fun c d => Disjoint (Ico (left c) (right c)) (Ico (left d) (right d)))
    (c : C) (i : ℕ) (hi : i < n) {t : ℝ} (ht : t ∈ Ico (left c) (right c)) :
    stitchedAllocation left x i t = serialAllocation (left c) (x c) i t := by
  apply Finset.sum_eq_single c
  · intro d _ hdc
    have hn : t ∉ serialBlock (left d) (x d) i := by
      intro hd
      exact Set.disjoint_left.mp (hdis hdc)
        (serialBlock_subset_cell hi (hx d) (hcap d) hd) ht
    simp [serialAllocation, hn]
  · simp

theorem stitchedAllocation_outside {C : Type*} [Fintype C]
    (left right : C → ℝ) (x : C → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c, prefixMass (x c) n ≤ right c - left c)
    (i : ℕ) (hi : i < n) {t : ℝ} (ht : ∀ c, t ∉ Ico (left c) (right c)) :
    stitchedAllocation left x i t = 0 := by
  apply Finset.sum_eq_zero
  intro c _
  have hn : t ∉ serialBlock (left c) (x c) i := by
    intro hc
    exact ht c (serialBlock_subset_cell hi (hx c) (hcap c) hc)
  simp [serialAllocation, hn]

theorem stitchedAllocation_binary {C : Type*} [Fintype C] [DecidableEq C]
    (left right : C → ℝ) (x : C → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c, prefixMass (x c) n ≤ right c - left c)
    (hdis : Pairwise fun c d => Disjoint (Ico (left c) (right c)) (Ico (left d) (right d)))
    (i : ℕ) (hi : i < n) (t : ℝ) :
    stitchedAllocation left x i t = 0 ∨ stitchedAllocation left x i t = 1 := by
  classical
  by_cases h : ∃ c, t ∈ Ico (left c) (right c)
  · obtain ⟨c, hc⟩ := h
    rw [stitchedAllocation_on_cell left right x n hx hcap hdis c i hi hc]
    exact serialAllocation_binary (left c) (x c) i t
  · exact Or.inl (stitchedAllocation_outside left right x n hx hcap i hi
      (not_exists.mp h))

theorem stitchedAllocation_capacity {C : Type*} [Fintype C] [DecidableEq C]
    (left right : C → ℝ) (x : C → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c, prefixMass (x c) n ≤ right c - left c)
    (hdis : Pairwise fun c d => Disjoint (Ico (left c) (right c)) (Ico (left d) (right d)))
    (t : ℝ) : (∑ i : Fin n, stitchedAllocation left x i t) ≤ 1 := by
  classical
  by_cases h : ∃ c, t ∈ Ico (left c) (right c)
  · obtain ⟨c, hc⟩ := h
    have heq : (∑ i : Fin n, stitchedAllocation left x i t) =
        ∑ i : Fin n, serialAllocation (left c) (x c) i t := by
      apply Finset.sum_congr rfl
      intro i _
      exact stitchedAllocation_on_cell left right x n hx hcap hdis c i i.isLt hc
    rw [heq]
    exact serialAllocation_capacity (left c) (x c) n (hx c) t
  · have heq : (∑ i : Fin n, stitchedAllocation left x i t) = 0 := by
      apply Finset.sum_eq_zero
      intro i _
      exact stitchedAllocation_outside left right x n hx hcap i i.isLt (not_exists.mp h)
    linarith

theorem stitchedAllocation_cell_integral {C : Type*} [Fintype C] [DecidableEq C]
    (left right : C → ℝ) (x : C → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c, prefixMass (x c) n ≤ right c - left c)
    (hdis : Pairwise fun c d => Disjoint (Ico (left c) (right c)) (Ico (left d) (right d)))
    (c : C) (i : ℕ) (hi : i < n) :
    (∫ t in Ico (left c) (right c), stitchedAllocation left x i t) = x c i := by
  calc
    (∫ t in Ico (left c) (right c), stitchedAllocation left x i t) =
        ∫ t in Ico (left c) (right c), serialAllocation (left c) (x c) i t := by
      apply setIntegral_congr_fun measurableSet_Ico
      intro t ht
      exact stitchedAllocation_on_cell left right x n hx hcap hdis c i hi ht
    _ = x c i := serialAllocation_cell_integral hi (hx c) (hcap c)

theorem stitchedAllocation_intervalIntegrable {C : Type*} [Fintype C] [DecidableEq C]
    (left right : C → ℝ) (x : C → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c, prefixMass (x c) n ≤ right c - left c)
    (hdis : Pairwise fun c d => Disjoint (Ico (left c) (right c)) (Ico (left d) (right d)))
    (i : ℕ) (hi : i < n) (lo up : ℝ) :
    IntervalIntegrable (stitchedAllocation left x i) volume lo up := by
  apply bounded_measurable_intervalIntegrable _
    (stitchedAllocation_measurable left x i).aestronglyMeasurable
  · exact Filter.Eventually.of_forall fun t => by
      rcases stitchedAllocation_binary left right x n hx hcap hdis i hi t with h | h <;> simp [h]
  · exact Filter.Eventually.of_forall fun t => by
      rcases stitchedAllocation_binary left right x n hx hcap hdis i hi t with h | h <;> simp [h]

def GridOrdered (b : ℕ → ℝ) (K : ℕ) : Prop :=
  ∀ i j : ℕ, i ≤ j → j ≤ K → b i ≤ b j

theorem grid_cell_subset_domain (b : ℕ → ℝ) (K : ℕ) (hb : GridOrdered b K)
    (k : ℕ) (hk : k < K) :
    Icc (b k) (b (k + 1)) ⊆ Icc (b 0) (b K) := by
  intro t ht
  exact ⟨le_trans (hb 0 k (Nat.zero_le k) (Nat.le_of_lt hk)) ht.1,
    le_trans ht.2 (hb (k + 1) K (Nat.succ_le_of_lt hk) (le_refl K))⟩

theorem localized_grouped_observation_on_grid {S : Type*} [Fintype S] [DecidableEq S]
    (a : S → ℝ → ℝ) (b : ℕ → ℝ) (K : ℕ) (hb : GridOrdered b K)
    (ha : ∀ i, AEStronglyMeasurable (a i) (volume.restrict (Icc (b 0) (b K))))
    (hnonneg : ∀ i t, t ∈ Icc (b 0) (b K) → 0 ≤ a i t)
    (hcapacity : ∀ᵐ t ∂volume.restrict (Icc (b 0) (b K)), (∑ i : S, a i t) ≤ 1)
    (G : Finset S) (p q : ℕ) (hpq : p ≤ q) (hq : q ≤ K) :
    (∑ s ∈ G, ∫ t in b p..b q, a s t) =
      ∑ s ∈ G, ∑ k ∈ Finset.Ico p q, cellMass (a s) b k := by
  apply grouped_observation_from_cells G a b p q hpq
  intro i _ k hk
  have hkK : k < K := lt_of_lt_of_le (Finset.mem_Ico.mp hk).2 hq
  exact localized_history_intervalIntegrable (Icc (b 0) (b K)) measurableSet_Icc
    a ha hnonneg hcapacity i (b k) (b (k + 1))
    (hb k (k + 1) (Nat.le_succ k) (Nat.succ_le_of_lt hkK))
    (grid_cell_subset_domain b K hb k hkK)

theorem ordered_cells_disjoint (b : ℕ → ℝ) (K : ℕ) (hb : GridOrdered b K) :
    Pairwise fun c d : Fin K =>
      Disjoint (Ico (b c) (b (c + 1))) (Ico (b d) (b (d + 1))) := by
  intro c d hcd
  apply Set.disjoint_left.mpr
  intro t hc hd
  rcases lt_or_gt_of_ne hcd with hlt | hgt
  · have horder := hb (c + 1) d (Nat.succ_le_of_lt hlt) (Nat.le_of_lt d.isLt)
    exact (not_lt_of_ge (le_trans horder hd.1)) hc.2
  · have horder := hb (d + 1) c (Nat.succ_le_of_lt hgt) (Nat.le_of_lt c.isLt)
    exact (not_lt_of_ge (le_trans horder hc.1)) hd.2

def reconstructedHistory (b : ℕ → ℝ) (x : ℕ → ℕ → ℝ) (K : ℕ) : ℕ → ℝ → ℝ :=
  stitchedAllocation (fun c : Fin K => b c) (fun c => x c)

theorem reconstructed_cellMass (b : ℕ → ℝ) (K : ℕ) (hb : GridOrdered b K)
    (x : ℕ → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c < K, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c < K, prefixMass (x c) n ≤ b (c + 1) - b c)
    (k i : ℕ) (hk : k < K) (hi : i < n) :
    cellMass (reconstructedHistory b x K i) b k = x k i := by
  unfold cellMass
  rw [intervalIntegral.integral_of_le (hb k (k + 1) (Nat.le_succ k) (Nat.succ_le_of_lt hk))]
  rw [← integral_Ico_eq_integral_Ioc]
  exact stitchedAllocation_cell_integral
    (fun c : Fin K => b c) (fun c : Fin K => b (c + 1)) (fun c => x c) n
    (fun c => hx c c.isLt) (fun c => hcap c c.isLt)
    (ordered_cells_disjoint b K hb) ⟨k, hk⟩ i hi

theorem reconstructed_observation (b : ℕ → ℝ) (K : ℕ) (hb : GridOrdered b K)
    (x : ℕ → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c < K, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c < K, prefixMass (x c) n ≤ b (c + 1) - b c)
    (G : Finset (Fin n)) (p q : ℕ) (hpq : p ≤ q) (hq : q ≤ K) :
    (∑ i ∈ G, ∫ t in b p..b q, reconstructedHistory b x K i t) =
      ∑ i ∈ G, ∑ k ∈ Finset.Ico p q, x k i := by
  have hint : ∀ i ∈ G, ∀ k ∈ Finset.Ico p q,
      IntervalIntegrable (reconstructedHistory b x K i) volume (b k) (b (k + 1)) := by
    intro i _ k _
    exact stitchedAllocation_intervalIntegrable
      (fun c : Fin K => b c) (fun c : Fin K => b (c + 1)) (fun c => x c) n
      (fun c => hx c c.isLt) (fun c => hcap c c.isLt)
      (ordered_cells_disjoint b K hb) i i.isLt (b k) (b (k + 1))
  rw [grouped_observation_from_cells G _ b p q hpq hint]
  apply Finset.sum_congr rfl
  intro i _
  apply Finset.sum_congr rfl
  intro k hk
  exact reconstructed_cellMass b K hb x n hx hcap k i
    (lt_of_lt_of_le (Finset.mem_Ico.mp hk).2 hq) i.isLt

theorem full_attention_reconstruction (b : ℕ → ℝ) (K : ℕ) (hb : GridOrdered b K)
    (x : ℕ → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c < K, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c < K, prefixMass (x c) n ≤ b (c + 1) - b c) :
    ∃ a : Fin n → ℝ → ℝ,
      (∀ i, Measurable (a i)) ∧
      (∀ i t, a i t = 0 ∨ a i t = 1) ∧
      (∀ t, (∑ i : Fin n, a i t) ≤ 1) ∧
      (∀ i k, k < K → cellMass (a i) b k = x k i) ∧
      (∀ (G : Finset (Fin n)) p q, p ≤ q → q ≤ K →
        (∑ i ∈ G, ∫ t in b p..b q, a i t) =
          ∑ i ∈ G, ∑ k ∈ Finset.Ico p q, x k i) := by
  classical
  refine ⟨fun i => reconstructedHistory b x K i, ?_, ?_, ?_, ?_, ?_⟩
  · intro i
    exact stitchedAllocation_measurable (fun c : Fin K => b c) (fun c => x c) i
  · intro i t
    exact stitchedAllocation_binary
      (fun c : Fin K => b c) (fun c : Fin K => b (c + 1)) (fun c => x c) n
      (fun c => hx c c.isLt) (fun c => hcap c c.isLt)
      (ordered_cells_disjoint b K hb) i i.isLt t
  · intro t
    exact stitchedAllocation_capacity
      (fun c : Fin K => b c) (fun c : Fin K => b (c + 1)) (fun c => x c) n
      (fun c => hx c c.isLt) (fun c => hcap c c.isLt)
      (ordered_cells_disjoint b K hb) t
  · intro i k hk
    exact reconstructed_cellMass b K hb x n hx hcap k i hk i.isLt
  · intro G p q hpq hq
    exact reconstructed_observation b K hb x n hx hcap G p q hpq hq

theorem reconstructed_weighted_target (b : ℕ → ℝ) (K : ℕ) (hb : GridOrdered b K)
    (x : ℕ → ℕ → ℝ) (n : ℕ)
    (hx : ∀ c < K, ∀ i < n, 0 ≤ x c i)
    (hcap : ∀ c < K, prefixMass (x c) n ≤ b (c + 1) - b c)
    (w : Fin K → Fin n → ℝ) :
    (∑ k : Fin K, ∑ i : Fin n,
      w k i * cellMass (reconstructedHistory b x K i) b k) =
      ∑ k : Fin K, ∑ i : Fin n, w k i * x k i := by
  apply Finset.sum_congr rfl
  intro k _
  apply Finset.sum_congr rfl
  intro i _
  rw [reconstructed_cellMass b K hb x n hx hcap k i k.isLt i.isLt]

end LabourBounds
