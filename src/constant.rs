use vidmod_node::{Frame, FrameKind, PullFrame, PullPort, TickNode};

#[derive(Debug)]
pub struct Constant {
    value: Frame,
}

impl Constant {
    pub fn new(value: Frame) -> Self {
        Self { value }
    }
}

impl PullFrame for Constant {
    fn pull_frame(&mut self, port: &PullPort, count: usize) -> Frame {
        assert_eq!(count, 1);
        match port.name() {
            "out" => self.value.clone(),
            _ => panic!("Unknown port {}", port.name()),
        }
    }

    fn test_pull_port(&self, name: &str) -> bool {
        name == "out"
    }

    fn pull_port_kind(&self, name: &str) -> FrameKind {
        match name {
            "out" => FrameKind::from(&self.value),
            _ => panic!("Unknown port {}", name),
        }
    }

    fn ready_to_pull(&self, port: &PullPort) -> usize {
        match port.name() {
            "out" => 1,
            _ => panic!("Unknown port {}", port.name()),
        }
    }
}

impl TickNode for Constant {}
