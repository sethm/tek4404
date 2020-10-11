//! Device Service Queue
//
// Copyright 2020 Seth Morabito <web@loomcom.com>
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy,
// modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//
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
