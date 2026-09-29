From NanoYalla Require Import macroll.

Lemma certificate (A B : formula) : ll [wn (dual A); wn (dual B); oc (tens A A)].
Proof.
apply (wk_r_ext [wn (dual A)]); cbn_sequent.
apply (oc_r_ext [dual A] (tens A A) []); cbn_sequent.
apply (co_r_ext []); cbn_sequent.
apply (ex_perm_r [0; 2; 1] [wn (dual A); tens A A; wn (dual A)]).
apply (tens_r_ext [wn (dual A)]); cbn_sequent.
{
  apply (de_r_ext []); cbn_sequent.
  ax_expansion.
}
{
  apply (de_r_ext [A]); cbn_sequent.
  ax_expansion.
}
Qed.
