// Copyright 2016 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! Data structures representative of various [procfs][procfs-url] reports.
//!
//! [procfs-url]: https://github.com/torvalds/linux/blob/master/Documentation/filesystems/proc.txt


extern crate rustc_serialize;
#[macro_use] extern crate enum_primitive;
extern crate num;
use num::FromPrimitive;


use std::default::Default;
use std::process::Command;
use std::io::{self, ErrorKind};
use std::os::unix::process::ExitStatusExt;
use std::net::Ipv4Addr;
use std::fs::File;
use std::io::Read;
use rustc_serialize::hex::FromHex;


/// Represents the output of `cat /proc/stat`
#[derive(Clone, RustcDecodable, RustcEncodable)]
pub struct Stat {
    pub cpu: Vec<u64>,
    pub cpus: Vec<Vec<u64>>,
    pub intr: Vec<u64>,
    pub ctxt: u64,
    pub btime: u32,
    pub processes: u32,
    pub procs_running: u32,
    pub procs_blocked: u32,
    pub softirq: Vec<u64>
}

/// Represents the output of `cat /proc/meminfo`
#[derive(Clone, RustcDecodable, RustcEncodable)]
pub struct MemInfo {
    pub mem_total: u64,
    pub mem_free: u64,
    pub mem_available: u64,
    pub bufers: u64,
    pub cached: u64,
    pub swap_cached: u64,
    pub active: u64,
    pub inactive: u64,
    pub active_anon: u64,
    pub inactive_anon: u64,
    pub active_file: u64,
    pub inactive_file: u64,
    pub unevictable: u64,
    pub mlocked: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub dirty: u64,
    pub writeback: u64,
    pub anon_pages: u64,
    pub mapped: u64,
    pub shmem: u64,
    pub slab: u64,
    pub s_reclaimable: u64,
    pub s_unreclaim: u64,
    pub kernel_stack: u64,
    pub page_tables: u64,
    pub nfs_unstable: u64,
    pub bounce: u64,
    pub writeback_tmp: u64,
    pub commit_limit: u64,
    pub committed_as: u64,
    pub vmalloc_total: u64,
    pub vmalloc_used: u64,
    pub vmalloc_chunk: u64,
    pub hardware_corrupted: u64,
    pub anon_huge_pages: u64,
    pub cma_total: u64,
    pub cma_free: u64,
    pub huge_pages_total: u64,
    pub huge_pages_free: u64,
    pub huge_pages_rsvd: u64,
    pub huge_pages_surp: u64,
    pub hugepagesize: u64,
    pub direct_map_4k: u64,
    pub direct_map_2m: u64
}


enum_from_primitive! {
    /// Represents TCP socket's state.
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub enum SocketState {
        Established = 1,
        SynSent = 2,
        SynRecv = 3,
        FinWait1 = 4,
        FinWait2 = 5,
        TimeWait = 6,
        Close = 7,
        CloseWait = 8,
        LastAck = 9,
        Listen = 10,
        Closing = 11
    }
}

/// Represents TCP socket's timer status.
#[derive(Clone, Debug, PartialEq)]
pub enum SocketTimerState {
    // TODO: other timer states, timeout
    Inactive,
    Active
}

/// Represents a line (socket) in output of `cat /proc/net/{tcp,udp}`
#[derive(Clone)]
pub struct Socket {
    pub sl: u64,
    pub local_address: Ipv4Addr,
    pub remote_address: Ipv4Addr,
    pub state: SocketState,
    pub tx_queue: u64,
    pub rx_queue: u64,
    pub timer: SocketTimerState,
    pub uid: u32,
    pub inode: u64
}

