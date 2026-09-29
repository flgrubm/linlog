From NanoYalla Require Import macroll.

Lemma certificate (A B C : formula) : ll [bot; aplus (dual A) (dual B); tens B (dual C); C].
Proof.
apply (bot_r_ext []); cbn_sequent.
apply (plus_r2_ext []); cbn_sequent.
apply (tens_r_ext [dual B]); cbn_sequent.
{
  ax_expansion.
}
{
  ax_expansion.
}
Qed.
