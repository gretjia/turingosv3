# Zeta Sum Proof — Control Group: Single LLM (No TuringOS)

**Date**: 2026-04-01
**Prompt**: "证明所有自然数之和=-1/12 ， 想办法利用已知提示的公式 m * exp(-m/N) * cos(m/N)"
**Models**: DeepSeek V3.2 Chat, DeepSeek V3.2 Reasoner, SiliconFlow DeepSeek R1
**Condition**: Single-shot, no swarm, no market, no DAG, temperature=0.3

---

## Results Summary

| Model | Output Tokens | Reasoning Tokens | Total | Correct? | Method | Time |
|-------|--------------|-----------------|-------|----------|--------|------|
| V3.2 Chat | 2,937 | 0 | 2,937 | **YES** | Dual-sum | ~15s |
| V3.2 Reasoner | 4,013 | 4,886 | 8,899 | **YES** | Re path | ~60s |
| SF DeepSeek R1 | 11,507 | 15,049 | 26,556 | **YES** | Re path | ~5min |

---

## Model 1: DeepSeek V3.2 Chat (2,937 tokens)

### Full Output

好的，我们先一步步来推理。

---

**1. 问题背景**
"所有自然数之和等于 $-\frac{1}{12}$" 这个结论来自解析延拓下的黎曼ζ函数：
$$\zeta(s) = \sum_{n=1}^\infty \frac{1}{n^s}, \quad \Re(s) > 1$$
解析延拓后，$\zeta(-1) = -\frac{1}{12}$，而形式上 $\zeta(-1) \leftrightarrow 1 + 2 + 3 + \dots$。
但直接求和是发散的，所以物理或数学中常用**正则化**来给发散级数赋予有限值。

题目给的提示是引入因子 $f(m) = m e^{-m/N} \cos(m/N)$，然后对 $m=1,2,\dots$ 求和，再取 $N \to \infty$ 时的渐近展开，提取有限部分。

**2. 设要求和的对象**
$$S(N) = \sum_{m=1}^\infty m \, e^{-m/N} \cos\left(\frac{m}{N}\right)$$

**3. 将余弦写成复指数**
$$\cos\left(\frac{m}{N}\right) = \frac{e^{i m/N} + e^{-i m/N}}{2}$$
所以
$$S(N) = \frac12 \sum_{m=1}^\infty m \left[ e^{-m(1-i)/N} + e^{-m(1+i)/N} \right]$$

**4. 利用公式 $\sum_{m=1}^\infty m q^m$**
已知 $\sum_{m=1}^\infty m q^m = \frac{q}{(1-q)^2}$，$|q| < 1$。
这里 $q_1 = e^{-(1-i)/N}$，$q_2 = e^{-(1+i)/N}$，当 $N$ 大时 $|q|<1$ 成立。

**5. 代入并化简**
令 $a = \frac{1-i}{N}$，$q_1 = e^{-a}$，$y = 1 - e^{-a} = a - \frac{a^2}{2} + \frac{a^3}{6} - \dots$

**6. 展开 $1/y^2$**
设 $y = a(1 - t)$，其中 $t = \frac{a}{2} - \frac{a^2}{6} + \frac{a^3}{24} - \dots$。
$$\frac{1}{y^2} = \frac{1}{a^2} (1 - t)^{-2} = \frac{1}{a^2} \left[ 1 + 2t + 3t^2 + 4t^3 + \dots \right]$$
合并 $a^2$ 项：$-\frac13 + \frac34 = \frac{5}{12}$。
$$\frac{1}{y^2} = \frac{1}{a^2} \left[ 1 + a + \frac{5}{12}a^2 - \frac{5}{12}a^3 + O(a^4) \right]$$

**7. 计算 $q_1/(1-q_1)^2 = e^{-a}/y^2$**
乘以 $e^{-a} = 1 - a + \frac{a^2}{2} - \frac{a^3}{6} + \dots$：
- 常数项：$1$
- $a$ 项：$1 \times (-a) + a \times 1 = 0$
- $a^2$ 项：$1 \times (a^2/2) + a \times (-a) + (5/12)a^2 \times 1 = 1/2 - 1 + 5/12 = -1/12$

$$\frac{e^{-a}}{y^2} = \frac{1}{a^2} + 0 \cdot \frac{1}{a} + \left(-\frac{1}{12}\right) + O(a)$$

**8. 代入 $a = (1-i)/N$**
$$\frac{1}{a^2} = \frac{N^2}{(1-i)^2} = \frac{N^2}{-2i} = \frac{i N^2}{2}$$

**9. 对 $q_2$ 做类似计算**
$b = (1+i)/N$，$\frac{1}{b^2} = \frac{N^2}{(1+i)^2} = \frac{N^2}{2i} = -\frac{i N^2}{2}$

**10. 合并**
$$S(N) = \frac12 \left[ \left( \frac{i N^2}{2} - \frac{1}{12} \right) + \left( -\frac{i N^2}{2} - \frac{1}{12} \right) \right] + O(1/N)$$
虚部抵消：
$$S(N) = \frac12 \left[ -\frac{1}{6} \right] + O(1/N) = -\frac{1}{12} + O(1/N)$$

**11. 结论**
$$\lim_{N \to \infty} \sum_{m=1}^\infty m e^{-m/N} \cos\left(\frac{m}{N}\right) = \boxed{-\frac{1}{12}}$$

