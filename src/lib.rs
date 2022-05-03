use std::collections::BTreeMap;

use vidmod_node::Node;

pub mod conversion;
pub mod image;
pub mod raw_file;

#[no_mangle]
pub fn plugin_name() -> String {
    "vidmod-plugins-core".to_owned()
}

#[no_mangle]
pub fn register_plugin() -> Vec<(String, fn(params: BTreeMap<String, String>) -> Node)> {
    vec![
        ("RawFileSource".to_owned(), |params| {
            Node(Box::new(raw_file::RawFileSource::new(params)))
        }),
        ("RawFileSink".to_owned(), |params| {
            Node(Box::new(raw_file::RawFileSink::new(params)))
        }),
        ("Convert".to_owned(), |params| {
            Node(Box::new(conversion::Convert::new(params)))
        }),
        ("ImageSource".to_owned(), |params| {
            Node(Box::new(image::ImageSource::new(params)))
        }),
        ("ImageSink".to_owned(), |params| {
            Node(Box::new(image::ImageSink::new(params)))
        }),
    ]
}
