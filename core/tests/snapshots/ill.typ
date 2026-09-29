#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)
#show math.equation: set text(font: "Euler Math")

#prooftree(
  rule(
    name: $bold(1) upright(L)$,
    rule(
      name: $\& upright(L)_2$,
      rule(
        name: $⊸ upright(L)$,
        rule(name: $"ax"$, $B ⊢ B$),
        rule(name: $"ax"$, $C ⊢ C$),
        $B, B ⊸ C ⊢ C$,
      ),
      $A class("binary", \&) B, B ⊸ C ⊢ C$,
    ),
    $bold(1), A class("binary", \&) B, B ⊸ C ⊢ C$,
  ),
)