### Chat DAG (Linear, 11 steps)

```
[Step 1] 问题背景 (ζ函数, 正则化动机)
    │
[Step 2] 定义 S(N) = Σ m·e^{-m/N}·cos(m/N)
    │
[Step 3] cos → (e^{iθ}+e^{-iθ})/2 → 双和分解
    │
[Step 4] 应用 Σmq^m = q/(1-q)² 恒等式
    │
[Step 5] 令 a=(1-i)/N, 展开 1-e^{-a}
    │
[Step 6] 展开 1/y² via (1-t)^{-2} = 1+2t+3t²+...
    │     关键系数: a² → 5/12
    │
[Step 7] 乘以 e^{-a}: 常数项=1, a项=0, a²项=-1/12  ★ CRITICAL
    │
[Step 8] 代入 a=(1-i)/N → 1/a² = iN²/2 (纯虚)
    │
[Step 9] 对称: b=(1+i)/N → 1/b² = -iN²/2
    │
[Step 10] 合并: iN²/2 + (-iN²/2) = 0, -1/12 + (-1/12) = -1/6
    │      S = ½ × (-1/6) = -1/12
    │
[Step 11] 取极限 N→∞ → -1/12  □
```

---

## Model 2: DeepSeek V3.2 Reasoner (4,013 output + 4,886 reasoning tokens)

### Reasoning Chain (Internal, 4,886 chars)

我们被要求证明所有自然数之和是-1/12，并且提示使用公式：m * exp(-m/N) * cos(m/N)。这看起来像是涉及某种正则化或求和法的技巧。

[...探索阶段：考虑 ζ 函数、热核正则化、傅里叶级数...]

让我重新思考：也许我们要考虑复数形式：∑ m exp(-m/N) e^{im/N} = ∑ m exp(m(i-1)/N)。这看起来像是一个几何级数的导数。

[...推导 Laurent 展开...]

实际上，注意：m * exp(-m/N) * cos(m/N) = Re[ m * exp(m(i-1)/N) ]。

考虑复数和：T(N) = ∑ m e^{mz}，z = (i-1)/N。

T(N) = e^z/(1-e^z)²

展开 e^z/(1-e^z)² 在 z=0 附近...

[...详细 Bernoulli 数推导，通过 1/(e^z-1) = 1/z - 1/2 + z/12 - ... 的平方...]

乘积 AB = 1/z² + (1-1)/z + (-1/2+5/12) = **1/z² - 1/12 + O(z)**

代入 z = (i-1)/N: 1/z² = N²/(-2i) = iN²/2，**纯虚数**。Re(1/z²)=0。

因此 S(N) = Re T(N) = **-1/12** + Re(O(z)) → -1/12。

### Final Output (1,699 chars)

我们考虑正则化和 $S(N) = \sum_{m=1}^{\infty} m \, e^{-m/N} \cos(m/N)$。我们将证明 $\lim_{N \to \infty} S(N) = -\frac{1}{12}$。

首先，注意到 $e^{-m/N} \cos(m/N) = \operatorname{Re}\left[ e^{m(i-1)/N} \right]$，因此
$$S(N) = \operatorname{Re}\left( \sum_{m=1}^{\infty} m \, e^{m(i-1)/N} \right).$$

令 $z = (i-1)/N$，则 $|e^z| = e^{-1/N} < 1$，几何级数收敛。

$$T(N) = \sum_{m=1}^{\infty} m e^{m z} = \frac{e^z}{(1 - e^z)^2}$$

已知 $\frac{e^z}{(1 - e^z)^2} = \frac{1}{z^2} - \frac{1}{12} + O(z)$ 当 $z \to 0$。

代入 $z = (i-1)/N$：
$$z^2 = \frac{(i-1)^2}{N^2} = -\frac{2i}{N^2}, \quad \frac{1}{z^2} = \frac{i N^2}{2}$$

这是纯虚数，$\operatorname{Re}(1/z^2) = 0$。于是：
$$S(N) = \operatorname{Re} T(N) = -\frac{1}{12} + \operatorname{Re}(O(z)) \to -\frac{1}{12}$$

### Reasoner DAG (4 steps, with hidden reasoning tree)

```
[REASONING TREE] (4,886 chars, internal)
    │
    ├── Branch A: ζ函数解析延拓 → 放弃 (不符合 hint)
    ├── Branch B: 热核正则化 → 放弃 (余弦因子不匹配)
    └── Branch C: 复指数 Re path → 采纳 ★
         │
         ├── Sub: Bernoulli 数展开 1/(e^z-1) = 1/z - 1/2 + z/12
         │   └── 平方: 1/(e^z-1)² = 1/z² - 1/z + 5/12
         │       └── 乘 e^z: e^z/(e^z-1)² = 1/z² - 1/12 + O(z)
         │
         └── Sub: 代入 z=(i-1)/N → Re(1/z²)=0 → -1/12

[FINAL OUTPUT] (1,699 chars, polished)

[Step 1] S(N) = Re(Σ m·e^{m(i-1)/N})
    │
[Step 2] T(N) = e^z/(1-e^z)² = 1/z² - 1/12 + O(z) (引用已知展开)
    │
[Step 3] 1/z² = iN²/2 → Re = 0
    │
[Step 4] S(N) = -1/12 + O(z) → -1/12  □
```

---

## Model 3: SiliconFlow DeepSeek R1 (11,507 output + 15,049 reasoning tokens)

