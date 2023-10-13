use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use eframe::egui;



use crate::{image_buffer::ImageBuffer, CliArgs};

#[derive(Default)]
struct FolderSelect {
    source_path: Option<PathBuf>,
    destination_path: Option<PathBuf>,
}

struct ImageShow {
    _src: PathBuf,
    dst: PathBuf,
    image_paths: Box<[PathBuf]>,
    copied: Box<[bool]>,
    image_id: usize,
    image_buffer: ImageBuffer,
}

enum AppState {
    FolderSelect(FolderSelect),
    ImageShow(ImageShow),
}

struct App {
    args: CliArgs,
    state: AppState,
}

pub(crate) fn run(args: crate::CliArgs) {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "sc2f",
        options,
        Box::new(|_cc| Box::<App>::new(App::new(args))),
    )
    .unwrap();
}

impl App {
    fn new(args: crate::CliArgs) -> Self {
        Self {
            args,
            state: AppState::FolderSelect(FolderSelect::default()),
        }
    }

    fn view_folder_select(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let state = if let AppState::FolderSelect(state) = &mut self.state {
                state
            } else {
                panic!("method should only be called if in correct state")
            };
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
                    self.state = AppState::ImageShow(
                        create_img_show(&self.args, src.clone(), dst.clone()).expect("could not load image"),
                    );
                    ctx.request_repaint();
                } else {
                    ui.label("Please select source and destination path first!");
                }
            }
        });
    }

    /// Assume [App] is in State [AppState::ImageShow] else panic
    fn image_viewer(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let state = if let AppState::ImageShow(state) = &mut self.state {
                state
            } else {
                panic!("method should only be called if in correct state")
            };
            load_future_images(state.image_id, &state.image_paths, &mut state.image_buffer);
            let img = &state
                .image_buffer
                .get_or_load(&state.image_paths[state.image_id], state.image_id);
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
            ui.vertical_centered(|ui| {
                img.show_max_size(ui, ui.available_size());
            });

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !state.copied[state.image_id] {
                let filename = state.image_paths[state.image_id].file_name().unwrap();
                if fs::copy(&state.image_paths[state.image_id], state.dst.join(filename)).is_ok() {
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
    state
        .image_buffer
        .load_async(&state.image_paths[new_id], new_id);
    state.image_id = new_id;
}

fn create_img_show(cli_args: &crate::CliArgs, src: PathBuf, dst: PathBuf) -> Option<ImageShow> {
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
    Some(ImageShow {
        _src: src,
        dst,
        image_paths,
        copied,
        image_id: 0,
        image_buffer: ImageBuffer::new(0, cli_args.cached_images, len),
    })
}


/// Load future images
fn load_future_images(current_index: usize, paths: &[PathBuf], buffer: &mut ImageBuffer) {
    // unsure current index is loaded
    buffer.get_or_load(&paths[current_index], current_index);
    // load left image
    if current_index > 0 {
        let left = current_index - 1;
        buffer.load_async(&paths[left], left);
    }
    // load right image
    if current_index < paths.len() - 1 {
        let right = current_index + 1;
        buffer.load_async(&paths[right], right);
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
