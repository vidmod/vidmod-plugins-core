use std::{
    collections::BTreeMap,
    fs::File,
    io::{ErrorKind, Read, Write},
    path::PathBuf,
};

use all_asserts::assert_le;
use byte_slice_cast::{AsByteSlice, AsSliceOf};
use vidmod_macros::*;
use vidmod_node::{
    frame::{Frame, FrameKind},
    limvecdeque::LimVecDeque,
    Node2MT, Node2T, PullPort, PushPort,
};

#[node_decl]
pub struct RawFileSource {
    file:   File,
    kind:   FrameKind,
    finish: bool,
}

impl RawFileSource {
    #[node_new]
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let file = File::open(
            PathBuf::from(params.get("vidmod.path").unwrap()).join(params.get("file").unwrap()),
        )
        .unwrap();
        let kind = params.get("kind").unwrap().as_str().into();

        Self {
            file,
            kind,
            finish: false,
        }
    }
}

impl Node2T for RawFileSource {
    fn init(&mut self) {
        self.register_pullport("out", self.kind, 4096);
    }

    fn tick(&mut self) -> bool {
        if self.outbuf_avail("out") > 2048 || (self.finish && self.outbuf_avail("out") > 0) {
            match self.kind {
                FrameKind::U8 => {
                    let mut buf = vec![0u8; self.outbuf_avail("out")];
                    if let Ok(amt) = read_full(&mut self.file, &mut buf) {
                        assert_le!(amt, self.outbuf_avail("out"));
                        if amt == 0 {
                            false
                        } else if amt < buf.len() {
                            self.outbuf_put(
                                "out",
                                Frame::U8(LimVecDeque::from(Vec::from(&buf[..amt]))),
                            );
                            true
                        } else {
                            self.outbuf_put("out", Frame::U8(LimVecDeque::from(buf)));
                            true
                        }
                    } else {
                        false
                    }
                }
                FrameKind::U16 => {
                    let mut buf = vec![0u8; self.outbuf_avail("out") * 2];
                    if let Ok(amt) = read_full(&mut self.file, &mut buf) {
                        assert_le!(amt, self.outbuf_avail("out") * 2);
                        if amt == 0 {
                            false
                        } else if amt < buf.len() {
                            self.outbuf_put(
                                "out",
                                Frame::U16(LimVecDeque::from(Vec::from(
                                    buf[..(amt / 2) * 2].as_slice_of::<u16>().unwrap(),
                                ))),
                            );
                            true
                        } else {
                            self.outbuf_put(
                                "out",
                                Frame::U16(LimVecDeque::from(Vec::from(
                                    buf.as_slice_of::<u16>().unwrap(),
                                ))),
                            );
                            true
                        }
                    } else {
                        false
                    }
                }
                FrameKind::F32 => {
                    let mut buf = vec![0u8; self.outbuf_avail("out") * 4];
                    if let Ok(amt) = read_full(&mut self.file, &mut buf) {
                        if amt == 0 {
                            false
                        } else if amt < buf.len() {
                            self.outbuf_put(
                                "out",
                                Frame::F32(LimVecDeque::from(Vec::from(
                                    buf[..(amt / 4) * 4].as_slice_of::<f32>().unwrap(),
                                ))),
                            );
                            true
                        } else {
                            self.outbuf_put(
                                "out",
                                Frame::F32(LimVecDeque::from(Vec::from(
                                    buf.as_slice_of::<f32>().unwrap(),
                                ))),
                            );
                            true
                        }
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

    fn finish(&mut self) -> bool {
        self.finish = true;
        false
    }
}

#[node_decl]
pub struct RawFileSink {
    file:   File,
    kind:   FrameKind,
    finish: bool,
}

impl RawFileSink {
    #[node_new]
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let file = File::create(
            PathBuf::from(params.get("vidmod.path").unwrap()).join(params.get("file").unwrap()),
        )
        .unwrap();
        let kind = params.get("kind").unwrap().as_str().into();

        Self {
            file,
            kind,
            finish: false,
        }
    }
}

impl Node2T for RawFileSink {
    fn init(&mut self) {
        self.register_pushport("in", self.kind, 4096);
    }

    fn tick(&mut self) -> bool {
        if self.inbuf_avail("in") > 2048 || (self.finish && self.inbuf_avail("in") > 0) {
            match self.inbuf_get_all("in") {
                Frame::U8(v) => {
                    let (a, b) = v.as_slices();
                    self.file.write_all(a).unwrap();
                    self.file.write_all(b).unwrap();
                    true
                }
                Frame::U16(v) => {
                    let (a, b) = v.as_slices();
                    self.file.write_all(a.as_byte_slice()).unwrap();
                    self.file.write_all(b.as_byte_slice()).unwrap();
                    true
                }
                Frame::RGBA8x2(v) => {
                    for el in &v {
                        for px in el.iter() {
                            self.file.write_all(&[px.r, px.g, px.b, px.a]).unwrap();
                        }
                    }
                    true
                }
                v => todo!("{:?}", FrameKind::from(&v)),
            }
        } else {
            false
        }
    }

    fn finish(&mut self) -> bool {
        self.finish = true;
        // We want to be ticked until our input buffer is empty
        false
    }
}

fn read_full(f: &mut File, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let mut count = 0;
    while !buf.is_empty() {
        match f.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
                count += n;
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(count)
}
