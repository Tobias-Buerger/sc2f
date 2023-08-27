#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::{Path, PathBuf}, ffi::OsStr, fs, fmt::format};

use egui::ColorImage;
use rfd;
use eframe::egui;
use egui_extras::RetainedImage;
use image;

struct App {
    source_path: Option<PathBuf>,
    destination_path: Option<PathBuf>,
    view_images: bool,
    current_image: Option<RetainedImage>,
    image_paths: Vec<PathBuf>,
    copied: Vec<bool>,
    image_id: usize,
}

pub(crate) fn run(args: &crate::CliArgs) {
    let options = eframe::NativeOptions::default();
    eframe::run_native("sc2f", options, Box::new(|_cc| Box::<App>::default())).unwrap();
}

impl App {
    fn view_folder_select(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Select source and destination directory:");

            if ui.button("Select source directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.source_path = Some(path);
                }
            }

            if let Some(picked_path) = &self.source_path {
                ui.horizontal(|ui| {
                    ui.label("Picked source path:");
                    ui.monospace(picked_path.display().to_string());
                });
            }


            if ui.button("Select destination directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.destination_path = Some(path);
                }
            }

            if let Some(picked_path) = &self.destination_path {
                ui.horizontal(|ui| {
                    ui.label("Picked destination path:");
                    ui.monospace(picked_path.display().to_string());
                });
            }

            if ui.button("Go").clicked() {
                if self.source_path.is_some() && self.destination_path.is_some() {
                    self.view_images = true;
                    self.collect_image_files();
                    ctx.request_repaint();
                } else {
                    ui.label("Please select source and destination path first!");
                }
            }
        });
    }

    fn collect_image_files(&mut self) {
        let source = self.source_path.as_ref().unwrap();
        for path in source.read_dir().expect("could not read source directory") {
            let path = path.expect("could not read dir entry").path();
            if path.is_file() {
                if path.has_extension(&["jpg", "png", "jpeg", "webp", "svg"]) {
                    self.image_paths.push(path);
                }
            }
        }
        self.image_paths.sort();
        self.copied = vec![false; self.image_paths.len()];
        if self.image_paths.len() == 0 {
            return;
        }
        self.load_current_img();
    }

    fn load_current_img(&mut self) {
        if let Ok(img) = load_image_from_path(&self.image_paths[self.image_id]) {
            self.current_image = Some(RetainedImage::from_color_image("image preview", img));
        }
    }

    fn image_viewer(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(img) = &self.current_image {
                ui.horizontal(|ui| {
                    ui.label(format!("image {}/{}", self.image_id + 1, self.image_paths.len()));
                    ui.label(self.image_paths[self.image_id].file_name().unwrap().to_str().unwrap());
                    if self.copied[self.image_id] {
                        ui.label("copied");
                    } else {
                        ui.label("not copied");
                    }
                });
                img.show_max_size(ui, ui.available_size());
            } else {
                ui.label("Could not load any images!");
            }

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                if !self.copied[self.image_id] {
                    let filename = self.image_paths[self.image_id].file_name().unwrap();
                    if fs::copy(&self.image_paths[self.image_id], self.destination_path.as_ref().unwrap().join(filename)).is_ok() {
                        self.copied[self.image_id] = true;
                        ctx.request_repaint();
                    }
                }
            }

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                if self.image_id > 0 {
                    self.image_id -= 1;
                    self.load_current_img();
                    ctx.request_repaint();
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                if self.image_id < self.image_paths.len() - 1 {
                    self.image_id += 1;
                    self.load_current_img();
                    ctx.request_repaint();
                }
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.view_images{
            self.view_folder_select(ctx, frame);
        } else {
            self.image_viewer(ctx, frame);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            source_path: None,
            destination_path: None,
            view_images: false,
            current_image: None,
            image_paths: vec![],
            copied: vec![],
            image_id: 0,
        }
    }
}

fn load_image_from_path<P: AsRef<Path>>(path: P) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
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