import Mathlib

set_option maxHeartbeats 400000

open BigOperators Real Nat Topology Rat

/-- 2025 AIME I Problem 1:
Find the sum of all integer bases b > 9 for which 17_b divides 97_b.
17 in base b = b + 7, 97 in base b = 9*b + 7.
Answer: 70 (bases are 21 and 49) -/
theorem aime_2025_i_p1 (S : Finset ℕ)
    (hS : S = Finset.filter (fun b : ℕ => b > 9 ∧ (b + 7) ∣ (9 * b + 7)) (Finset.range 100)) :
    S.sum id = 70 := by
  sorry
