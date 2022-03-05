use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use vidmod_macros::*;
use vidmod_node::{
    Frame, FrameKind, Node2MT, Node2T, PullFrame, PullPort, PushFrame, PushPort, TickNode,
};

#[derive(Debug)]
pub struct RawFileSource {
    file:  File,
    kind:  FrameKind,
    ready: usize,
}

impl RawFileSource {
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let mut file = File::open(
            PathBuf::from(params.get("vidmod.path").unwrap()).join(params.get("file").unwrap()),
        )
        .unwrap();
        let kind = params.get("kind").unwrap().as_str().into();
        let ready = match kind {
            FrameKind::U8 => file.stream_len().unwrap() as usize,
            FrameKind::U16 => (file.stream_len().unwrap() as usize) / 2,
            _ => todo!(),
        };

        Self { file, kind, ready }
    }
}

impl PullFrame for RawFileSource {
    fn pull_frame(&mut self, port: &PullPort, count: usize) -> Frame {
        assert_eq!(count, 1);
        match port.name() {
            "out" => match self.kind {
                FrameKind::U8 => {
                    let mut buf = vec![0u8];
                    self.file.read_exact(&mut buf).unwrap();
                    self.ready -= 1;
                    Frame::U8(buf)
                }
                FrameKind::U16 => {
                    let mut buf = [0u8; 2];
                    self.file.read_exact(&mut buf).unwrap();
                    self.ready -= 1;
                    Frame::U16(vec![u16::from_le_bytes(buf)])
                }
                _ => todo!(),
            },
            _ => panic!("Unknown port {}", port.name()),
        }
    }

    fn test_pull_port(&self, name: &str) -> bool {
        name == "out"
    }

    fn pull_port_kind(&self, name: &str) -> FrameKind {
        match name {
            "out" => self.kind,
            _ => panic!("Unknown port {}", name),
        }
    }

    fn ready_to_pull(&self, port: &PullPort) -> usize {
        match port.name() {
            "out" => self.ready,
            _ => panic!("Unknown port {}", port.name()),
        }
    }
}

impl TickNode for RawFileSource {}

#[derive(Debug)]
pub struct RawFileSink {
    file: File,
    kind: FrameKind,
}

impl RawFileSink {
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let file = File::create(
            PathBuf::from(params.get("vidmod.path").unwrap()).join(params.get("file").unwrap()),
        )
        .unwrap();
        let kind = params.get("kind").unwrap().as_str().into();

        Self { file, kind }
    }
}

impl PushFrame for RawFileSink {
    fn push_frame(&mut self, port: &PushPort, frame: Frame) {
        match port.name() {
            "in" => {
                if let Frame::U8(f) = frame {
                    assert_eq!(f.len(), 1);
                    self.file.write_all(&f).unwrap();
                }
            }
            _ => panic!("Unknown port {}", port.name()),
        }
    }

    fn test_push_port(&self, name: &str) -> bool {
        name == "in"
    }

    fn push_port_kind(&self, name: &str) -> FrameKind {
        match name {
            "in" => self.kind,
            _ => panic!("Unknown port {}", name),
        }
    }
    fn ready_to_push(&self, port: &PushPort) -> usize {
        match port.name() {
            "in" => 1,
            _ => panic!("Unknown port {}", port.name()),
        }
    }
}

impl TickNode for RawFileSink {}

#[node]
pub struct RawFileSource2 {
    file: File,
    kind: FrameKind,
}

impl RawFileSource2 {
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let file = File::open(
            PathBuf::from(params.get("vidmod.path").unwrap()).join(params.get("file").unwrap()),
        )
        .unwrap();
        let kind = params.get("kind").unwrap().as_str().into();

        #[node2]
        Self { file, kind }
    }
}

impl Node2T for RawFileSource2 {
    fn init(&mut self) {
        self.register_pullport("out", self.kind, 1);
    }

    fn tick(&mut self) -> bool {
        if self.outbuf_avail("out") > 0 {
            match self.kind {
                FrameKind::U8 => {
                    let mut buf = vec![0u8];
                    if self.file.read_exact(&mut buf).is_ok() {
                        self.outbuf_put("out", Frame::U8(buf));
                        true
                    } else {
                        false
                    }
                }
                FrameKind::U16 => {
                    let mut buf = [0u8; 2];
                    if self.file.read_exact(&mut buf).is_ok() {
                        self.outbuf_put("out", Frame::U16(vec![u16::from_le_bytes(buf)]));
                        true
                    } else {
                        false
                    }
                }
                _ => todo!(),
            }
        } else {
            false
        }
    }
}
