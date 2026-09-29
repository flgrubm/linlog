#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)
#show math.equation: set text(font: "Euler Math")

#prooftree(
  rule(
    name: $class("normal", ?) upright(w)$,
    rule(
      name: $!$,
      rule(
        name: $class("normal", ?) upright(c)$,
        rule(
          name: $⊗$,
          rule(
            name: $class("normal", ?) upright(d)$,
            rule(name: $"ax"$, $⊢ A^⊥, A$),
            $⊢ class("normal", ?)A^⊥, A$,
          ),
          rule(
            name: $class("normal", ?) upright(d)$,
            rule(name: $"ax"$, $⊢ A^⊥, A$),
            $⊢ class("normal", ?)A^⊥, A$,
          ),
          $⊢ class("normal", ?)A^⊥, class("normal", ?)A^⊥, A ⊗ A$,
        ),
        $⊢ class("normal", ?)A^⊥, A ⊗ A$,
      ),
      $⊢ class("normal", ?)A^⊥, !(A ⊗ A)$,
    ),
    $⊢ class("normal", ?)A^⊥, class("normal", ?)B^⊥, !(A ⊗ A)$,
  ),
)