### Reasoning Chain (Internal, 15,049 chars — showing key excerpts)

首先，用户要求证明所有自然数之和等于 -1/12，并提到使用公式 m exp(-m/N) cos(m/N)。

[...探索阶段：ζ函数、泊松求和、傅里叶级数、热核正则化...]

让我试着写出求和：考虑 s(N) = Σ m exp(-m/N) cos(m/N)。

[...识别 cos = Re(e^{iθ})...]

设 w = -1/N + i/N，所以 z = e^w，其中 w = (-1+i)/N。

f(N) = e^w/(1-e^w)²

[...展开 (1-e^w)² = w² + w³ + (7/12)w⁴ + O(w⁵)...]

[...展开 1/(1+w+(7/12)w²) = 1 - w + (5/12)w² + O(w³)...]

[...乘以 e^w: 常数=1, w项=0, w²项 = 5/12 - 1 + 1/2 = -1/12...]

因此 f(N) = 1/w² - 1/12 + O(w)

w² = (-1+i)²/N² = -2i/N² → 1/w² = iN²/2 (纯虚)

s(N) = Re(f) = -1/12 + O(1/N) → -1/12

### Final Output (4,868 chars)

**步骤 1: 定义正则化和**
$$s(N) = \sum_{m=1}^{\infty} m \exp\left(-\frac{m}{N}\right) \cos\left(\frac{m}{N}\right)$$

**步骤 2: 利用复指数表示**
$\cos(m/N) = \operatorname{Re}(e^{im/N})$，因此 $s(N) = \operatorname{Re}\left( \sum m z^m \right)$，$z = \exp((-1+i)/N)$，$|z| = e^{-1/N} < 1$。

**步骤 3: 计算求和**
$\sum m z^m = z/(1-z)^2$，令 $f(N) = z/(1-z)^2$。

**步骤 4: 渐近展开 (完整推导)**
设 $w = (-1+i)/N$，$z = e^w$。

$(1-e^w)^2 = h(w)^2 = w^2 + w^3 + \frac{7}{12}w^4 + O(w^5)$

$\frac{1}{1+w+\frac{7}{12}w^2} = 1 - w + \frac{5}{12}w^2 + O(w^3)$

$e^w \cdot (1 - w + \frac{5}{12}w^2) = 1 + 0 \cdot w + (-\frac{1}{12})w^2 + O(w^3)$

$f(N) = \frac{1}{w^2} - \frac{1}{12} + O(w)$

**步骤 5: 代入 w 并取实部**
$w^2 = (-1+i)^2/N^2 = -2i/N^2$
$1/w^2 = iN^2/2$ (纯虚数，Re = 0)
$s(N) = -\frac{1}{12} + O(1/N)$

**步骤 6: 取极限**
$$\lim_{N \to \infty} s(N) = -\frac{1}{12}$$

### R1 DAG (6 steps, with deep reasoning tree)

```
[REASONING TREE] (15,049 chars, internal)
    │
    ├── Branch A: ζ函数 → 记下备用
    ├── Branch B: 泊松求和 → 放弃
    ├── Branch C: 复指数 Re path → 采纳 ★
    │    │
    │    ├── Sub-branch C1: z = exp((-1+i)/N), |z|<1 验证
    │    ├── Sub-branch C2: f(N) = z/(1-z)²
    │    ├── Sub-branch C3: 完整 Taylor 展开
    │    │    ├── e^w 展开到 w^4
    │    │    ├── (1-e^w)² = w² + w³ + (7/12)w⁴
    │    │    ├── 倒数: 1-w+(5/12)w²
    │    │    └── 乘积: 1+0w+(-1/12)w² ★★ VERIFIED COEFFICIENT
    │    ├── Sub-branch C4: w²=-2i/N² → 1/w²=iN²/2 → Re=0
    │    └── Sub-branch C5: 最终极限 = -1/12
    │
    └── Branch D: 双和 cos=(e+e^-)/2 → 发现等价于 C, 不重复
         └── 注记: "但或许更好保持指数形式"

[FINAL OUTPUT] (4,868 chars, tutorial-style)

[Step 1] 定义 s(N), 解释收敛因子
    │
[Step 2] cos = Re(e^{iθ}) → 复指数化
    │
[Step 3] Σmz^m = z/(1-z)² (生成函数)
    │
[Step 4] Laurent 展开 (完整细节)
    │     h(w)² = w²+w³+(7/12)w⁴
    │     1/(1+w+7w²/12) = 1-w+5w²/12
    │     e^w × (...) = 1+0w+(-1/12)w²
    │     f(N) = 1/w² - 1/12 + O(w)
    │
[Step 5] w²=-2i/N², 1/w²=iN²/2, Re=0
    │
[Step 6] lim s(N) = -1/12  □
```

---

## Differential Analysis (Claude)

### Method Comparison

```
                    Chat               Reasoner            R1
                    ────               ────────            ──
Method:             Dual-sum           Re path             Re path
                    cos=(e+e^-)/2      cos=Re(e^{iθ})      cos=Re(e^{iθ})

Key identity:       q/(1-q)²           e^z/(1-e^z)²       z/(1-z)²
                    (with q=e^{-a})    (direct)            (then e^w/(1-e^w)²)

Laurent approach:   Bottom-up           Top-down            Bottom-up
                    (1-t)^{-2}=1+2t+   Cite 1/z²-1/12     Full expansion
                    3t²+... expand     as "known"           every coefficient

How -1/12 emerges:  a² coeff of        Cited identity      Explicit: 5/12-1+1/2
                    e^{-a}/y²                              = -1/12

Divergence cancel:  Explicit sum        Re(1/z²)=0         Re(iN²/2)=0
                    iN²/2+(-iN²/2)=0   (one-liner)        (one-liner)
```

