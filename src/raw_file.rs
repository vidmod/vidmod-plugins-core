use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use vidmod_macros::*;
use vidmod_node::{Frame, FrameKind, FrameSingle, Node2MT, Node2T, PullPort, PushPort};

#[node]
pub struct RawFileSource {
    file: File,
    kind: FrameKind,
}

impl RawFileSource {
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

impl Node2T for RawFileSource {
    fn init(&mut self) {
        self.register_pullport("out", self.kind, 1);
    }

    fn tick(&mut self) -> bool {
        if self.outbuf_avail("out") > 0 {
            match self.kind {
                FrameKind::U8 => {
                    let mut buf = vec![0u8];
                    if self.file.read_exact(&mut buf).is_ok() {
                        self.outbuf_put("out", Frame::U8(VecDeque::from(buf)));
                        true
                    } else {
                        false
                    }
                }
                FrameKind::U16 => {
                    let mut buf = [0u8; 2];
                    if self.file.read_exact(&mut buf).is_ok() {
                        self.outbuf_put(
                            "out",
                            Frame::U16(VecDeque::from(vec![u16::from_le_bytes(buf)])),
                        );
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

#[node]
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

        #[node2]
        Self { file, kind }
    }
}

impl Node2T for RawFileSink {
    fn init(&mut self) {
        self.register_pushport("in", self.kind, 1);
    }

    fn tick(&mut self) -> bool {
        if self.inbuf_avail("in") > 0 {
            match self.inbuf_get_single("in") {
                FrameSingle::U8(v) => {
                    self.file.write_all(&[v]).unwrap();
                    true
                }
                _ => todo!(),
            }
        } else {
            false
        }
    }
}
