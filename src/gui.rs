use gtk4::prelude::*;
use gtk4::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib, Application, ApplicationWindow, Box, Button, ComboBoxText, Entry, Image, Label,
    Notebook, Orientation, ScrolledWindow, Separator, SpinButton, TextView,
};
use glib::Bytes;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::camera::list_cameras;
use crate::dual_recorder::{CameraSource, DualCameraRecorder};
use crate::player::list_recordings;
use crate::playback_camera::StereoPlaybackSystem;

const APP_ID: &str = "com.github.fasttube.CamRecordSim";

type LogBuffer = Rc<RefCell<gtk4::TextBuffer>>;

pub fn run_gui() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Camera Record Simulator")
        .default_width(1200)
        .default_height(800)
        .build();

    let main_box = Box::new(Orientation::Vertical, 10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);

    let header = Label::new(Some("<big><b>Dual-Camera Recording and Simulation</b></big>"));
    header.set_use_markup(true);
    main_box.append(&header);

    main_box.append(&Separator::new(Orientation::Horizontal));

    let notebook = Notebook::new();

    let log_buffer = Rc::new(RefCell::new(gtk4::TextBuffer::new(None)));

    let recorder = Rc::new(RefCell::new(DualCameraRecorder::new()));

    let record_tab = create_dual_record_tab(recorder.clone(), log_buffer.clone());
    notebook.append_page(&record_tab, Some(&Label::new(Some("Recording"))));

    let simulation_tab = create_simulation_tab(log_buffer.clone());
    notebook.append_page(&simulation_tab, Some(&Label::new(Some("Simulation"))));

    let playback_tab = create_playback_tab(log_buffer.clone());
    notebook.append_page(&playback_tab, Some(&Label::new(Some("Playback"))));

    let log_tab = create_log_tab(log_buffer.clone());
    notebook.append_page(&log_tab, Some(&Label::new(Some("Log"))));

    main_box.append(&notebook);

    window.set_child(Some(&main_box));
    window.present();
}

fn log_message(log_buffer: &LogBuffer, message: &str) {
    let buffer = log_buffer.borrow();
    let mut end_iter = buffer.end_iter();
    let timestamp = chrono::Local::now().format("%H:%M:%S");
    buffer.insert(&mut end_iter, &format!("[{}] {}\n", timestamp, message));
}