pub fn stat() -> io::Result<Stat> {
    let output_result = Command::new("cat")
        .arg("/proc/stat")
        .output();

    if output_result.is_err() {
        let err = output_result.unwrap_err();
        return Err(err);
    }

    let output = output_result.unwrap();
    if !output.status.success() {
        let code = match output.status.code() {
            Some(c) => c,
            None => output.status.signal().unwrap()
        };

        let stderr = unsafe { String::from_utf8_unchecked(output.stderr) };
        let err = io::Error::new(ErrorKind::Other, format!("ExitStatus: {}    Stderr: {}",
                                                           code,
                                                           stderr));
        return Err(err);
    }

    // > cat /proc/stat
    //     cpu  2255 34 2290 22625563 6290 127 456 0 0 0
    //     cpu0 1132 34 1441 11311718 3675 127 438 0 0 0
    //     cpu1 1123 0 849 11313845 2614 0 18 0 0 0
    //     intr 114930548 113199788 3 0 5 263 0 4 [... lots more numbers ...]
    //     ctxt 1990473
    //     btime 1062191376
    //     processes 2915
    //     procs_running 1
    //     procs_blocked 0
    //     softirq 183433 0 21755 12 39 1137 231 21459 2263
    let stdout = unsafe { String::from_utf8_unchecked(output.stdout) };
    let lines = stdout.lines();

    let mut stat: Stat = Default::default();
    let mut line_num: usize = 0;
    for ref line in lines {

        if line_num == 0 {
            stat.cpu = to_vecu64(line);
        }

        if line.contains("cpu") && line_num > 0 {
            stat.cpus.push(to_vecu64(line));
        }

        if line.contains("intr") {
            stat.intr = to_vecu64(line);
        }

        if line.contains("ctxt") {
            let mut chunks = line.split_whitespace();
            chunks.next();

            stat.ctxt = chunks.next().unwrap().parse::<u64>().unwrap();
        }

        if line.contains("btime") {
            let mut chunks = line.split_whitespace();
            chunks.next();

            stat.btime = chunks.next().unwrap().parse::<u32>().unwrap();
        }

        if line.contains("processes") {
            let mut chunks = line.split_whitespace();
            chunks.next();

            stat.processes = chunks.next().unwrap().parse::<u32>().unwrap();
        }

        if line.contains("procs_running") {
            let mut chunks = line.split_whitespace();
            chunks.next();

            stat.procs_running = chunks.next().unwrap().parse::<u32>().unwrap();
        }

        if line.contains("procs_blocked") {
            let mut chunks = line.split_whitespace();
            chunks.next();

            stat.procs_blocked = chunks.next().unwrap().parse::<u32>().unwrap();
        }

        if line.contains("softirq") {
            stat.softirq = to_vecu64(line);
        }

        line_num += 1;
    }

    return Ok(stat);
}

