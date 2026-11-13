//! Integration tests of `argus migrate board-ids` on a fixture home in a temp folder. The real
//! `~/.coord` is never touched.

mod fixture;
mod t1_round_trip;
mod t2_counts;
mod t3_interrupted;
mod t4_backup;
mod t5_guards;
mod t6_review_fixes;
