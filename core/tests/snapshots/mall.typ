#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $\&$,
    rule(
      name: $⊥$,
      rule(
        name: $⊕_2$,
        rule(name: $"ax"$, $⊢ A^⊥, A$),
        $⊢ A^⊥, B ⊕ A$,
      ),
      $⊢ A^⊥, ⊥, B ⊕ A$,
    ),
    rule(
      name: $⊥$,
      rule(
        name: $⊕_1$,
        rule(name: $"ax"$, $⊢ B^⊥, B$),
        $⊢ B^⊥, B ⊕ A$,
      ),
      $⊢ B^⊥, ⊥, B ⊕ A$,
    ),
    $⊢ A^⊥ class("binary", \&) B^⊥, ⊥, B ⊕ A$,
  ),
)