pub fn meminfo() -> io::Result<MemInfo> {
    let output_result = Command::new("cat")
        .arg("/proc/meminfo")
        .output();

    if output_result.is_err() {
        let err = output_result.unwrap_err();
        return Err(err);
    }

    let output = output_result.unwrap();
    if !output.status.success() {
        let code = match output.status.code() {
            Some(c) => c,
            None => output.status.signal().unwrap()
        };

        let stderr = unsafe { String::from_utf8_unchecked(output.stderr) };
        let err = io::Error::new(ErrorKind::Other, format!("ExitStatus: {}    Stderr: {}",
                                                           code,
                                                           stderr));
        return Err(err);
    }

    // > cat /proc/meminfo
    //     MemTotal:        3521920 kB
    //     MemFree:         1878240 kB
    //     MemAvailable:    2275916 kB
    //     Buffers:           35428 kB
    //     Cached:           386132 kB
    //     SwapCached:            0 kB
    //     Active:          1229080 kB
    //     Inactive:         284000 kB
    //     Active(anon):    1094728 kB
    //     Inactive(anon):    17664 kB
    //     Active(file):     134352 kB
    //     Inactive(file):   266336 kB
    //     Unevictable:        3660 kB
    //     Mlocked:            3660 kB
    //     SwapTotal:             0 kB
    //     SwapFree:              0 kB
    //     Dirty:                12 kB
    //     Writeback:             0 kB
    //     AnonPages:       1095172 kB
    //     Mapped:            71384 kB
    //     Shmem:             18456 kB
    //     Slab:              50800 kB
    //     SReclaimable:      24684 kB
    //     SUnreclaim:        26116 kB
    //     KernelStack:        5584 kB
    //     PageTables:         6184 kB
    //     NFS_Unstable:          0 kB
    //     Bounce:                0 kB
    //     WritebackTmp:          0 kB
    //     CommitLimit:     1760960 kB
    //     Committed_AS:    2064016 kB
    //     VmallocTotal:   34359738367 kB
    //     VmallocUsed:           0 kB
    //     VmallocChunk:          0 kB
    //     HardwareCorrupted:     0 kB
    //     AnonHugePages:   1013760 kB
    //     CmaTotal:              0 kB
    //     CmaFree:               0 kB
    //     HugePages_Total:       0
    //     HugePages_Free:        0
    //     HugePages_Rsvd:        0
    //     HugePages_Surp:        0
    //     Hugepagesize:       2048 kB
    //     DirectMap4k:       67520 kB
    //     DirectMap2M:     3602432 kB
    let stdout = unsafe { String::from_utf8_unchecked(output.stdout) };
    let lines = stdout.lines();

    let mut meminfo: MemInfo = Default::default();
    for ref line in lines {

        if line.contains("MemTotal") {
            meminfo.mem_total = to_u64(line);
        }

        if line.contains("MemFree") {
            meminfo.mem_free = to_u64(line);
        }

        if line.contains("MemAvailable") {
            meminfo.mem_available = to_u64(line);
        }

        if line.contains("Buffers") {
            meminfo.bufers = to_u64(line);
        }

        if line.contains("Cached") {
            meminfo.cached = to_u64(line);
        }

        if line.contains("SwapCached") {
            meminfo.swap_cached = to_u64(line);
        }

        if line.contains("Active") {
            meminfo.active = to_u64(line);
        }

        if line.contains("Inactive") {
            meminfo.inactive = to_u64(line);
        }

        if line.contains("Active(anon)") {
            meminfo.active_anon = to_u64(line);
        }

        if line.contains("Inactive(anon)") {
            meminfo.inactive_anon = to_u64(line);
        }

        if line.contains("Active(file)") {
            meminfo.active_file = to_u64(line);
        }

        if line.contains("Inactive(file)") {
            meminfo.inactive_file = to_u64(line);
        }

        if line.contains("Unevictable") {
            meminfo.unevictable = to_u64(line);
        }

        if line.contains("Mlocked") {
            meminfo.mlocked = to_u64(line);
        }

        if line.contains("SwapTotal") {
            meminfo.swap_total = to_u64(line);
        }

        if line.contains("SwapFree") {
            meminfo.swap_free = to_u64(line);
        }

        if line.contains("Dirty") {
            meminfo.dirty = to_u64(line);
        }

        if line.contains("Writeback") {
            meminfo.writeback = to_u64(line);
        }

        if line.contains("AnonPages") {
            meminfo.anon_pages = to_u64(line);
        }

        if line.contains("Mapped") {
            meminfo.mapped = to_u64(line);
        }

        if line.contains("Shmem") {
            meminfo.shmem = to_u64(line);
        }

        if line.contains("Slab") {
            meminfo.slab = to_u64(line);
        }

        if line.contains("SReclaimable") {
            meminfo.s_reclaimable = to_u64(line);
        }

        if line.contains("SUnreclaim") {
            meminfo.s_unreclaim = to_u64(line);
        }

        if line.contains("KernelStack") {
            meminfo.kernel_stack = to_u64(line);
        }

        if line.contains("PageTables") {
            meminfo.page_tables = to_u64(line);
        }

        if line.contains("NFS_Unstable") {
            meminfo.nfs_unstable = to_u64(line);
        }

        if line.contains("Bounce") {
            meminfo.bounce = to_u64(line);
        }

        if line.contains("WritebackTmp") {
            meminfo.writeback_tmp = to_u64(line);
        }

        if line.contains("CommitLimit") {
            meminfo.commit_limit = to_u64(line);
        }

        if line.contains("Committed_AS") {
            meminfo.committed_as = to_u64(line);
        }

        if line.contains("VmallocTotal") {
            meminfo.vmalloc_total = to_u64(line);
        }

        if line.contains("VmallocUsed") {
            meminfo.vmalloc_used = to_u64(line);
        }

        if line.contains("VmallocChunk") {
            meminfo.vmalloc_chunk = to_u64(line);
        }

        if line.contains("HardwareCorrupted") {
            meminfo.hardware_corrupted = to_u64(line);
        }

        if line.contains("AnonHugePages") {
            meminfo.anon_huge_pages = to_u64(line);
        }

        if line.contains("CmaTotal") {
            meminfo.cma_total = to_u64(line);
        }

        if line.contains("CmaFree") {
            meminfo.cma_free = to_u64(line);
        }

        if line.contains("HugePages_Total") {
            meminfo.huge_pages_total = to_u64(line);
        }

        if line.contains("HugePages_Free") {
            meminfo.huge_pages_free = to_u64(line);
        }

        if line.contains("HugePages_Rsvd") {
            meminfo.huge_pages_rsvd = to_u64(line);
        }

        if line.contains("HugePages_Surp") {
            meminfo.huge_pages_surp = to_u64(line);
        }

        if line.contains("Hugepagesize") {
            meminfo.hugepagesize = to_u64(line);
        }

        if line.contains("DirectMap4k") {
            meminfo.direct_map_4k = to_u64(line);
        }

        if line.contains("DirectMap2M") {
            meminfo.direct_map_2m = to_u64(line);
        }
    }

    Ok(meminfo)
}

pub fn tcp() -> io::Result<Vec<Socket>> {
    net("/proc/net/tcp")
}

pub fn udp() -> io::Result<Vec<Socket>> {
    net("/proc/net/udp")
}

fn read_file(path: &str) -> io::Result<String> {
    let file = File::open(path);
    let mut content = String::new();

    file
        .map(|mut f| f.read_to_string(&mut content))
        .and(Ok(content))
}

