///! K1: the short age text.
// test-tag: critical
use argus_core::clock::age_text;

#[test]
fn ages_read_short() {
    assert_eq!(age_text(10 * 60_000), "10m");
    assert_eq!(age_text(0), "1m");
    assert_eq!(age_text(60 * 60_000), "1h");
    assert_eq!(age_text(47 * 3_600_000), "47h");
    assert_eq!(age_text(3 * 86_400_000), "3d");
}