fn create_dual_record_tab(recorder: Rc<RefCell<DualCameraRecorder>>, log_buffer: LogBuffer) -> Box {
    let tab_box = Box::new(Orientation::Vertical, 10);
    tab_box.set_margin_start(10);
    tab_box.set_margin_end(10);
    tab_box.set_margin_top(10);
    tab_box.set_margin_bottom(10);

    let settings_box = Box::new(Orientation::Horizontal, 10);

    let left_col = Box::new(Orientation::Vertical, 5);

    let desc_label = Label::new(Some("<b>Record from real camera(s):</b>"));
    desc_label.set_use_markup(true);
    desc_label.set_xalign(0.0);
    left_col.append(&desc_label);

    let info_label = Label::new(Some("<i>Choose how many cameras you want to record from</i>"));
    info_label.set_use_markup(true);
    info_label.set_xalign(0.0);
    left_col.append(&info_label);

    let cam_count_box = Box::new(Orientation::Horizontal, 5);
    let cam_count_label = Label::new(Some("Number of Cameras:"));
    let camera_count = ComboBoxText::new();
    camera_count.append(Some("1"), "1 Camera");
    camera_count.append(Some("2"), "2 Cameras");
    camera_count.set_active(Some(0));
    cam_count_box.append(&cam_count_label);
    cam_count_box.append(&camera_count);
    left_col.append(&cam_count_box);

    log_message(&log_buffer, "Detecting cameras...");
    let detected_cameras = list_cameras();
    log_message(&log_buffer, &format!("Found {} camera(s)", detected_cameras.len()));

    let cam_ids_box = Box::new(Orientation::Horizontal, 5);
    let cam0_label = Label::new(Some("Camera 0:"));
    let cam0_combo = ComboBoxText::new();
    cam0_combo.set_hexpand(true);

    for cam in &detected_cameras {
        cam0_combo.append(Some(&cam.index.to_string()), &cam.name);
        log_message(&log_buffer, &format!("  - {}", cam.name));
    }
    if !detected_cameras.is_empty() {
        cam0_combo.set_active(Some(0));
    }

    // Debug: Log when camera selection changes
    let log_buffer_cam0 = log_buffer.clone();
    cam0_combo.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            log_message(&log_buffer_cam0, &format!("Camera 0 selection changed to: {}", id));
        }
    });

    let cam1_label = Label::new(Some("Camera 1:"));
    let cam1_combo = ComboBoxText::new();
    cam1_combo.set_hexpand(true);

    for cam in &detected_cameras {
        cam1_combo.append(Some(&cam.index.to_string()), &cam.name);
    }
    if detected_cameras.len() > 1 {
        cam1_combo.set_active(Some(1));
    }

    // Debug: Log when camera selection changes
    let log_buffer_cam1 = log_buffer.clone();
    cam1_combo.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            log_message(&log_buffer_cam1, &format!("Camera 1 selection changed to: {}", id));
        }
    });

    cam1_label.set_visible(false);
    cam1_combo.set_visible(false);

    cam_ids_box.append(&cam0_label);
    cam_ids_box.append(&cam0_combo);
    cam_ids_box.append(&cam1_label);
    cam_ids_box.append(&cam1_combo);

    let cam1_label_clone = cam1_label.clone();
    let cam1_combo_clone = cam1_combo.clone();
    camera_count.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            let show_cam1 = id.as_str() == "2";
            cam1_label_clone.set_visible(show_cam1);
            cam1_combo_clone.set_visible(show_cam1);
        }
    });

    left_col.append(&cam_ids_box);

    let fps_box = Box::new(Orientation::Horizontal, 5);
    let fps_label = Label::new(Some("FPS:"));
    let fps_spin = SpinButton::with_range(1.0, 60.0, 1.0);
    fps_spin.set_value(30.0);
    fps_box.append(&fps_label);
    fps_box.append(&fps_spin);
    left_col.append(&fps_box);

    let duration_box = Box::new(Orientation::Horizontal, 5);
    let duration_label = Label::new(Some("Duration (sec):"));
    let duration_spin = SpinButton::with_range(1.0, 300.0, 1.0);
    duration_spin.set_value(10.0);
    duration_box.append(&duration_label);
    duration_box.append(&duration_spin);
    left_col.append(&duration_box);

    let output_box = Box::new(Orientation::Horizontal, 5);
    let output_label = Label::new(Some("Output:"));
    let output_entry = Entry::new();
    output_entry.set_text("recordings");
    output_entry.set_hexpand(true);
    output_box.append(&output_label);
    output_box.append(&output_entry);
    left_col.append(&output_box);

    settings_box.append(&left_col);

    tab_box.append(&settings_box);

    let button_box = Box::new(Orientation::Horizontal, 5);

    let start_btn = Button::with_label("Start Recording");
    start_btn.add_css_class("suggested-action");

    let stop_btn = Button::with_label("Stop Recording");
    stop_btn.add_css_class("destructive-action");
    stop_btn.set_sensitive(false);

    let status_label = Label::new(Some("Ready"));
    status_label.set_margin_start(20);

    button_box.append(&start_btn);
    button_box.append(&stop_btn);
    button_box.append(&status_label);

    tab_box.append(&button_box);

    tab_box.append(&Separator::new(Orientation::Horizontal));

    let preview_label = Label::new(Some("<b>Live Preview:</b>"));
    preview_label.set_use_markup(true);
    preview_label.set_xalign(0.0);
    tab_box.append(&preview_label);

    let preview_box = Box::new(Orientation::Horizontal, 10);
    preview_box.set_homogeneous(true);

    let left_preview_box = Box::new(Orientation::Vertical, 5);
    let left_label = Label::new(Some("Camera 0"));
    let left_image = Image::new();
    left_image.set_pixel_size(320);
    left_preview_box.append(&left_label);
    left_preview_box.append(&left_image);

    let right_preview_box = Box::new(Orientation::Vertical, 5);
    let right_label = Label::new(Some("Camera 1"));
    let right_image = Image::new();
    right_image.set_pixel_size(320);
    right_preview_box.append(&right_label);
    right_preview_box.append(&right_image);
    right_preview_box.set_visible(false);

    preview_box.append(&left_preview_box);
    preview_box.append(&right_preview_box);

    let right_preview_box_clone = right_preview_box.clone();
    let camera_count_clone = camera_count.clone();
    camera_count.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            right_preview_box_clone.set_visible(id.as_str() == "2");
        }
    });

    tab_box.append(&preview_box);

    let recorder_clone = recorder.clone();
    let cam0_combo_clone = cam0_combo.clone();
    let cam1_combo_clone = cam1_combo.clone();
    let fps_spin_clone = fps_spin.clone();
    let duration_spin_clone = duration_spin.clone();
    let output_entry_clone = output_entry.clone();
    let stop_btn_clone = stop_btn.clone();
    let status_label_clone = status_label.clone();
    let left_image_clone = left_image.clone();
    let right_image_clone = right_image.clone();
    let log_buffer_clone = log_buffer.clone();

    start_btn.connect_clicked(move |btn| {
        let cam_count = camera_count_clone.active_id().unwrap();

        let cam0_id = cam0_combo_clone
            .active_id()
            .and_then(|id| id.as_str().parse::<u32>().ok());

        let cam1_id = cam1_combo_clone
            .active_id()
            .and_then(|id| id.as_str().parse::<u32>().ok());

        let source = if cam_count.as_str() == "1" {
            if let Some(id) = cam0_id {
                log_message(&log_buffer_clone, &format!("Starting recording from camera {}", id));
                CameraSource::Single(id)
            } else {
                status_label_clone.set_label("Error: No camera selected");
                log_message(&log_buffer_clone, "Error: No camera selected for recording");
                return;
            }
        } else {
            match (cam0_id, cam1_id) {
                (Some(id0), Some(id1)) => {
                    log_message(&log_buffer_clone, &format!("Starting recording from cameras {} and {}", id0, id1));
                    CameraSource::Dual(id0, id1)
                }
                _ => {
                    status_label_clone.set_label("Error: Select both cameras");
                    log_message(&log_buffer_clone, "Error: Both cameras must be selected");
                    return;
                }
            }
        };

        let output_dir = PathBuf::from(output_entry_clone.text().as_str());
        let fps = fps_spin_clone.value();
        let duration = duration_spin_clone.value() as u64;

        match recorder_clone
            .borrow_mut()
            .start_recording(source, &output_dir, fps, duration)
        {
            Ok(_) => {
                btn.set_sensitive(false);
                stop_btn_clone.set_sensitive(true);
                status_label_clone.set_label("Recording...");
                log_message(&log_buffer_clone, "Recording started successfully");

                let recorder_preview = recorder_clone.clone();
                let left_img = left_image_clone.clone();
                let right_img = right_image_clone.clone();

                glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
                    let rec = recorder_preview.borrow();

                    if !rec.is_recording() {
                        return glib::ControlFlow::Break;
                    }

                    if let Some(frame) = rec.get_left_frame() {
                        if let Some(pixbuf) = frame_to_pixbuf(&frame, 640, 480) {
                            left_img.set_from_pixbuf(Some(&pixbuf));
                        }
                    }

                    if let Some(frame) = rec.get_right_frame() {
                        if let Some(pixbuf) = frame_to_pixbuf(&frame, 640, 480) {
                            right_img.set_from_pixbuf(Some(&pixbuf));
                        }
                    }

                    glib::ControlFlow::Continue
                });
            }
            Err(e) => {
                let error_msg = format!("Error: {}", e);
                status_label_clone.set_label(&error_msg);
                log_message(&log_buffer_clone, &error_msg);
            }
        }
    });

    let recorder_clone2 = recorder.clone();
    let start_btn_clone = start_btn.clone();
    let status_label_clone2 = status_label.clone();

    stop_btn.connect_clicked(move |btn| {
        recorder_clone2.borrow_mut().stop_recording();
        btn.set_sensitive(false);
        start_btn_clone.set_sensitive(true);
        status_label_clone2.set_label("Recording stopped");
    });

    tab_box
}

