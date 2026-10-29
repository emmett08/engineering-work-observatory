import FormalVerification
import Mathlib.Tactic.NormNum

open scoped BigOperators
open EffortSpeed

-- Total division cannot be used as the p=0 continuous extension.
example : cappedTail 0 3 = 3 ∧ (1-(1-(0:ℝ))^3)/(0:ℝ) = 0 := by
  norm_num [cappedTail]

-- The cap-zero boundary survives even when the pass probability is positive.
example : cappedTail (1/2) 0 = 0 ∧ 1-(1-(1/2:ℝ))^0 = 0 := by
  exact capped_zero_cap (1/2)

example : cappedTail 1 1 = 1 ∧ 1-(1-(1:ℝ))^1 = 1 := by
  exact capped_one_probability 1 (by norm_num)

-- A finite two-atom stopped process, not only quotient arithmetic.
example : (∑ ω : Bool, (1/2:ℝ) * ((min (if ω then 2 else 1) 2 : ℕ) : ℝ)) =
    cappedTail (1/2) 2 := by
  norm_num [cappedTail]

-- Empty and positive-length histories respect the nonidentification boundary.
example : observedLaw (1/2) (1/5) [] = 1 := rfl
example : historyExpectation (1/2) 0 activeCount =
    historyExpectation (1/4) 0 activeCount := rfl
example : historyExpectation (1/2) 1 activeCount >
    historyExpectation (1/4) 1 activeCount := by
  exact latent_effort_unequal 1 (by norm_num)

-- A zero reference score defeats an otherwise true strict-rate threshold.
example : ¬ ((0:ℝ)*1*1/1 < 0*1*1/(1/2)) ∧
    (1/2:ℝ)/1 < (1*1)/(1*1) := by norm_num

-- A zero baseline cost defeats the denominator-free version of the iff.
example : ¬ ((1:ℝ)/1 < 0/1 ↔ 1/0 < 1/1) := by norm_num

-- Intrinsic-duration improvement need not survive unequal initial waits.
example : (1/2:ℝ)/1 < 1/1 ∧ 2+(1/2:ℝ)/1 > 0+1/1 := by norm_num

-- At d=0 the purported strict detection effect disappears.
example : (1-(0:ℝ)*0) = 1-0*1 ∧
    (1-(0:ℝ))/(1-0*0) = (1-0)/(1-0*1) := by norm_num

-- At d=s=1 the pass event is impossible; its totalised quotient is not quality.
example : (∑ a : Bool, detectionJoint 1 1 a true) = 0 ∧
    (1-(1:ℝ))/(1-1*1) = 0 := by norm_num [detectionJoint]

example : HasSum (fun n : ℕ => ((n:ℝ)+1)*(1-(1:ℝ))^n*1) 1 := by
  simpa using retry_first_moment 1 (by norm_num) (by norm_num)

example : HasSum (fun n : ℕ => (3*((n:ℝ)+1)-3/(1:ℝ))^2*((1-(1:ℝ))^n*1)) 0 := by
  simpa using retry_labour_variance 1 3 (by norm_num) (by norm_num)

-- Perfectly dependent inclusion indicators can still give unbiased totals.
example : (∑ ω : Bool, (1/2:ℝ) *
    ((if ω then 1 else 0) * 2 / (1/2) +
     (if ω then 1 else 0) * 3 / (1/2))) = 5 := by norm_num