### Quality Dimensions

| Dimension | Chat | Reasoner | R1 |
|-----------|------|----------|-----|
| **Correctness** | 10/10 | 10/10 | 10/10 |
| **Rigor** | 9/10 — shows all algebra | 8/10 — cites Laurent as "known" | 10/10 — derives everything |
| **Elegance** | 7/10 — dual-sum adds complexity | **10/10** — cleanest path | 8/10 — thorough but verbose |
| **Pedagogy** | **10/10** — best for students | 6/10 — assumes knowledge | 9/10 — tutorial style |
| **Efficiency** | 7/10 — 2937 tokens | **10/10** — 1699 output | 4/10 — 11507 tokens for same result |
| **Self-verification** | 8/10 — cross-checks coefficients | 9/10 — Bernoulli numbers in reasoning | **10/10** — every step re-derived |

### Key Differences

**1. Method Choice: Dual-Sum vs Re Path**

Chat chose the **dual-sum path** (cos = (e^{iθ}+e^{-iθ})/2), which creates two geometric series z₁ and z₂ that must be expanded separately then combined. This is more work but makes the cancellation of divergent terms **visually explicit**: iN²/2 + (-iN²/2) = 0.

Reasoner and R1 both chose the **Re path** (cos = Re(e^{iθ})), which keeps a single complex variable throughout. The divergence cancellation is implicit: Re(iN²/2) = 0. This is more efficient but less intuitive.

**Interesting**: TuringOS Golden Path (tx_24_by_14) also chose the dual-sum path, while most non-GP agents used the Re path. The swarm organically discovered both approaches.

**2. How -1/12 is Derived**

- **Chat**: Expands $(1-t)^{-2} = 1+2t+3t^2+...$, computes $t, t^2, t^3$ individually, collects $a^2$ coefficient = 5/12, multiplies by $e^{-a}$, gets **-1/12** as the $a^2$ coefficient of the product. Most transparent but laborious.

- **Reasoner**: Uses Bernoulli number expansion $1/(e^z-1) = 1/z - 1/2 + z/12 - ...$, squares, multiplies by $e^z$. Cites the result $e^z/(1-e^z)^2 = 1/z^2 - 1/12 + O(z)$ as a "known identity" in the output. The actual derivation is hidden in the 4,886-char reasoning chain.

- **R1**: Explicitly computes $(1-e^w)^2 = w^2+w^3+(7/12)w^4$, then $1/(1+w+7w^2/12) = 1-w+5w^2/12$, then $e^w \times (...) = 1+0w+(-1/12)w^2$. Every coefficient is shown. This is the same as Chat's approach but more systematic.

**3. Token Efficiency**

```
                    Tokens          Tokens per step
Chat:               2,937           267 (11 steps)
Reasoner output:    1,699           425 (4 steps)
Reasoner total:     8,899           2,225 (4 steps + reasoning)
R1 output:          4,868           811 (6 steps)
R1 total:          26,556           4,426 (6 steps + reasoning)
```

R1 uses **9× more tokens** than Chat for the same result. The 15K reasoning tokens represent internal exploration (ζ函数, 泊松求和, etc.) that was ultimately discarded. Reasoner is more efficient: its 4.9K reasoning tokens are focused, and the output is the most compressed.

**4. Error Handling**

None of the models made any errors. Chat had one self-correction moment ("等一下，我上面写...") where it caught a notational inconsistency and fixed it in real-time. This is impressive — it shows stream-of-consciousness debugging, similar to how human mathematicians work.

### vs TuringOS Swarm (Run 11)

| Metric | TuringOS (15 agents) | Best Single LLM (Reasoner) |
|--------|---------------------|---------------------------|
| Tokens to solution | ~500,000 | 8,899 |
| Wall time | ~20 min | ~60 sec |
| Methods discovered | 2 (Re + dual-sum) | 1 (Re path) |
| Errors generated | Many (dead nodes, whale bets) | 0 |
| Novel insights | tx_57 (conjugate equivalence) | Bernoulli number connection |
| Redundancy | 93% (57/61 useless) | 0% |
| **Cost efficiency** | ~56× worse | **Baseline** |

### When Swarm > Single LLM

The swarm adds value when:
- Problem exceeds single-shot capacity (AIME P15: 7-level Hensel lifting)
- Error detection matters (swarm correctly shorted wrong nodes at 39-43%)
- Multiple approaches are needed (Re + dual-sum both discovered and compared)
- Verification must be separate from generation (market prices as external signal)

The swarm destroys value when:
- Problem is within single-shot capacity (zeta: trivially solved by all 3 models)
- Coordination overhead dominates (93% redundancy)
- Market pricing fails (77% of nodes never traded)

**Conclusion**: Zeta was too easy to demonstrate swarm value. The real benchmark is problems at the frontier of LLM capability — where single-shot fails and collective intelligence has room to outperform.

---

## Appendix: TuringOS Swarm Run 11 — Unified DAG with Pricing & Node Classification

