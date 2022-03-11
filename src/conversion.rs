use std::collections::BTreeMap;

use vidmod_macros::*;
use vidmod_node::{FrameKind, FrameSingle, Node2MT, Node2T, PullPort, PushPort};

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
        if self.inbuf_avail("in") > 0 {
            if self.outbuf_avail("out") > 0 {
                match (self.inbuf_get_single("in"), self.to) {
                    (FrameSingle::U8(v), FrameKind::U8) => {
                        self.outbuf_put_single("out", FrameSingle::U8(v));
                        true
                    }
                    (FrameSingle::U16(v), FrameKind::U16) => {
                        self.outbuf_put_single("out", FrameSingle::U16(v));
                        true
                    }
                    (FrameSingle::U8(v), FrameKind::U16) => {
                        self.outbuf_put_single("out", FrameSingle::U16(u16::from(v) * 256u16));
                        true
                    }
                    (FrameSingle::U16(v), FrameKind::U8) => {
                        self.outbuf_put_single(
                            "out",
                            FrameSingle::U8((v / 256).try_into().unwrap()),
                        );
                        true
                    }
                    _ => todo!("Conversion {:?} -> {:?}", self.from, self.to),
                }
            } else {
                false
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