fn frame_to_pixbuf(frame: &[u8], width: i32, height: i32) -> Option<Pixbuf> {
    let expected_size = (width * height * 3) as usize;
    if frame.len() < expected_size {
        return None;
    }

    Some(Pixbuf::from_bytes(
        &Bytes::from(&frame[..expected_size]),
        Colorspace::Rgb,
        false,
        8,
        width,
        height,
        width * 3,
    ))
}

fn create_simulation_tab(_log_buffer: LogBuffer) -> Box {
    let tab_box = Box::new(Orientation::Vertical, 10);
    tab_box.set_margin_start(10);
    tab_box.set_margin_end(10);
    tab_box.set_margin_top(10);
    tab_box.set_margin_bottom(10);

    let desc = Label::new(Some(
        "<b>Virtual Camera Simulation:</b>\n\
        Select a folder containing recordings. Videos will be played as virtual cameras:\n\
        • First file → Left virtual camera (loop)\n\
        • Second file → Right virtual camera (loop)\n\n\
        Other applications can use these virtual cameras like real cameras!",
    ));
    desc.set_use_markup(true);
    desc.set_xalign(0.0);
    tab_box.append(&desc);

    tab_box.append(&Separator::new(Orientation::Horizontal));

    let folder_label = Label::new(Some("<b>Recordings Folder:</b>"));
    folder_label.set_use_markup(true);
    folder_label.set_xalign(0.0);
    tab_box.append(&folder_label);

    let folder_box = Box::new(Orientation::Horizontal, 5);
    let folder_entry = Entry::new();
    folder_entry.set_text("recordings");
    folder_entry.set_hexpand(true);
    folder_box.append(&folder_entry);

    let load_btn = Button::with_label("Load");
    load_btn.add_css_class("suggested-action");
    folder_box.append(&load_btn);

    tab_box.append(&folder_box);

    let status_box = Box::new(Orientation::Vertical, 5);
    let left_status = Label::new(Some("<i>Left camera: Not loaded</i>"));
    left_status.set_use_markup(true);
    left_status.set_xalign(0.0);

    let right_status = Label::new(Some("<i>Right camera: Not loaded</i>"));
    right_status.set_use_markup(true);
    right_status.set_xalign(0.0);

    status_box.append(&left_status);
    status_box.append(&right_status);
    tab_box.append(&status_box);

    tab_box.append(&Separator::new(Orientation::Horizontal));

    let control_label = Label::new(Some("<b>Simulation:</b>"));
    control_label.set_use_markup(true);
    control_label.set_xalign(0.0);
    tab_box.append(&control_label);

    let button_box = Box::new(Orientation::Horizontal, 5);

    let start_sim_btn = Button::with_label("Start Simulation");
    start_sim_btn.add_css_class("suggested-action");
    start_sim_btn.set_sensitive(false);

    let stop_sim_btn = Button::with_label("Stop Simulation");
    stop_sim_btn.add_css_class("destructive-action");
    stop_sim_btn.set_sensitive(false);

    let sim_status = Label::new(Some("Stopped"));
    sim_status.set_margin_start(20);

    button_box.append(&start_sim_btn);
    button_box.append(&stop_sim_btn);
    button_box.append(&sim_status);

    tab_box.append(&button_box);

    tab_box.append(&Separator::new(Orientation::Horizontal));

    let preview_label = Label::new(Some("<b>Virtual Camera Preview:</b>"));
    preview_label.set_use_markup(true);
    preview_label.set_xalign(0.0);
    tab_box.append(&preview_label);

    let preview_box = Box::new(Orientation::Horizontal, 10);
    preview_box.set_homogeneous(true);

    let left_preview_box = Box::new(Orientation::Vertical, 5);
    let left_preview_label = Label::new(Some("Virtual Camera 0 (Left)"));
    let left_preview_image = Image::new();
    left_preview_image.set_pixel_size(320);
    left_preview_box.append(&left_preview_label);
    left_preview_box.append(&left_preview_image);

    let right_preview_box = Box::new(Orientation::Vertical, 5);
    let right_preview_label = Label::new(Some("Virtual Camera 1 (Right)"));
    let right_preview_image = Image::new();
    right_preview_image.set_pixel_size(320);
    right_preview_box.append(&right_preview_label);
    right_preview_box.append(&right_preview_image);

    preview_box.append(&left_preview_box);
    preview_box.append(&right_preview_box);

    tab_box.append(&preview_box);

    let stereo_system: Rc<RefCell<Option<StereoPlaybackSystem>>> = Rc::new(RefCell::new(None));
    let is_running = Rc::new(RefCell::new(false));

    let folder_entry_clone = folder_entry.clone();
    let left_status_clone = left_status.clone();
    let right_status_clone = right_status.clone();
    let start_sim_btn_clone = start_sim_btn.clone();
    let stereo_system_clone = stereo_system.clone();

    load_btn.connect_clicked(move |_| {
        let folder_path = PathBuf::from(folder_entry_clone.text().as_str());

        match StereoPlaybackSystem::load_from_directory(&folder_path) {
            Ok(system) => {
                let status = system.get_status();
                let lines: Vec<&str> = status.lines().collect();

                if lines.len() >= 2 {
                    left_status_clone.set_markup(&format!("<b>{}</b>", lines[0]));
                    right_status_clone.set_markup(&format!("<b>{}</b>", lines[1]));
                }

                *stereo_system_clone.borrow_mut() = Some(system);
                start_sim_btn_clone.set_sensitive(true);

                println!("Stereo system loaded successfully!");
            }
            Err(e) => {
                left_status_clone.set_markup("<span foreground='red'><i>Error loading</i></span>");
                right_status_clone.set_markup(&format!("<span foreground='red'>{}</span>", e));
                start_sim_btn_clone.set_sensitive(false);
            }
        }
    });

    let stereo_system_clone2 = stereo_system.clone();
    let is_running_clone = is_running.clone();
    let stop_sim_btn_clone = stop_sim_btn.clone();
    let sim_status_clone = sim_status.clone();
    let left_preview_image_clone = left_preview_image.clone();
    let right_preview_image_clone = right_preview_image.clone();

    start_sim_btn.connect_clicked(move |btn| {
        if stereo_system_clone2.borrow().is_none() {
            sim_status_clone.set_label("Error: No videos loaded");
            return;
        }

        *is_running_clone.borrow_mut() = true;
        btn.set_sensitive(false);
        stop_sim_btn_clone.set_sensitive(true);
        sim_status_clone.set_label("Running...");

        println!("Simulation started!");

        let stereo_clone = stereo_system_clone2.clone();
        let is_running_preview = is_running_clone.clone();
        let left_img = left_preview_image_clone.clone();
        let right_img = right_preview_image_clone.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
            if !*is_running_preview.borrow() {
                return glib::ControlFlow::Break;
            }

            if let Some(system) = stereo_clone.borrow_mut().as_mut() {
                if let Ok(frame) = system.get_left_frame() {
                    if let Some(pixbuf) = frame_to_pixbuf(&frame, 640, 480) {
                        left_img.set_from_pixbuf(Some(&pixbuf));
                    }
                }

                if let Ok(frame) = system.get_right_frame() {
                    if let Some(pixbuf) = frame_to_pixbuf(&frame, 640, 480) {
                        right_img.set_from_pixbuf(Some(&pixbuf));
                    }
                }
            }

            glib::ControlFlow::Continue
        });
    });

    let is_running_clone2 = is_running.clone();
    let start_sim_btn_clone2 = start_sim_btn.clone();
    let sim_status_clone2 = sim_status.clone();

    stop_sim_btn.connect_clicked(move |btn| {
        *is_running_clone2.borrow_mut() = false;
        btn.set_sensitive(false);
        start_sim_btn_clone2.set_sensitive(true);
        sim_status_clone2.set_label("Stopped");

        println!("Simulation stopped");
    });

    tab_box
}

