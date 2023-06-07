// Copyright 2016 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.

//! Data structures representative of various [procfs][procfs-url] reports.
//!
//! [procfs-url]: https://github.com/torvalds/linux/blob/master/Documentation/filesystems/proc.txt

#[macro_use]
extern crate enum_primitive;
extern crate hex;
extern crate num;

use hex::FromHex;
use num::FromPrimitive;

use std::convert::Infallible;
use std::default::Default;
use std::fs::File;
use std::io;
use std::io::Read;
use std::net::Ipv4Addr;
use std::str::FromStr;

/// Represents the output of `cat /proc/stat`
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Stat {
    pub cpu: Vec<u64>,
    pub cpus: Vec<Vec<u64>>,
    pub intr: Vec<u64>,
    pub ctxt: u64,
    pub btime: u32,
    pub processes: u32,
    pub procs_running: u32,
    pub procs_blocked: u32,
    pub softirq: Vec<u64>,
}

impl FromStr for Stat {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Stat, Infallible> {
        let mut stat: Stat = Default::default();
        for (line_num, line) in s.lines().enumerate() {
            if line_num == 0 {
                stat.cpu = to_vecu64(line);
            }

            if line.starts_with("cpu") && line_num > 0 {
                stat.cpus.push(to_vecu64(line));
            }

            if line.starts_with("intr") {
                stat.intr = to_vecu64(line);
            }

            if line.starts_with("ctxt") {
                let mut chunks = line.split_whitespace();
                chunks.next();

                stat.ctxt = chunks.next().unwrap().parse::<u64>().unwrap();
            }

            if line.starts_with("btime") {
                let mut chunks = line.split_whitespace();
                chunks.next();

                stat.btime = chunks.next().unwrap().parse::<u32>().unwrap();
            }

            if line.starts_with("processes") {
                let mut chunks = line.split_whitespace();
                chunks.next();

                stat.processes = chunks.next().unwrap().parse::<u32>().unwrap();
            }

            if line.starts_with("procs_running") {
                let mut chunks = line.split_whitespace();
                chunks.next();

                stat.procs_running = chunks.next().unwrap().parse::<u32>().unwrap();
            }

            if line.starts_with("procs_blocked") {
                let mut chunks = line.split_whitespace();
                chunks.next();

                stat.procs_blocked = chunks.next().unwrap().parse::<u32>().unwrap();
            }

            if line.starts_with("softirq") {
                stat.softirq = to_vecu64(line);
            }
        }

        Ok(stat)
    }
}

/// Represents the output of `cat /proc/meminfo`
#[derive(Debug, PartialEq, Clone, Default)]
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
    pub direct_map_2m: u64,
}

impl FromStr for MemInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<MemInfo, Infallible> {
        let mut meminfo: MemInfo = Default::default();