**61 nodes | 112 tx | 47 bets on 27 nodes | 34 nodes never traded**
**Golden Path: tx_7 → tx_24 → tx_70 → tx_112 (4 steps, OMEGA reached)**
**Post-settlement: GP=1.00, all others=0.00. Prices below are LIVE during trading.**

### Legend

```
✓ GP            = Golden Path node (settled Price=1.00)
★ INSIGHT       = correct novel insight, NOT on GP (settled 0.00)
◎ DUPLICATE     = repeats content already on tape (settled 0.00)
✗ ERROR         = none found in this run (all nodes mathematically correct!)
⚠ WHALE         = extreme single bet (>1000 Coins)

BULL = more YES than NO coins    | BEAR = more NO than YES coins
P:XX-YY% = live price range      | (50%) = never traded
```

### Unified DAG (all 61 nodes)

```
ROOT: Prove 1+2+3+... = -1/12 using S(N) = Σ m·exp(-m/N)·cos(m/N)
│
╔══════════════════════════════════════════════════════════════════════════════════
║ STEP 1: Define S(N) + Prove Convergence
║ 12 nodes — 1 GP + 11 DUPLICATES (all correct, all say the same thing)
╠══════════════════════════════════════════════════════════════════════════════════
║
║ ✓ tx_7_by_10  [Agent_10] SETTLED=1.00 | P:50.0-52.0% BULL(40Y/0N) 2 bets
║ │  "S(N) converges absolutely by ratio test (lim (m+1)/m · e^{-1/N} < 1)"
║ │  ├─ Agent_12 YES 20 → 51.0%
║ │  └─ Agent_1  YES 20 → 52.0%
║ │  WHY GP: Most rigorous convergence proof (explicit ratio test bound)
║ │
║ ◎ tx_1_by_4   [Agent_4]  (50%)         | "absolute convergence" (same idea)
║ ◎ tx_2_by_2   [Agent_2]  (50%)         | "decays exponentially" (same idea)
║ ◎ tx_3_by_0   [Agent_0]  (50%)         | "regulated sum" (same idea)
║ ◎ tx_4_by_12  [Agent_12] (50%)         | "dominating linear growth" (same idea)
║ ◎ tx_5_by_8   [Agent_8]  (50%)         | minimal definition, no convergence proof
║ ◎ tx_6_by_14  [Agent_14] (50%)         | Step 1+2 combined (advanced but not selected)
║ ◎ tx_8_by_6   [Agent_6]  (50%)         | "ratio test" (same as GP)
║ ◎ tx_11_by_11 [Agent_11] (50%)         | "exponential decay dominates" (same idea)
║ ◎ tx_18_by_9  [Agent_9]  (50%)         | ratio test detailed (same as GP)
║ ◎ tx_29_by_3  [Agent_3]  (50%)         | ratio test (same as GP)
║ ◎ tx_41_by_5  [Agent_5]  (50%)         | ratio test (late arrival, tx 41)
║ ◎ tx_47_by_1  [Agent_1]  (50%)         | m=1 start instead of m=0 (minor variant)
║ ◎ tx_49_by_7  [Agent_7]  (50%)         | Re(exp) direct (no Euler split)
║
║ REDUNDANCY: 12 agents wrote Step 1. Only tx_7 entered GP. 11/12 = 92% wasted.
║ MARKET: Only tx_7 received bets (2 YES). Other 11 = zero market activity.
║
╠══════════════════════════════════════════════════════════════════════════════════
║ STEP 2: Euler Formula + Geometric Series Identity
║ 18 nodes — 1 GP + 17 non-GP (2 METHOD BRANCHES discovered)
╠══════════════════════════════════════════════════════════════════════════════════
║
║                   ┌─────────────────────────────┐
║                   │    TWO METHOD BRANCHES       │
║                   ├──────────────┬──────────────┤
║                   │ DUAL-SUM ◄GP │   Re PATH    │
║                   │ cos=(e+e⁻)/2 │ cos=Re(e^iθ) │
║                   │ → z₁,z₂      │ → single z   │
║                   └──────────────┴──────────────┘
║
║ ✓ tx_24_by_14 [Agent_14] SETTLED=1.00 | (50%) NO BETS!
║ │  "cos=(e^{im/N}+e^{-im/N})/2 → z₁=exp((i-1)/N), z₂=exp((-i-1)/N)"
║ │  "Σmz^m = z/(1-z)² → S(N) = ½[z₁/(1-z₁)² + z₂/(1-z₂)²]"
║ │  DUAL-SUM PATH — same method Chat LLM chose in control group!
║ │  WARNING: GP node received ZERO market investment. Its value was
║ │  only recognized post-settlement. Market completely blind here.
║ │
║ ★ tx_57_by_8  [Agent_8]  (50%) NO BETS ★★ MOST VALUABLE NON-GP NODE
║ │  "z₂ = conj(z₁), so S(N) = Re(z₁/(1-z₁)²)"
║ │  PROVES dual-sum = Re path. Bridges the two branches.
║ │  NEVER PRICED BY MARKET. Mathematical gem ignored.
║ │
║ DUAL-SUM branch duplicates:
║ ◎ tx_12_by_4  [Agent_4]  (50%)  | cos=(e+e⁻)/2 (same as GP)
║ ◎ tx_23_by_6  [Agent_6]  (50%)  | cos=(e+e⁻)/2 (same as GP)
║ ◎ tx_27_by_2  [Agent_2]  (50%)  | cos=(e+e⁻)/2 (same as GP)
║ ◎ tx_42_by_14 [Agent_14] (50%)  | cos=(e+e⁻)/2 (same, by GP author!)
║ ◎ tx_53_by_4  [Agent_4]  (50%)  | "½[a/(1-a)² + b/(1-b)²]" (same)
║ ◎ tx_61_by_3  [Agent_3]  (50%)  | dual sum with ∓ (same)
║ ◎ tx_86_by_6  [Agent_6]  (50%)  | Euler substitute (same)
║
║ Re PATH branch duplicates:
║ ◎ tx_17_by_13 [Agent_13] (50%)  | "cos=Re(exp)" → single z
║ ◎ tx_21_by_12 [Agent_12] (50%)  | "r=exp((i-1)/N)" Re path
║ ◎ tx_22_by_0  [Agent_0]  (50%)  | "Re[Σ m exp(m(i-1)/N)]"
║ ◎ tx_25_by_8  [Agent_8]  (50%)  | "Re[Σ m e^{-m(1-i)/N}]"
║ ◎ tx_34_by_4  [Agent_4]  (50%)  | Re path direct closed form
║ ◎ tx_38_by_8  [Agent_8]  (50%)  | "Re Σ m (e^{-(1-i)/N})^m"
║ ◎ tx_56_by_14 [Agent_14] (50%)  | "Re(Σ m [e^{(i-1)/N}]^m)"
║ ◎ tx_66_by_6  [Agent_6]  (50%)  | Re path + identity in one
║ ◎ tx_76_by_4  [Agent_4]  (50%)  | "Re(z/(1-z)²)" direct
║
╠══════════════════════════════════════════════════════════════════════════════════
║ STEP 3: Closed Form → Laurent Expansion
║ 18 nodes — 1 GP + 17 non-GP (most stuck at closed form, didn't expand)
╠══════════════════════════════════════════════════════════════════════════════════
║
║ ✓ tx_70_by_10 [Agent_10] SETTLED=1.00 | (50%) NO BETS!
║ │  "z₁/(1-z₁)² = 1/((i-1)²ε²) - 1/12 + O(ε²)"
║ │  "similarly z₂/(1-z₂)² = 1/((-i-1)²ε²) - 1/12 + O(ε²)"
║ │  THE CRITICAL STEP: Laurent expansion with -1/12 constant term.
║ │  WARNING: GP node received ZERO bets. Market blind again.
║ │
║ ★ tx_84_by_12 [Agent_12] (50%) NO BETS
║ │  "Real-valued closed form with (r,θ) parametrization"
║ │  INSIGHT: Alternative real representation. Never priced.
║ │
║ ★ tx_60_by_9  [Agent_9]  (50%) NO BETS
║ │  "Re[exp(z)/(exp(z)-1)²]" — alternative form via 1-exp(-z)
║ │  INSIGHT: Equivalent reformulation. Never priced.
║ │
║ STUCK AT CLOSED FORM (correct but didn't reach Laurent expansion):
║ ◎ tx_26_by_10 [Agent_10] (50%)  | "rewrite using Euler's formula" (vague)
║ ◎ tx_36_by_0  [Agent_0]  (50%)  | "Σmx^m = x/(1-x)²" applied
║ ◎ tx_39_by_12 [Agent_12] (50%)  | "z/(1-z)²" (closed form only)
║ ◎ tx_46_by_6  [Agent_6]  P:43.1-50.0% BEAR(20Y/150N) 3 bets ⚠
║ │  "Σm e^{-m(1-i)/N} = e^{-(1-i)/N}/(1-e^{-(1-i)/N})²"
║ │  SHORTED: Agent_8 NO 50, Agent_4 NO 100 (!!), but Agent_0 YES 20
║ │  Market thought this was wrong — but it's CORRECT (just incomplete)!
║ │  MARKET ERROR: Correct node shorted to 43.1%.
║ │
║ ◎ tx_48_by_2  [Agent_2]  (50%)  | "complex exponential closed form" (vague)
║ ◎ tx_51_by_0  [Agent_0]  (50%)  | "rewrite using complex exponential"
║ ◎ tx_67_by_8  [Agent_8]  (50%)  | "z/(1-z)²" (same as others)
║ ◎ tx_74_by_6  [Agent_6]  (50%)  | "r/(1-r)²" (same)
║ ◎ tx_78_by_8  [Agent_8]  (50%)  | "Apply the formula" (too brief)
║ ◎ tx_83_by_10 [Agent_10] (50%)  | "arithmetico-geometric" (same)
║ ◎ tx_85_by_7  [Agent_7]  (50%)  | "z/(1-z)²" (same)
║ ◎ tx_87_by_1  [Agent_1]  (50%)  | "r/(1-r)²" (same)
║ ◎ tx_93_by_11 [Agent_11] (50%)  | "z/(1-z)²" (same)
║ ◎ tx_99_by_6  [Agent_6]  (50%)  | "substitute z in terms of ω" (vague)
║ ◎ tx_100_by_4 [Agent_4]  (50%)  | "exp((i-1)/N)/(1-exp)²" (same)
║ ◎ tx_102_by_14[Agent_14] (50%)  | "r/(1-r)²" (same)
║ ◎ tx_104_by_3 [Agent_3]  (50%)  | "Σm exp(m(i-1)/N) converges" (same)
║
╠══════════════════════════════════════════════════════════════════════════════════
║ STEP 4: Substitute w=(i-1)/N → Re(1/w²)=0 → limit = -1/12 → OMEGA
║ 8 nodes — 1 GP (OMEGA) + 7 non-GP (some independently derived -1/12)
╠══════════════════════════════════════════════════════════════════════════════════
║
║ ✓ tx_112_by_9 [Agent_9]  SETTLED=1.00 | (50%) NO BETS!  ★ OMEGA ★
║ │  "(i-1)²=-2i → 1/((i-1)²)=i/2 → Re=0"
║ │  "S(N) = ½[-1/6+O(ε²)] = -1/12"
║ │  "[COMPLETE] → OMEGA VERIFIED"
║ │  WARNING: THE OMEGA NODE HAD ZERO MARKET INVESTMENT.
║ │  The proof was found entirely through exploration, not market guidance.
║ │
║ ★ tx_91_by_14 [Agent_14] (50%) NO BETS
║ │  "e^{-ε}/(1-e^{-ε})² = N²/(1-i)² - 1/12 + O(1/N)"
║ │  INSIGHT: Independent Laurent derivation. Same -1/12. Never priced.
║ │
║ ★ tx_92_by_13 [Agent_13] (50%) NO BETS
║ │  "a/(1-a)² = N²/(i-1)² - 1/12 + O(1/N)"
║ │  INSIGHT: Independent Laurent. Same result. Never priced.
║ │
║ ★ tx_103_by_0 [Agent_0]  (50%) NO BETS
║ │  "1/w² - 1/12 + O(w²)"
║ │  INSIGHT: Compact Laurent form. Never priced.
║ │
║ ★ tx_109_by_2 [Agent_2]  (50%) NO BETS
║ │  "f(z) = 1/z² - 1/12 + z/12 + O(z²)"
║ │  INSIGHT: Extended Laurent (extra term z/12). Never priced.
║ │
║ △ tx_65_by_2  [Agent_2]  (50%) NO BETS
║ │  "Apply summation formula... with r₁,r₂" (setup for Step 4, incomplete)
║ △ tx_73_by_11 [Agent_11] (50%) NO BETS
║ │  "Taylor expansion of e^{-(1-i)ε}" (concrete computation, incomplete)
║ △ tx_101_by_10[Agent_10] (50%) NO BETS
║ │  "Σ m z^m for z₁,z₂" (Step 4 setup)
║ △ tx_105_by_8 [Agent_8]  (50%) NO BETS
║ │  "expand e^{-w} and 1-e^{-w}" (Taylor details)
║
╠══════════════════════════════════════════════════════════════════════════════════
║ HEAVILY TRADED NON-GP NODES (market active but ultimately Price=0)
╠══════════════════════════════════════════════════════════════════════════════════
║
║ ⚠ tx_5_by_14  [Agent_14] (50%) → Agent_6 YES 2000 → P:90.0%!
║ │  "Define S(N)... cos=(e+e⁻)/2... S(N) = ½Σ m[...]"
║ │  Step 1+2 combined. Correct math but NOT selected for GP.
║ │  Agent_6 bet 20% of genesis funds (2000 Coins). LOST EVERYTHING.
║ │  BIGGEST SINGLE BET IN ENTIRE RUN. Post-settlement: 0.00.
║ │
║ ◎ tx_6_by_6   [Agent_6]  → Agent_10 YES 100 → P:54.8%
║ │  Step 1, correct but duplicate. Heavy endorsement.
║ │
║ ◎ tx_13_by_3  [Agent_3]  → Agent_0/4/6 YES 10+10+20 → P:52.0%
║ │  Step 1, correct but duplicate. Triple endorsement.
║ │
║ SHORTED NODES:
║ ◎ tx_1_by_2   [Agent_2]  P:43.5-50.0% BEAR(0Y/140N) | Step 1 (shorted!)
║ ◎ tx_20_by_11 [Agent_11] P:45.2-50.0% BEAR(0Y/100N) | Step 1 (shorted!)
║ ◎ tx_21_by_8  [Agent_8]  P:42.2-50.0% BEAR(0Y/170N) | Step 2 Re path (shorted!)
║ ◎ tx_40_by_0  [Agent_0]  P:41.0-50.0% BEAR(0Y/200N) | Step 2 (MOST SHORTED)
║ │  MARKET ERROR: tx_40 is mathematically CORRECT but shorted to 41%.
║ │  Why? Probably because agents couldn't distinguish correct-but-redundant
║ │  from incorrect. Shorting = "I think this is a dead branch",
║ │  not "I found a mathematical error."
║ │
║ ◎ tx_4_by_8   [Agent_8]  P:47.6% BEAR(0Y/50N)  | Step 1 duplicate
║ ◎ tx_10_by_2  [Agent_2]  P:47.6% BEAR(0Y/50N)  | Step 1 duplicate
║ ◎ tx_12_by_9  [Agent_9]  P:47.6% BEAR(0Y/50N)  | Step 2 duplicate
║ ◎ tx_55_by_9  [Agent_9]  P:48.0% BEAR(10Y/50N) | Step 3 duplicate
║ ◎ tx_69_by_11 [Agent_11] P:48.0% BEAR(0Y/40N)  | Step 3 duplicate
║ ◎ tx_77_by_6  [Agent_6]  P:47.6% BEAR(0Y/50N)  | shorted
║ ◎ tx_89_by_14 [Agent_14] P:47.6% BEAR(0Y/50N)  | shorted
║ │
║ ◎ tx_91_by_8  [Agent_8]  → Agent_8 AUTO-LONG 100 → P:54.8%
║     Agent_8's own Step 4 attempt. Self-invested 100 Coins. Lost.
║
╚══════════════════════════════════════════════════════════════════════════════════
```

