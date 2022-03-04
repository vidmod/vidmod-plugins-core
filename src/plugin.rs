use vidmod_node::Node;
use vidmod_plugin::Plugin;

pub const RAW_FILE_SOURCE: Plugin = Plugin {
    make_node: |params| Node::Source(Box::new(crate::raw_file::RawFileSource::new(params))),
};

pub const RAW_FILE_SINK: Plugin = Plugin {
    make_node: |params| Node::Sink(Box::new(crate::raw_file::RawFileSink::new(params))),
};
