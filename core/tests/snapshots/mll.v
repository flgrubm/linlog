From NanoYalla Require Import macroll.

Lemma certificate (A B : formula) : ll [parr (dual A) (dual B); tens B A].
Proof.
apply (parr_r_ext []); cbn_sequent.
apply (ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]).
apply (tens_r_ext [dual B]); cbn_sequent.
{
  ax_expansion.
}
{
  ax_expansion.
}
Qed.