### Node Classification Summary

```
Category          Nodes  %     Market Reaction            Correct?
────────────────  ─────  ────  ─────────────────────────  ─────────
GP STEP 1 ✓         1    2%   52.0% (2 YES bets)         ✓ OMEGA
GP STEP 2 ✓         1    2%   (50%) NO BETS              ✓ OMEGA
GP STEP 3 ✓         1    2%   (50%) NO BETS              ✓ OMEGA
GP STEP 4 ✓         1    2%   (50%) NO BETS              ✓ OMEGA
INSIGHT ★          10   16%   ALL (50%) NO BETS          ✓ correct but ignored
DUPLICATE ◎        42   69%   Mix: 50% / 41-48% BEAR     ✓ correct but redundant
INCOMPLETE △        4    7%   ALL (50%) NO BETS          ✓ partial
WHALE ⚠             1    2%   90.0% → settled 0.00       ✓ correct but dead
ERROR ✗             0    0%   N/A                        (no errors in run!)
────────────────  ─────  ────
TOTAL              61   100%
```

### Price Spectrum (all 61 nodes)

```
Price Band         Nodes   Key Examples                         Signal Quality
─────────────────  ─────   ──────────────────────────────────   ──────────────
90.0% WHALE         1      tx_5: Agent_6 bet 2000C             ✗ WRONG SIGNAL
                           (correct math, but not GP → lost)    (whale noise)

54.8% STRONG YES    2      tx_6, tx_91_by_8                    NOISY
                           (correct Step 1 / Step 4 attempts)   (endorsing duplicates)

51.0-52.0%          5      tx_7(GP!), tx_13, tx_16, tx_44, tx_51  MIXED
                           (GP + duplicates get same price)     (can't tell GP from dup)

50.0% FLAT         34      ALL 4 GP nodes except tx_7!          ✗✗ TERRIBLE
                           ALL 10 insight nodes!                 (market completely blind
                           Most duplicates.                      to the actual proof)

47.6-48.0% BEAR     7      tx_4,10,12,55,69,77,89              NOISY
                           (all correct, just redundant)         (shorting correct math)

43.5-45.2% BEAR     3      tx_1,20,21                          NOISY
                           (all correct, just redundant)         (aggressive mispricing)

41.0% MOST SHORT    1      tx_40: Agent_0 Step 2                ✗ MARKET ERROR
                           (correct math, 200 NO Coins!)         (killed correct node)
```

