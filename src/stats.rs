use std::collections::HashMap;
use std::net::SocketAddr;

struct AddrStats {
    pub receive_count: usize,
    pub send_count: usize,
}

impl AddrStats {
    pub fn new() -> AddrStats {
        AddrStats {
            receive_count: 0,
            send_count: 0,
        }
    }
}
pub struct Stats {
    stats: HashMap<SocketAddr, AddrStats>,
}

impl Stats {
    pub fn new() -> Stats {
        let stats = HashMap::new();
        Stats { stats }
    }

    pub fn receive_increase(&mut self, addr: SocketAddr) {
        let stat = self.stats.entry(addr).or_insert(AddrStats::new());
        stat.receive_count += 1;
    }

    pub fn send_increase(&mut self, addr: SocketAddr) {
        let stat = self.stats.entry(addr).or_insert(AddrStats::new());
        stat.send_count += 1;
    }

    pub fn receive_count(&self, addr: SocketAddr) -> usize {
        let stat = self.stats.get(&addr);
        if let Some(v) = stat {
            return v.receive_count;
        }
        0
    }

    pub fn send_count(&self, addr: SocketAddr) -> usize {
        let stat = self.stats.get(&addr);
        if let Some(v) = stat {
            return v.send_count;
        }
        0
    }
}
