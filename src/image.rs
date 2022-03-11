use std::{
    collections::BTreeMap,
    fmt::Debug,
    fs::File,
    path::PathBuf,
};

use dynfmt::Format;
use image::{ImageBuffer, ImageOutputFormat};
use maplit::btreemap;
use ndarray::ArcArray2;
use vidmod_macros::*;
use vidmod_node::{FrameKind, FrameSingle, Node2MT, Node2T, PullPort, PushPort, RGBA8};

struct PngReader(png::Reader<File>);

impl Debug for PngReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PngReader")
            .field("decoder", &"png::Reader")
            .finish()
    }
}

#[node_decl]
pub struct ImageSource {
    reader: PngReader,
    kind:   FrameKind,
}

impl ImageSource {
    #[node_new]
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let file = File::open(
            PathBuf::from(params.get("vidmod.path").unwrap()).join(params.get("file").unwrap()),
        )
        .unwrap();
        let decoder = png::Decoder::new(file);

        let reader = PngReader(decoder.read_info().unwrap());
        let kind = match (reader.0.info().bit_depth, reader.0.info().color_type) {
            (png::BitDepth::Eight, png::ColorType::Rgba) => FrameKind::RGBA8x2,
            (a, b) => todo!("{:?},{:?}", a, b),
        };

        Self { reader, kind }
    }
}

impl Node2T for ImageSource {
    fn init(&mut self) {
        self.register_pullport("out", self.kind, 1);
    }

    fn tick(&mut self) -> bool {
        if self.outbuf_avail("out") > 0 {
            match self.kind {
                FrameKind::RGBA8x2 => {
                    let width = self.reader.0.info().width as usize;
                    let height = self.reader.0.info().height as usize;
                    let mut buf = vec![0u8; width * height * 4];
                    if self.reader.0.next_frame(&mut buf).is_ok() {
                        let pixels = unsafe {
                            ::std::slice::from_raw_parts(
                                buf.as_ptr() as *const RGBA8,
                                width * height,
                            )
                        }
                        .to_vec();

                        self.outbuf_put_single(
                            "out",
                            FrameSingle::RGBA8x2(
                                ArcArray2::<RGBA8>::from_shape_vec((width, height), pixels)
                                    .unwrap(),
                            ),
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

#[node_decl]
pub struct ImageSink {
    path:     PathBuf,
    template: String,
    kind:     FrameKind,
    frame:    usize,
}

impl ImageSink {
    #[node_new]
    pub fn new(params: BTreeMap<String, String>) -> Self {
        let path = PathBuf::from(params.get("vidmod.path").unwrap());
        let template = params.get("template").unwrap().clone();
        let kind = params.get("kind").unwrap().as_str().into();
        Self {
            path,
            template,
            kind,
            frame: 0,
        }
    }
}

impl Node2T for ImageSink {
    fn init(&mut self) {
        self.register_pushport("in", self.kind, 1);
    }

    fn tick(&mut self) -> bool {
        if self.inbuf_avail("in") > 0 {
            match self.inbuf_get_single("in") {
                FrameSingle::RGBA8x2(v) => {
                    let filename = String::from(
                        dynfmt::curly::SimpleCurlyFormat
                            .format(&self.template, btreemap! {"frame" => self.frame})
                            .unwrap(),
                    );
                    let mut file = File::create(self.path.join(filename)).unwrap();
                    let buf = unsafe {
                        ::std::slice::from_raw_parts(
                            v.as_ptr() as *const u8,
                            v.ncols() * v.nrows() * 4,
                        )
                    }
                    .to_vec();
                    println!("{},{},{}", buf.len(), v.ncols(), v.nrows());
                    let buf: ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_vec(
                        v.ncols().try_into().unwrap(),
                        v.nrows().try_into().unwrap(),
                        buf,
                    )
                    .unwrap();
                    buf.write_to(&mut file, ImageOutputFormat::Jpeg(50))
                        .unwrap();
                    self.frame += 1;
                    true
                }
                _ => todo!(),
            }
        } else {
            false
        }
    }
}
