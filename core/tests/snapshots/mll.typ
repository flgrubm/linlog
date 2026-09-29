#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $⅋$,
    rule(
      name: $⊗$,
      rule(name: $"ax"$, $⊢ B^⊥, B$),
      rule(name: $"ax"$, $⊢ A^⊥, A$),
      $⊢ A^⊥, B^⊥, B ⊗ A$,
    ),
    $⊢ A^⊥ ⅋ B^⊥, B ⊗ A$,
  ),
)
