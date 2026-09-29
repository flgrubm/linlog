From NanoYalla Require Import macroll.

Lemma certificate (A B : formula) : ll [wn (aplus (dual A) (dual B)); tens (oc A) (oc B)].
Proof.
apply (co_r_ext []); cbn_sequent.
apply (ex_perm_r [0; 2; 1] [wn (aplus (dual A) (dual B)); tens (oc A) (oc B); wn (aplus (dual A) (dual B))]).
apply (tens_r_ext [wn (aplus (dual A) (dual B))]); cbn_sequent.
{
  apply (oc_r_ext [aplus (dual A) (dual B)] (A) []); cbn_sequent.
  apply (de_r_ext []); cbn_sequent.
  apply (plus_r1_ext []); cbn_sequent.
  ax_expansion.
}
{
  apply (oc_r_ext [] (B) [aplus (dual A) (dual B)]); cbn_sequent.
  apply (de_r_ext [B]); cbn_sequent.
  apply (plus_r2_ext [B]); cbn_sequent.
  ax_expansion.
}
Qed.