fn create_playback_tab(_log_buffer: LogBuffer) -> Box {
    let tab_box = Box::new(Orientation::Vertical, 10);
    tab_box.set_margin_start(10);
    tab_box.set_margin_end(10);
    tab_box.set_margin_top(10);
    tab_box.set_margin_bottom(10);

    let title = Label::new(Some("<b>Play Recordings</b>"));
    title.set_use_markup(true);
    title.set_xalign(0.0);
    tab_box.append(&title);

    let list_box = Box::new(Orientation::Horizontal, 5);

    let recordings_combo = ComboBoxText::new();
    recordings_combo.append(Some("none"), "No recordings");
    recordings_combo.set_active(Some(0));
    recordings_combo.set_hexpand(true);
    list_box.append(&recordings_combo);

    let refresh_btn = Button::with_label("Refresh");
    let recordings_combo_clone = recordings_combo.clone();
    refresh_btn.connect_clicked(move |_| {
        recordings_combo_clone.remove_all();

        match list_recordings(&PathBuf::from("recordings")) {
            Ok(recordings) => {
                if recordings.is_empty() {
                    recordings_combo_clone.append(Some("none"), "No recordings");
                } else {
                    for rec in recordings {
                        recordings_combo_clone.append(Some(&rec), &rec);
                    }
                }
            }
            Err(e) => {
                println!("Error loading recordings: {}", e);
                recordings_combo_clone.append(Some("none"), "Error loading");
            }
        }
        recordings_combo_clone.set_active(Some(0));
    });
    list_box.append(&refresh_btn);

    tab_box.append(&list_box);

    let play_btn = Button::with_label("Play");
    play_btn.add_css_class("suggested-action");
    tab_box.append(&play_btn);

    tab_box
}

