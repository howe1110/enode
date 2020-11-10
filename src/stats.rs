use time::PreciseTime;

pub struct Stats {
    pub receive_count: usize,
    pub send_count: usize,
    pub receive_count_for_rate: usize,
    pub send_count_for_rate: usize,
    pub start: PreciseTime,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            receive_count: 0,
            send_count: 0,
            start: PreciseTime::now(),
            receive_count_for_rate: 0,
            send_count_for_rate: 0,
        }
    }

    pub fn receive_increase(&mut self) {
        self.receive_count += 1;
        self.receive_count_for_rate += 1;
    }

    pub fn send_increase(&mut self) {
        self.send_count += 1;
        self.send_count_for_rate += 1;
    }

    pub fn get_receive_count(&self) -> usize {
        self.receive_count
    }

    pub fn get_send_count(&self) -> usize {
        self.send_count
    }

    pub fn show(&mut self, duration: usize) {
        println!(
            "send rate: {}, receive rate: {} .",
            self.send_count_for_rate / duration,
            self.receive_count_for_rate / duration
        );
        self.receive_count_for_rate = 0;
        self.send_count_for_rate = 0;
        self.start = PreciseTime::now();
    }
}
