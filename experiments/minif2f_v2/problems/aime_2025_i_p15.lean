import Mathlib

set_option maxHeartbeats 800000

open BigOperators Real Nat Topology Rat Finset

/-- 2025 AIME I Problem 15:
Let N denote the number of ordered triples of positive integers (a, b, c)
such that a, b, c ≤ 3^6 and a^3 + b^3 + c^3 is a multiple of 3^7.
Find the remainder when N is divided by 1000.
Answer: 735 -/
theorem aime_2025_i_p15 :
    (Finset.filter (fun t : ℕ × ℕ × ℕ =>
      let (a, b, c) := t
      1 ≤ a ∧ a ≤ 3^6 ∧ 1 ≤ b ∧ b ≤ 3^6 ∧ 1 ≤ c ∧ c ≤ 3^6 ∧
      3^7 ∣ (a^3 + b^3 + c^3))
      ((Finset.range (3^6 + 1)) ×ˢ (Finset.range (3^6 + 1)) ×ˢ (Finset.range (3^6 + 1)))).card % 1000 = 735 := by
  sorry