        for line in s.lines() {
            if line.starts_with("MemTotal") {
                meminfo.mem_total = to_u64(line);
            }

            if line.starts_with("MemFree") {
                meminfo.mem_free = to_u64(line);
            }

            if line.starts_with("MemAvailable") {
                meminfo.mem_available = to_u64(line);
            }

            if line.starts_with("Buffers") {
                meminfo.bufers = to_u64(line);
            }

            if line.starts_with("Cached") {
                meminfo.cached = to_u64(line);
            }

            if line.starts_with("SwapCached") {
                meminfo.swap_cached = to_u64(line);
            }

            if line.starts_with("Active") {
                meminfo.active = to_u64(line);
            }

            if line.starts_with("Inactive") {
                meminfo.inactive = to_u64(line);
            }

            if line.starts_with("Active(anon)") {
                meminfo.active_anon = to_u64(line);
            }

            if line.starts_with("Inactive(anon)") {
                meminfo.inactive_anon = to_u64(line);
            }

            if line.starts_with("Active(file)") {
                meminfo.active_file = to_u64(line);
            }

            if line.starts_with("Inactive(file)") {
                meminfo.inactive_file = to_u64(line);
            }

            if line.starts_with("Unevictable") {
                meminfo.unevictable = to_u64(line);
            }

            if line.starts_with("Mlocked") {
                meminfo.mlocked = to_u64(line);
            }

            if line.starts_with("SwapTotal") {
                meminfo.swap_total = to_u64(line);
            }

            if line.starts_with("SwapFree") {
                meminfo.swap_free = to_u64(line);
            }

            if line.starts_with("Dirty") {
                meminfo.dirty = to_u64(line);
            }

            if line.starts_with("Writeback") {
                meminfo.writeback = to_u64(line);
            }

            if line.starts_with("AnonPages") {
                meminfo.anon_pages = to_u64(line);
            }

            if line.starts_with("Mapped") {
                meminfo.mapped = to_u64(line);
            }

            if line.starts_with("Shmem") {
                meminfo.shmem = to_u64(line);
            }

            if line.starts_with("Slab") {
                meminfo.slab = to_u64(line);
            }

            if line.starts_with("SReclaimable") {
                meminfo.s_reclaimable = to_u64(line);
            }

            if line.starts_with("SUnreclaim") {
                meminfo.s_unreclaim = to_u64(line);
            }

            if line.starts_with("KernelStack") {
                meminfo.kernel_stack = to_u64(line);
            }

            if line.starts_with("PageTables") {
                meminfo.page_tables = to_u64(line);
            }

            if line.starts_with("NFS_Unstable") {
                meminfo.nfs_unstable = to_u64(line);
            }

            if line.starts_with("Bounce") {
                meminfo.bounce = to_u64(line);
            }

            if line.starts_with("WritebackTmp") {
                meminfo.writeback_tmp = to_u64(line);
            }

            if line.starts_with("CommitLimit") {
                meminfo.commit_limit = to_u64(line);
            }

            if line.starts_with("Committed_AS") {
                meminfo.committed_as = to_u64(line);
            }

            if line.starts_with("VmallocTotal") {
                meminfo.vmalloc_total = to_u64(line);
            }

            if line.starts_with("VmallocUsed") {
                meminfo.vmalloc_used = to_u64(line);
            }

            if line.starts_with("VmallocChunk") {
                meminfo.vmalloc_chunk = to_u64(line);
            }

            if line.starts_with("HardwareCorrupted") {
                meminfo.hardware_corrupted = to_u64(line);
            }

            if line.starts_with("AnonHugePages") {
                meminfo.anon_huge_pages = to_u64(line);
            }

            if line.starts_with("CmaTotal") {
                meminfo.cma_total = to_u64(line);
            }

            if line.starts_with("CmaFree") {
                meminfo.cma_free = to_u64(line);
            }

            if line.starts_with("HugePages_Total") {
                meminfo.huge_pages_total = to_u64(line);
            }

            if line.starts_with("HugePages_Free") {
                meminfo.huge_pages_free = to_u64(line);
            }

            if line.starts_with("HugePages_Rsvd") {
                meminfo.huge_pages_rsvd = to_u64(line);
            }

            if line.starts_with("HugePages_Surp") {
                meminfo.huge_pages_surp = to_u64(line);
            }

            if line.starts_with("Hugepagesize") {
                meminfo.hugepagesize = to_u64(line);
            }

            if line.starts_with("DirectMap4k") {
                meminfo.direct_map_4k = to_u64(line);
            }

            if line.starts_with("DirectMap2M") {
                meminfo.direct_map_2m = to_u64(line);
            }
        }

        Ok(meminfo)
    }
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
    Active(u64),
}

/// Represents a line (socket) in output of `cat /proc/net/{tcp,udp}`
#[derive(Clone)]
pub struct Socket {
    pub sl: u64,
    pub local_address: Ipv4Addr,
    pub local_port: u16,
    pub remote_address: Ipv4Addr,
    pub remote_port: u16,
    pub state: SocketState,
    pub tx_queue: u64,
    pub rx_queue: u64,
    pub timer: SocketTimerState,
    pub uid: u32,
    pub inode: u64,
}

pub fn stat() -> io::Result<Stat> {
    read_file("/proc/stat")?
        .parse()
        .map_err(|_| panic!("Infallible result occured"))
}