### Market Effectiveness Scorecard

```
Detection Type                      Detected?   Price Signal       Score
──────────────────────────────────  ─────────   ────────────       ─────
GP Step 1 (tx_7)                     YES        52.0% (mild YES)    6/10
GP Step 2 (tx_24)                    NO         50.0% (invisible)   0/10
GP Step 3 (tx_70)                    NO         50.0% (invisible)   0/10
GP Step 4/OMEGA (tx_112)             NO         50.0% (invisible)   0/10
Conjugate equivalence (tx_57)        NO         50.0% (invisible)   0/10
Independent Laurent (tx_91,92,103)   NO         ALL 50% (invisible) 0/10
Duplicate detection                  PARTIAL    41-48% (some shorted) 3/10
Error detection                      N/A        (no errors in run)  N/A
Whale noise (tx_5: 2000C)           NO         90% bubble (wrong!)  0/10
Correct node killed (tx_40)          N/A        41% (market error)  0/10
──────────────────────────────────────────────────────────────────────────
OVERALL MARKET EFFECTIVENESS:                                      1/10

3 out of 4 GP nodes had ZERO market activity.
ALL 10 insight nodes had ZERO market activity.
The proof was found by pure exploration, NOT by market guidance.
The market's only contribution: mildly endorsed GP Step 1 (52%).
The market's worst failure: pumped dead node to 90% (whale), killed correct node to 41%.
```

### Comparison: Zeta Market (1/10) vs AIME P15 Market (5/10)

```
                           Zeta (OMEGA reached)    AIME P15 (failed)
                           ────────────────────    ─────────────────
Market overall score:      1/10                    5/10
Killing errors:            N/A (no errors!)        10/10 (killed 7/9)
Endorsing GP/insight:      0/10 (3/4 GP = 50%)     8/10 (tx_615 → 60%)
Duplicate handling:        3/10 (some shorted)      0/10 (all at 50%)
Biggest market mistake:    Whale 2000C → 90%        None (all bets reasonable)
GP discovery mechanism:    Pure exploration          Pure exploration
Market's actual role:      NONE (proof found         Pruning only (killed errors,
                           without market help)      but couldn't guide to answer)
```

**Paradox**: The market was WORSE on the easier problem (zeta: 1/10) than the harder one (AIME: 5/10). Why?
- Zeta had NO mathematical errors to catch → the market's best skill (killing errors) was useless
- Zeta's market suffered from **whale noise** (2000C bet on wrong node) that didn't occur in AIME
- AIME generated genuine errors (tx_552 "486²", tx_700) that gave the market something useful to do
- **The market adds value only when there are errors to catch**. On a clean run, it's pure overhead.
