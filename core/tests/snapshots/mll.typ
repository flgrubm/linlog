#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)
#show math.equation: set text(font: "Euler Math")

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
