use std::{cmp::min, collections::BTreeMap, convert::TryInto};

use vidmod_macros::*;
use vidmod_node::{
    frame::{Frame, FrameKind},
    Node2MT, Node2T, PullPort, PushPort,
};

#[node_decl]
pub struct Convert {
    from: FrameKind,
    to:   FrameKind,
}

impl Convert {
    #[node_new]
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let from = params.get("from").unwrap().as_str().into();
        let to = params.get("to").unwrap().as_str().into();

        Self { from, to }
    }
}

impl Node2T for Convert {
    fn init(&mut self) {
        self.register_pullport("out", self.to, 1);
        self.register_pushport("in", self.from, 1);
    }

    fn tick(&mut self) -> bool {
        let to_transfer = min(self.inbuf_avail("in"), self.outbuf_avail("out"));

        if to_transfer > 0 {
            match (self.inbuf_get("in", to_transfer), self.to) {
                (Frame::U8(v), FrameKind::U8) => {
                    self.outbuf_put("out", Frame::U8(v));
                    true
                }
                (Frame::U16(v), FrameKind::U16) => {
                    self.outbuf_put("out", Frame::U16(v));
                    true
                }
                (Frame::U8(v), FrameKind::U16) => {
                    self.outbuf_put(
                        "out",
                        Frame::U16(v.iter().map(|x| u16::from(*x) * 256u16).collect()),
                    );
                    true
                }
                (Frame::U16(v), FrameKind::U8) => {
                    self.outbuf_put(
                        "out",
                        Frame::U8(v.iter().map(|x| (x / 256).try_into().unwrap()).collect()),
                    );
                    true
                }
                (Frame::F32(v), FrameKind::U16) => {
                    self.outbuf_put(
                        "out",
                        Frame::U16(
                            v.iter()
                                .map(|x| (x * 65535.0).clamp(0.0, 65535.0).round() as u16)
                                .collect(),
                        ),
                    );
                    true
                }
                _ => todo!("Conversion {:?} -> {:?}", self.from, self.to),
            }
        } else {
            false
        }
    }

    fn finish(&mut self) -> bool {
        // We want to be ticked until our input buffer is empty
        false
    }
}
