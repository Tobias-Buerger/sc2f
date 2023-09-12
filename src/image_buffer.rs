use std::{path::PathBuf, thread::JoinHandle};

use egui_extras::RetainedImage;

use crate::ui::load_image_from_path;

enum ImageStatus {
    Loaded(RetainedImage),
    Loading(JoinHandle<RetainedImage>),
}

pub struct ImageBuffer {
    buffer_size: usize,
    num_items: usize,
    current_index: usize,
    image_storage: Vec<(usize, ImageStatus)>,
}

impl ImageBuffer {
    pub fn new(start_index: usize, buffer_size: usize, num_items: usize) -> Self {
        Self {
            current_index: start_index,
            buffer_size: buffer_size.min(num_items),
            num_items: num_items,
            image_storage: vec![]
        }
    }

    pub fn load_async(&mut self, path: &PathBuf, index: usize) {
        let path = path.clone();
        let handle = std::thread::spawn(move || {
            let img = load_image_from_path(path).unwrap();
            RetainedImage::from_color_image("img preview", img)
        });
        self.image_storage.push((index, ImageStatus::Loading(handle)));
    }

    pub fn get_or_load(&mut self, path: &PathBuf, index: usize) -> RetainedImage {
        todo!()
    }
}