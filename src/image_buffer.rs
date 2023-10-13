use log::*;
use rexif::{ExifTag, TagValue};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    thread::JoinHandle,
};

use egui_extras::RetainedImage;

enum ImageStatus {
    Loaded(RetainedImage),
    Loading(JoinHandle<RetainedImage>),
}

pub struct ImageBuffer {
    buffer_size: usize,
    num_items: usize,
    current_index: usize,
    image_storage: BTreeMap<usize, ImageStatus>,
}

impl ImageBuffer {
    pub fn new(start_index: usize, buffer_size: usize, num_items: usize) -> Self {
        assert!(buffer_size > 0 && num_items > 0);
        Self {
            current_index: start_index,
            buffer_size: buffer_size.min(num_items),
            num_items,
            image_storage: BTreeMap::new(),
        }
    }

    pub fn load_async(&mut self, path: &Path, index: usize) {
        assert!(index < self.num_items);
        if self.image_storage.contains_key(&index) {
            return;
        }
        let path = path.to_path_buf();
        let handle =
            std::thread::spawn(move || load_image_from_path(path).expect("could not load image"));
        self.image_storage
            .insert(index, ImageStatus::Loading(handle));
        self.update_buffer();
    }

    pub fn get_or_load<'a>(&'a mut self, path: &PathBuf, index: usize) -> &'a RetainedImage {
        self.current_index = index;
        if let std::collections::btree_map::Entry::Vacant(e) = self.image_storage.entry(index) {
            // image is currently not in buffer (not loaded and loading)
            let img = load_image_from_path(path).expect("could not load image");
            e.insert(ImageStatus::Loaded(img));
            self.update_buffer();
        }

        // finish async loading
        if matches!(
            self.image_storage.get(&index),
            Some(ImageStatus::Loading(_))
        ) {
            let status = self.image_storage.remove(&index).unwrap();
            match status {
                ImageStatus::Loaded(_) => unreachable!(),
                ImageStatus::Loading(handle) => {
                    let img = handle.join().expect("error while loading image");
                    self.image_storage.insert(index, ImageStatus::Loaded(img));
                }
            }
        }

        match self.image_storage.get(&index).unwrap() {
            ImageStatus::Loaded(img) => img,
            ImageStatus::Loading(_) => unreachable!(),
        }
    }

    /// Remove images from buffer
    fn update_buffer(&mut self) {
        self.image_storage
            .retain(|k, _| k.abs_diff(self.current_index) <= self.buffer_size / 2);
    }
}

pub fn load_image_from_path<P: AsRef<Path>>(path: P) -> Result<RetainedImage, image::ImageError> {
    debug!(
        "loading image {}",
        path.as_ref().file_name().unwrap().to_str().unwrap()
    );
    let reader = image::io::Reader::open(path.as_ref())?;
    trace!("image dimensions: {:?}", reader.into_dimensions().unwrap());
    let orientation = if let Ok(metadata) = rexif::parse_file(path.as_ref()) {
        let mut orientation_data = None;
        for entry in &metadata.entries {
            if entry.tag == ExifTag::Orientation {
                orientation_data = match &entry.value {
                    TagValue::U16(data) => Some(data.clone().into_boxed_slice()),
                    _ => None,
                };
                break;
            }
        }
        orientation_data
    } else {
        None
    };
    let image = image::io::Reader::open(path)?.decode()?;
    let image = {
        // https://stackoverflow.com/questions/57771795/how-to-fix-exif-orientation-data-in-jpg-files
        match orientation.as_deref() {
            Some(&[1]) => {
                // Normal
                image
            }
            Some(&[2]) => {
                // Top, right side (Mirror horizontal)
                image.fliph()
            }
            Some(&[3]) => {
                // Bottom, right side (Rotate 180)
                image.rotate180()
            }
            Some(&[4]) => {
                // Bottom, left side (Mirror vertical)
                image.flipv()
            }
            Some(&[5]) => {
                // Left side, top (Mirror horizontal and rotate 270 CW)
                image.fliph().rotate270()
            }
            Some(&[6]) => {
                // Right side, top (Rotate 90 CW)
                image.rotate90()
            }
            Some(&[7]) => {
                // Right side, bottom (Mirror horizontal and rotate 90 CW)
                image.fliph().rotate90()
            }
            Some(&[8]) => {
                // Left side, bottom (Rotate 270 CW)
                image.rotate270()
            }
            Some(data) => {
                warn!("unkown exif data {:?}", data);
                image
            }
            None => image,
        }
    };
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    Ok(RetainedImage::from_color_image("img preview", color_image))
}
