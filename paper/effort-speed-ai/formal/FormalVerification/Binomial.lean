import Mathlib.Data.Fintype.BigOperators
import Mathlib.Tactic.NormNum

namespace EffortSpeed
open scoped BigOperators

def positiveSigns (v : Fin 10 → Bool) : ℕ := ∑ i, if v i then 1 else 0

def rejectionEvent : Finset (Fin 10 → Bool) :=
  Finset.univ.filter fun v => positiveSigns v = 0 ∨ positiveSigns v = 1 ∨
    positiveSigns v = 9 ∨ positiveSigns v = 10

theorem sign_space_cardinality : Fintype.card (Fin 10 → Bool) = 1024 := by
  norm_num

set_option maxRecDepth 100000 in
set_option maxHeartbeats 8000000 in
theorem sign_event_cardinality : rejectionEvent.card = 22 := by decide

theorem sign_rejection_probability :
    (rejectionEvent.card : ℚ) / Fintype.card (Fin 10 → Bool) = 11/512 := by
  rw [sign_event_cardinality, sign_space_cardinality]
  norm_num

end EffortSpeed
