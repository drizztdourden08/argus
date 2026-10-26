//! The injected clock: the wall clock reads real time, the manual one moves only when told to.

use argus_core::clock::{Clock, ManualClock, SystemClock, age_text, now_ms};

#[test]
fn the_manual_clock_moves_when_told() {
    let clock = ManualClock::at(10);
    clock.advance(5);
    assert_eq!(clock.now_ms(), 15);
    clock.set(2);
    assert_eq!(clock.now_ms(), 2);
}

#[test]
fn clones_of_a_manual_clock_share_one_time() {
    let clock = ManualClock::at(1);
    let twin = clock.clone();
    twin.advance(9);
    assert_eq!(clock.now_ms(), 10);
}

#[test]
fn the_system_clock_reads_the_wall_clock() {
    let before = now_ms();
    assert!(SystemClock.now_ms() >= before);
}

#[test]
fn ages_read_short() {
    assert_eq!(age_text(10 * 60_000), "10m");
    assert_eq!(age_text(0), "1m");
    assert_eq!(age_text(60 * 60_000), "1h");
    assert_eq!(age_text(47 * 3_600_000), "47h");
    assert_eq!(age_text(3 * 86_400_000), "3d");
}