pub fn meminfo() -> io::Result<MemInfo> {
    read_file("/proc/meminfo")?
        .parse()
        .map_err(|_| panic!("Infallible result occured"))
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

    file.map(|mut f| f.read_to_string(&mut content))
        .and(Ok(content))
}

fn net(file: &str) -> io::Result<Vec<Socket>> {
    let content = read_file(file);
    match content {
        Ok(c) => Ok(c.lines().skip(1).map(to_net_socket).collect()),
        Err(e) => Err(e),
    }
}

fn to_vecu64(line: &str) -> Vec<u64> {
    let mut chunks = line.split_whitespace();
    let mut buf = Vec::<u64>::new();

    // First chunk is always a non-number, descriptive text.
    chunks.next();

    for chunk in chunks {
        buf.push(chunk.parse::<u64>().unwrap());
    }

    buf
}

fn to_u64(line: &str) -> u64 {
    let mut chunks = line.split_whitespace();
    chunks.next();

    chunks.next().unwrap().parse::<u64>().unwrap()
}

fn to_net_socket(line: &str) -> Socket {
    let mut chunks = line.split_whitespace();
    let sl = chunks
        .next()
        .unwrap()
        .split(':')
        .next()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    // Both local and remote addresses are formatted as <host>:<port> pair, so
    // split them further.
    let local: Vec<&str> = chunks.next().unwrap().split(':').collect();
    let remote: Vec<&str> = chunks.next().unwrap().split(':').collect();
    let state = Vec::<u8>::from_hex(chunks.next().unwrap()).unwrap()[0];
    let queues: Vec<&str> = chunks.next().unwrap().split(':').collect();
    let timer: Vec<&str> = chunks.next().unwrap().split(':').collect();
    // retrnsmt - unused
    chunks.next().unwrap();
    let uid = chunks.next().unwrap().parse::<u32>().unwrap();
    // timeout - unused
    chunks.next().unwrap();
    let inode = chunks.next().unwrap().parse::<u64>().unwrap();

    Socket {
        sl,
        local_address: to_ipaddr(local[0]),
        local_port: u16::from_str_radix(local[1], 16).unwrap(),
        remote_address: to_ipaddr(remote[0]),
        remote_port: u16::from_str_radix(remote[1], 16).unwrap(),
        state: SocketState::from_u8(state).unwrap(),
        tx_queue: u64::from_str_radix(queues[0], 16).unwrap(),
        rx_queue: u64::from_str_radix(queues[1], 16).unwrap(),
        timer: match timer[0].parse::<u8>().unwrap() {
            0 => SocketTimerState::Inactive,
            _ => SocketTimerState::Active(u64::from_str_radix(timer[1], 16).unwrap()),
        },
        uid,
        inode,
    }
}

fn to_ipaddr(hex: &str) -> Ipv4Addr {
    let bytes = Vec::<u8>::from_hex(hex).unwrap();
    Ipv4Addr::from([bytes[3], bytes[2], bytes[1], bytes[0]])
}

#[test]
fn test_to_ipaddr() {
    let addr = to_ipaddr("0100007F");
    assert_eq!(addr.octets(), [127, 0, 0, 1]);
}

#[test]
fn test_to_net_socket() {
    let sock = to_net_socket("  49: 0100007F:1132 5B41EE2E:0050 0A 0000000A:00000002 01:0000000B 00000000  1001        0 2796814 1 ffff938ed0741080 20 4 29 10 -1");
    assert_eq!(sock.local_address.octets(), [127, 0, 0, 1]);
    assert_eq!(sock.local_port, 4402);
    assert_eq!(sock.remote_address.octets(), [46, 238, 65, 91]);
    assert_eq!(sock.remote_port, 80);
    assert_eq!(sock.state, SocketState::Listen);
    assert_eq!(sock.tx_queue, 0xA);
    assert_eq!(sock.rx_queue, 2);
    assert_eq!(sock.timer, SocketTimerState::Active(0xB));
    assert_eq!(sock.uid, 1001);
    assert_eq!(sock.inode, 2796814);
}
