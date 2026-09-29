From NanoYalla Require Import macroll.

Lemma certificate (A B : formula) : ll [awith (dual A) (dual B); bot; aplus B A].
Proof.
apply (with_r_ext []); cbn_sequent.
{
  apply (bot_r_ext [dual A]); cbn_sequent.
  apply (plus_r2_ext [dual A]); cbn_sequent.
  ax_expansion.
}
{
  apply (bot_r_ext [dual B]); cbn_sequent.
  apply (plus_r1_ext [dual B]); cbn_sequent.
  ax_expansion.
}
Qed.