fn net(file: &str) -> io::Result<Vec<Socket>> {
    let content = read_file(file);
    match content {
        Ok(c) => Ok(c.lines().skip(1).map(to_net_socket).collect()),
        Err(e) => Err(e)
    }
}

fn to_vecu64(line: &str) -> Vec<u64> {
    let mut chunks = line.split_whitespace();
    let mut buf = Vec::<u64>::new();

    // First chunk is always a non-number, descriptive text.
    chunks.next();

    while let Some(chunk) = chunks.next() {
        buf.push(chunk.parse::<u64>().unwrap());
    }

    return buf;
}

fn to_u64(line: &str) -> u64 {
    let mut chunks = line.split_whitespace();
    chunks.next();

    return chunks.next().unwrap().parse::<u64>().unwrap();
}

fn to_net_socket(line: &str) -> Socket {
    let mut chunks = line.split_whitespace();
    let sl = chunks.next().unwrap().split(':').next().unwrap().parse::<u64>().unwrap();

    // Both local and remote addresses are formatted as <host>:<port> pair, so
    // split them further.
    // TODO ports
    let local : Vec<&str> = chunks.next().unwrap().split(':').collect();
    let remote : Vec<&str> = chunks.next().unwrap().split(':').collect();
    let state = chunks.next().unwrap().from_hex().unwrap()[0];
    let queues : Vec<&str> = chunks.next().unwrap().split(':').collect();
    let timer : Vec<&str> = chunks.next().unwrap().split(':').collect();
    // retrnsmt - unused
    chunks.next().unwrap();
    let uid = chunks.next().unwrap().parse::<u32>().unwrap();
    // timeout - unused
    chunks.next().unwrap();
    let inode = chunks.next().unwrap().parse::<u64>().unwrap();

    Socket {
        sl: sl,
        local_address: to_ipaddr(local[0]),
        remote_address: to_ipaddr(remote[0]),
        state: SocketState::from_u8(state).unwrap(),
        tx_queue: queues[0].parse::<u64>().unwrap(),
        rx_queue: queues[1].parse::<u64>().unwrap(),
        timer: match timer[0].parse::<u8>().unwrap() {
            0 => SocketTimerState::Inactive,
            _ => SocketTimerState::Active
        },
        uid: uid,
        inode: inode
    }
}

fn to_ipaddr(hex: &str) -> Ipv4Addr {
    let bytes = hex.from_hex().unwrap();
    Ipv4Addr::from([ bytes[3], bytes[2], bytes[1], bytes[0] ])
}

impl Default for Stat {
    fn default() -> Stat {
        Stat {
            cpu: Vec::new(),
            cpus: Vec::new(),
            intr: Vec::new(),
            ctxt: 0,
            btime: 0,
            processes: 0,
            procs_running: 0,
            procs_blocked: 0,
            softirq: Vec::new()
        }
    }
}

impl Default for MemInfo {
    fn default() -> MemInfo {
        MemInfo {
            mem_total: 0,
            mem_free: 0,
            mem_available: 0,
            bufers: 0,
            cached: 0,
            swap_cached: 0,
            active: 0,
            inactive: 0,
            active_anon: 0,
            inactive_anon: 0,
            active_file: 0,
            inactive_file: 0,
            unevictable: 0,
            mlocked: 0,
            swap_total: 0,
            swap_free: 0,
            dirty: 0,
            writeback: 0,
            anon_pages: 0,
            mapped: 0,
            shmem: 0,
            slab: 0,
            s_reclaimable: 0,
            s_unreclaim: 0,
            kernel_stack: 0,
            page_tables: 0,
            nfs_unstable: 0,
            bounce: 0,
            writeback_tmp: 0,
            commit_limit: 0,
            committed_as: 0,
            vmalloc_total: 0,
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
            hugepagesize: 0,
            direct_map_4k: 0,
            direct_map_2m: 0
        }
    }
}

#[test]
fn test_to_ipaddr() {
    let addr = to_ipaddr("0100007F");
    assert_eq!(addr.octets(), [127, 0, 0, 1]);
}

#[test]
fn test_to_net_socket() {
    let sock = to_net_socket("  49: 0100007F:1111 5B41EE2E:50 0A 00000001:00000002 00:00000000 00000000  1001        0 2796814 1 ffff938ed0741080 20 4 29 10 -1");
    assert_eq!(sock.local_address.octets(), [127, 0, 0, 1]);
    assert_eq!(sock.remote_address.octets(), [46, 238, 65, 91]);
    assert_eq!(sock.state, SocketState::Listen);
    assert_eq!(sock.tx_queue, 1);
    assert_eq!(sock.rx_queue, 2);
    assert_eq!(sock.timer, SocketTimerState::Inactive);
    assert_eq!(sock.uid, 1001);
    assert_eq!(sock.inode, 2796814);
}