fn create_log_tab(log_buffer: LogBuffer) -> Box {
    let tab_box = Box::new(Orientation::Vertical, 10);
    tab_box.set_margin_start(10);
    tab_box.set_margin_end(10);
    tab_box.set_margin_top(10);
    tab_box.set_margin_bottom(10);

    let title = Label::new(Some("<b>Application Log</b>"));
    title.set_use_markup(true);
    title.set_xalign(0.0);
    tab_box.append(&title);

    let info_label = Label::new(Some("<i>All system messages, camera detections, and errors appear here</i>"));
    info_label.set_use_markup(true);
    info_label.set_xalign(0.0);
    tab_box.append(&info_label);

    tab_box.append(&Separator::new(Orientation::Horizontal));

    let scrolled = ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);

    let text_view = TextView::new();
    text_view.set_buffer(Some(&*log_buffer.borrow()));
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_monospace(true);
    text_view.set_left_margin(10);
    text_view.set_right_margin(10);
    text_view.set_top_margin(10);
    text_view.set_bottom_margin(10);

    scrolled.set_child(Some(&text_view));
    tab_box.append(&scrolled);

    let button_box = Box::new(Orientation::Horizontal, 5);

    let clear_btn = Button::with_label("Clear Log");
    let log_buffer_clear = log_buffer.clone();
    clear_btn.connect_clicked(move |_| {
        log_buffer_clear.borrow().set_text("");
        log_message(&log_buffer_clear, "Log cleared");
    });
    button_box.append(&clear_btn);

    tab_box.append(&button_box);

    log_message(&log_buffer, "Application started");

    tab_box
}
