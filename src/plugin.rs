use vidmod_node::Node;
use vidmod_plugin::Plugin;

pub const RAW_FILE_SOURCE: Plugin = Plugin {
    make_node: |params| Node::N2(Box::new(crate::raw_file::RawFileSource::new(params))),
};

pub const RAW_FILE_SINK: Plugin = Plugin {
    make_node: |params| Node::N2(Box::new(crate::raw_file::RawFileSink::new(params))),
};

pub const CONVERT: Plugin = Plugin {
    make_node: |params| Node::N2(Box::new(crate::conversion::Convert::new(params))),
};
