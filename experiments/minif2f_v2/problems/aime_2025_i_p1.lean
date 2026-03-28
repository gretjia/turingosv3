import Mathlib

set_option maxHeartbeats 400000

open BigOperators Real Nat Topology Rat

/-- 2025 AIME I Problem 1:
Find the sum of all integer bases b > 9 for which 17_b divides 97_b.
17 in base b = b + 7, 97 in base b = 9*b + 7.

Formalized WITHOUT finite range — forces constructive proof.
Agent must discover: 9b+7 = 9(b+7) - 56, so (b+7)|56.
Answer: 70 (bases 21 and 49) -/
theorem aime_2025_i_p1_characterize :
    ∀ b : ℕ, b > 9 → ((b + 7) ∣ (9 * b + 7) ↔ (b = 21 ∨ b = 49)) := by
  sorry

theorem aime_2025_i_p1_sum : 21 + 49 = 70 := by
  sorry
