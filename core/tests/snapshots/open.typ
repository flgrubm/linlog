#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $⊸ upright(L)$,
    rule(name: $"ax"$, $A ⊢ A$),
    grid(align: center, row-gutter: 0.4em, $dots.v$, $B ⊢ B$),
    $A, A ⊸ B ⊢ B$,
  ),
)
