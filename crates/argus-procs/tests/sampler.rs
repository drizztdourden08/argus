//! E2: One real sample of this machine: it only reads, and checks the row of this test's own process.
// test-tag: e2e

use argus_procs::{Killer, Sampler, SystemKiller, SystemSampler, platform_name};

#[test]
#[ignore = "e2e"]
fn a_real_sample_has_this_process_with_a_plausible_shape() {
    let sample = SystemSampler.sample().unwrap();
    assert_eq!(sample.platform, platform_name());
    assert!(sample.at > 0);
    assert!(sample.procs.len() > 3, "more than a handful of processes");
    let me = sample
        .procs
        .iter()
        .find(|p| p.pid == std::process::id())
        .expect("this test process is in the sample");
    assert!(me.name.to_lowercase().starts_with("sampler"), "{}", me.name);
    assert!(me.ppid > 0);
    assert!(me.mem_bytes > 1024 * 1024, "some memory");
    assert!(me.start_ms <= sample.at + 1000 && sample.at - me.start_ms < 3_600_000);
    assert!(me.cpu_ms >= 0);
    assert_eq!(me.responding, None);
    assert!(me.cmd.to_lowercase().contains("sampler"), "{}", me.cmd);
}

#[test]
#[ignore = "e2e"]
fn the_killer_sees_this_process_alive_and_a_missing_one_gone() {
    assert!(SystemKiller.is_alive(std::process::id()));
    assert!(!SystemKiller.is_alive(u32::MAX - 1));
}
