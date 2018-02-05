use futures::{Async, Poll};
use futures::stream::Stream;
use std::io;
use hyper;
use futures::sync::mpsc;
use std::sync::{Arc, RwLock};
use bytes::Bytes;

pub struct Segment {
    pub bytes: Bytes,
    pub completion: bool,
}

pub struct SegmentStream {
    processed_bytes: usize,
    segment: Arc<RwLock<Segment>>,
}

impl SegmentStream {
    pub fn new(segment: Arc<RwLock<Segment>>) -> SegmentStream {
        SegmentStream {
            processed_bytes: 0,
            segment: segment,
        }
    }

    pub fn new_with_string(content: String) -> SegmentStream {
        SegmentStream {
            processed_bytes: 0,
            segment: Arc::new(RwLock::new(Segment {
                bytes: Bytes::from(content),
                completion: true,
            })),
        }
    }
}

impl Stream for SegmentStream {
    type Item = hyper::Chunk;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let segment = self.segment.read().expect("Failed to lock segment bytes");
        let bytes = &segment.bytes;
        if bytes.len() == self.processed_bytes {
            if segment.completion {
                Ok(Async::Ready(None))
            } else {
                Ok(Async::NotReady)
            }
        } else if bytes.len() > self.processed_bytes {
            let ready = Async::Ready(Some(hyper::Chunk::from(
                bytes.slice(self.processed_bytes, bytes.len()),
            )));
            self.processed_bytes = bytes.len();
            Ok(ready)
        } else {
            panic!(
                "Logic error: segment length is lessor than processed length: {} < {}",
                bytes.len(),
                self.processed_bytes
            )
        }
    }
}