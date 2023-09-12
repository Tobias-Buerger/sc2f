use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf}, mem::replace,
};

use eframe::egui;

use egui_extras::RetainedImage;

use crate::image_buffer::ImageBuffer;

#[derive(Default)]
struct FolderSelect {
    source_path: Option<PathBuf>,
    destination_path: Option<PathBuf>,
}

struct ImageShow {
    src: PathBuf,
    dst: PathBuf,
    current_image: RetainedImage,
    image_paths: Box<[PathBuf]>,
    copied: Box<[bool]>,
    image_id: usize,
    image_buffer: ImageBuffer,
}

enum AppState {
    FolderSelect(FolderSelect),
    ImageShow(ImageShow),
}

#[derive(Default)]
struct App {
    state: AppState,
}

pub(crate) fn run(_args: &crate::CliArgs) {
    let options = eframe::NativeOptions::default();
    eframe::run_native("sc2f", options, Box::new(|_cc| Box::<App>::default())).unwrap();
}

impl App {
    fn view_folder_select(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let state = if let AppState::FolderSelect(state) = &mut self.state { state } else { panic!("method should only be called if in correct state") };
            ui.label("Select source and destination directory:");

            if ui.button("Select source directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.source_path = Some(path);
                }
            }

            if let Some(picked_path) = &state.source_path {
                ui.horizontal(|ui| {
                    ui.label("Picked source path:");
                    ui.monospace(picked_path.display().to_string());
                });
            }

            if ui.button("Select destination directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.destination_path = Some(path);
                }
            }

            if let Some(picked_path) = &state.destination_path {
                ui.horizontal(|ui| {
                    ui.label("Picked destination path:");
                    ui.monospace(picked_path.display().to_string());
                });
            }

            if ui.button("Go").clicked() {
                if let (Some(src), Some(dst)) = (&state.source_path, &state.destination_path) {
                    self.state = AppState::ImageShow(create_img_show(src.clone(), dst.clone()).expect("could not load image"));
                    ctx.request_repaint();
                } else {
                    ui.label("Please select source and destination path first!");
                }
            }
        });
    }

    fn image_viewer(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let state = if let AppState::ImageShow(state) = &mut self.state { state } else { panic!("method should only be called if in correct state") };
            let img = &state.current_image;
            ui.horizontal(|ui| {
                ui.label(format!(
                    "image {}/{}",
                    state.image_id + 1,
                    state.image_paths.len()
                ));
                ui.label(
                    state.image_paths[state.image_id]
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                );
                if state.copied[state.image_id] {
                    ui.label("copied");
                } else {
                    ui.label("not copied");
                }
            });
            img.show_max_size(ui, ui.available_size());

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !state.copied[state.image_id] {
                let filename = state.image_paths[state.image_id].file_name().unwrap();
                if fs::copy(
                    &state.image_paths[state.image_id],
                    state.dst.join(filename),
                )
                .is_ok()
                {
                    state.copied[state.image_id] = true;
                    ctx.request_repaint();
                }
            }

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                if state.image_id > 0 {
                    let new_id = state.image_id - 1;
                    load_new_image(state, new_id);
                    ctx.request_repaint();
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight))
                && state.image_id < state.image_paths.len() - 1
            {
                let new_id = state.image_id + 1;
                load_new_image(state, new_id);
                ctx.request_repaint();
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.state {
            AppState::FolderSelect { .. } => self.view_folder_select(ctx, frame),
            AppState::ImageShow { .. } => self.image_viewer(ctx, frame),
        }
    }
}

fn load_new_image(state: &mut ImageShow, new_id: usize) {
    assert!(new_id < state.image_paths.len());
    if let Ok(img) = load_image_from_path(&state.image_paths[new_id]) {
        let new_image = RetainedImage::from_color_image("image preview", img);
        let old_img = replace(&mut state.current_image, new_image);
        state.image_id = new_id;
    }
}

pub fn load_image_from_path<P: AsRef<Path>>(path: P) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn create_img_show(src: PathBuf, dst: PathBuf) -> Option<ImageShow> {
    let mut image_paths = vec![];
    for path in src.read_dir().expect("could not read source directory") {
        let path = path.expect("could not read dir entry").path();
        if path.is_file() && path.has_extension(&["jpg", "png", "jpeg", "webp", "svg"]) {
            image_paths.push(path);
        }
    }
    image_paths.sort();
    let image_paths = image_paths.into_boxed_slice();
    let len = image_paths.len();
    let copied = vec![false; len].into_boxed_slice();
    if image_paths.is_empty() {
        return None;
    }
    if let Ok(img) = load_image_from_path(&image_paths[0]) {
        let current_image = RetainedImage::from_color_image("image preview", img);
        Some(ImageShow {
            src: src,
            dst: dst,
            current_image: current_image,
            image_paths: image_paths,
            copied: copied,
            image_id: 0,
            image_buffer: ImageBuffer::new(0, 4, len),
        })
    } else {
        None
    }
}

pub trait FileExtension {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool;
}

impl<P: AsRef<Path>> FileExtension for P {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool {
        if let Some(extension) = self.as_ref().extension().and_then(OsStr::to_str) {
            return extensions
                .iter()
                .any(|x| x.as_ref().eq_ignore_ascii_case(extension));
        }
        false
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::FolderSelect(FolderSelect::default())
    }
}