extern crate linux_stats;

use linux_stats::MemInfo;

const MEMINFO_1: MemInfo = MemInfo {
    mem_total: 3521920,
    mem_free: 1878240,
    mem_available: 2275916,
    bufers: 35428,
    cached: 386132,
    swap_cached: 0,
    active: 134352,
    inactive: 266336,
    active_anon: 1094728,
    inactive_anon: 17664,
    active_file: 134352,
    inactive_file: 266336,
    unevictable: 3660,
    mlocked: 3660,
    swap_total: 0,
    swap_free: 0,
    dirty: 12,
    writeback: 0,
    anon_pages: 1095172,
    mapped: 71384,
    shmem: 18456,
    slab: 50800,
    s_reclaimable: 24684,
    s_unreclaim: 26116,
    kernel_stack: 5584,
    page_tables: 6184,
    nfs_unstable: 0,
    bounce: 0,
    writeback_tmp: 0,
    commit_limit: 1760960,
    committed_as: 2064016,
    vmalloc_total: 34359738367,
    vmalloc_used: 0,
    vmalloc_chunk: 0,
    hardware_corrupted: 0,
    anon_huge_pages: 1013760,
    cma_total: 0,
    cma_free: 0,
    huge_pages_total: 0,
    huge_pages_free: 0,
    huge_pages_rsvd: 0,
    huge_pages_surp: 0,
    hugepagesize: 2048,
    direct_map_4k: 67520,
    direct_map_2m: 3602432,
};

const MEMINFO_2: MemInfo = MemInfo {
    mem_total: 32828552,
    mem_free: 12195628,
    mem_available: 13725248,
    bufers: 185048,
    cached: 1876616,
    swap_cached: 0,
    active: 806832,
    inactive: 1015204,
    active_anon: 1531372,
    inactive_anon: 105576,
    active_file: 806832,
    inactive_file: 1015204,
    unevictable: 132464,
    mlocked: 0,
    swap_total: 4194280,
    swap_free: 4194280,
    dirty: 224,
    writeback: 0,
    anon_pages: 1529596,
    mapped: 16887024,
    shmem: 0,
    slab: 354316,
    s_reclaimable: 155152,
    s_unreclaim: 199164,
    kernel_stack: 8912,
    page_tables: 47852,
    nfs_unstable: 0,
    bounce: 0,
    writeback_tmp: 0,
    commit_limit: 20608556,
    committed_as: 20066912,
    vmalloc_total: 34359738367,
    vmalloc_used: 0,
    vmalloc_chunk: 0,
    hardware_corrupted: 0,
    anon_huge_pages: 0,
    cma_total: 0,
    cma_free: 0,
    huge_pages_total: 0,
    huge_pages_free: 0,
    huge_pages_rsvd: 0,
    huge_pages_surp: 0,
    hugepagesize: 2048,
    direct_map_4k: 215212,
    direct_map_2m: 8062976,
};

const MEMINFO_1_RAW: &'static str = include_str!("./meminfo-1");
const MEMINFO_2_RAW: &'static str = include_str!("./meminfo-2");

#[test]
fn meminfo_empty() {
    assert_eq!("".parse::<MemInfo>().unwrap(), Default::default());
}

#[test]
fn meminfo_1() {
    assert_eq!(MEMINFO_1_RAW.parse::<MemInfo>().unwrap(), MEMINFO_1);
}

#[test]
fn meminfo_2() {
    assert_eq!(MEMINFO_2_RAW.parse::<MemInfo>().unwrap(), MEMINFO_2);
}
