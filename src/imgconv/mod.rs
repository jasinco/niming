use std::io::Cursor;

use base64ct::{Base64, Encoding};
use crossbeam::channel::{Receiver, Sender, bounded};
use image::ImageReader;

pub struct ImageProcessor {
    recv: Receiver<String>,
    send: Sender<String>,
}
impl ImageProcessor {
    pub fn new() -> Self {
        let (_tx, _rx) = bounded(10);
        ImageProcessor {
            recv: _rx,
            send: _tx,
        }
    }
    pub fn listen(&mut self) {
        for x in self.recv.iter() {}
    }
    pub fn handler(img_str: String) {
        let middle: Vec<_> = img_str.split(";base64,").collect();
        let format = middle.first().and_then(|x| x.split("data:").last());
        if format.is_none() || middle.last().is_none() {
            return;
        }
        let b64_enc = middle.last().unwrap();
        if let Ok(img) = Base64::decode_vec(b64_enc) {
            let mut img_reader = ImageReader::new(Cursor::new(&img));
            match format.unwrap() {
                "image/png" => img_reader.set_format(image::ImageFormat::Png),
                "image/jpeg" => img_reader.set_format(image::ImageFormat::Jpeg),
                "image/webp" => img_reader.set_format(image::ImageFormat::WebP),
                "image/gif" => img_reader.set_format(image::ImageFormat::Gif),
                _ => return,
            }
            if let Ok(dyn_img) = img_reader.decode() {
                // dyn_img.save_with_format()
            }
        }
    }
}
impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}
