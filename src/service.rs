use std::cmp::Ordering;
use std::collections::BinaryHeap;
use tokio::time::{Duration, Instant};

/// Device types that may be intermittently serviced
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ServiceKey {
    Scsi,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ServiceRequest {
    pub key: ServiceKey,
    pub when: Instant,
}

impl Ord for ServiceRequest {
    fn cmp(&self, other: &ServiceRequest) -> Ordering {
        other.when.cmp(&self.when)
    }
}

impl PartialOrd for ServiceRequest {
    fn partial_cmp(&self, other: &ServiceRequest) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ServiceQueue {
    pub queue: BinaryHeap<ServiceRequest>,
}

impl Default for ServiceQueue {
    fn default() -> Self {
        ServiceQueue::new()
    }
}

impl ServiceQueue {
    pub fn new() -> Self {
        ServiceQueue {
            queue: BinaryHeap::new(),
        }
    }

    pub fn schedule(&mut self, key: ServiceKey, delay: Duration) {
        self.queue.push(ServiceRequest {
            key,
            when: Instant::now() + delay,
        });
    }

    pub fn take(&mut self) -> Option<ServiceRequest> {
        match self.queue.peek() {
            Some(srq) if Instant::now() > srq.when => self.queue.pop(),
            _ => None,
        }
    }
}
