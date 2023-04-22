#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, path::PathBuf};

use fltk::{
    browser::CheckBrowser,
    prelude::{MenuExt, WidgetExt},
};

// mod csub_app;
mod ffsub;
mod ui;

#[derive(Default)]
struct FileCache {
    input_file: Option<PathBuf>,
    output_folder: Option<PathBuf>,
}

impl FileCache {
    fn request_input_file(&mut self) -> Option<PathBuf> {
        self.input_file = get_path_by_dialog(
            "Set input movie",
            Some("*.{mkv,mp4}"),
            None,
            fltk::dialog::FileDialogType::BrowseFile,
        );
        self.input_file.clone()
    }

    fn get_input_file(&mut self) -> Option<PathBuf> {
        if self.input_file.is_none() {
            self.request_input_file()
        } else {
            self.input_file.clone()
        }
    }

    fn get_input_file_folder(&self) -> Option<PathBuf> {
        if let Some(input_file) = self.input_file.clone() {
            if let Some(parent) = input_file.parent() {
                Some(parent.to_path_buf())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn request_output_folder(&mut self) -> Option<PathBuf> {
        self.output_folder = get_path_by_dialog(
            "Set output folder to save subtitle, the subtitle is the same as check browser named",
            None,
            self.get_input_file_folder().as_ref(),
            fltk::dialog::FileDialogType::BrowseDir,
        );
        self.output_folder.clone()
    }

    fn get_output_folder(&mut self) -> Option<PathBuf> {
        if self.output_folder.is_none() {
            self.request_output_folder()
        } else {
            self.output_folder.clone()
        }
    }
}

#[derive(Clone)]
enum CSubMsg {
    SetInputFile,
    SetOutputFolder,
    ExtractSubtitle,
    ShowHelp,
    ToolTTS,
    OnWork,
    OffWork,
}
fn main() {
    let app = fltk::app::App::default();
    let (sender, receiver) = fltk::app::channel();
    let mut file_cache = FileCache::default();

    let mut app_ui = ui::AppUI::make_window();

    app_ui.button.emit(sender.clone(), CSubMsg::ExtractSubtitle);

    app_ui.menu_bar.set_callback({
        let sender = sender.clone();
        move |mb| {
            if let Some(choice) = mb.choice() {
                match choice.as_str() {
                    "Open Movie File" => sender.send(CSubMsg::SetInputFile),
                    "Set Output Folder" => sender.send(CSubMsg::SetOutputFolder),
                    "Traditional to simplified" => sender.send(CSubMsg::ToolTTS),
                    "About" => sender.send(CSubMsg::ShowHelp),
                    _ => {}
                }
            }
        }
    });

    while app.wait() {
        if let Some(message) = receiver.recv() {
            match message {
                CSubMsg::SetInputFile => match file_cache.get_input_file() {
                    Some(file) => load_subtitle_to_checkbrowser(&mut app_ui.check_browser, &file),
                    None => {}
                },
                CSubMsg::ExtractSubtitle => {
                    if let Some(input_file) = file_cache.get_input_file() {
                        if app_ui.check_browser.nitems() == 0 {
                            sender.send(CSubMsg::SetInputFile);
                        } else {
                            if let Some(folder) = file_cache.get_output_folder() {
                                let checked = get_checked_from_checkbrowser(&app_ui.check_browser);
                                let sender = sender.clone();
                                let file = input_file.clone();
                                std::thread::spawn(move || {
                                    for (text, index) in checked {
                                        let folder = folder.join(format!("{}.srt", text));
                                        sender.send(CSubMsg::OnWork);
                                        ffsub::extract_subtitle(&file, &index, &folder);
                                    }
                                    sender.send(CSubMsg::OffWork);
                                });
                            }
                        }
                    }
                }
                CSubMsg::SetOutputFolder => {
                    file_cache.request_output_folder();
                }
                CSubMsg::ShowHelp => {
                    fltk::dialog::message_title("About");
                    fltk::dialog::message_default(
                        r#"CSub is under GPLv3 License! You can use it to extract subtitles.
It also provides a "Traditional to simplified" tool.
You may see the GPLv3 License at https://www.gnu.org/licenses/gpl-3.0.html.
You may access the code from https://github.com/felixmaker/csub
"#,
                    );
                }
                CSubMsg::ToolTTS => {
                    let input = get_path_by_dialog(
                        "Choose a traditional subtitle",
                        Some("*.{txt,srt,ass}"),
                        file_cache.get_input_file_folder().as_ref(),
                        fltk::dialog::FileDialogType::BrowseFile,
                    );
                    let output = get_path_by_dialog(
                        "Choose a path to save simplified subtitle",
                        Some("*.{txt,srt,ass}"),
                        file_cache.get_input_file_folder().as_ref(),
                        fltk::dialog::FileDialogType::BrowseSaveFile,
                    );
                    match (input, output) {
                        (Some(i), Some(o)) => {
                            let input_text = std::fs::read_to_string(i).unwrap();
                            let result = zhconv::zhconv(&input_text, zhconv::Variant::ZhCN);
                            let _w_result = std::fs::write(o, result);
                        }
                        _ => {
                            fltk::dialog::alert_default(
                                "Failed to translate! You need to provide input and output!",
                            );
                        }
                    }
                }
                CSubMsg::OnWork => {
                    app_ui.button.deactivate();
                }
                CSubMsg::OffWork => {
                    app_ui.button.activate();
                    fltk::dialog::message_default("Extract Finished!");
                }
            }
        }
    }
}

/// Use File Dialog to get filepath
fn get_path_by_dialog(
    title: &str,
    filter: Option<&str>,
    base_folder: Option<&PathBuf>,
    dialog_type: fltk::dialog::FileDialogType,
) -> Option<std::path::PathBuf> {
    let mut file_dialog = fltk::dialog::FileDialog::new(dialog_type);
    file_dialog.set_title(title);

    if let Some(f) = filter {
        file_dialog.set_filter(f);
    }
    if let Some(pb) = base_folder {
        let _rst = file_dialog.set_directory(pb);
    }

    file_dialog.show();
    if file_dialog.filename() == PathBuf::new() {
        None
    } else {
        Some(file_dialog.filename())
    }
}

fn load_subtitle_to_checkbrowser(check_browser: &mut CheckBrowser, file: &std::path::PathBuf) {
    check_browser.clear();

    let subtitles = ffsub::get_sub_info(file);

    for subtitle in subtitles {
        if subtitle.codec_type == "subtitle" {
            let index = subtitle.index;
            let language = subtitle.tags.language;
            let language = ffsub::lang_code_to_name(&language);
            let title = subtitle.tags.title;
            let subtitle = match title {
                Some(t) => format!("#{} {} - {}", index, language, t),
                None => format!("#{} {}", index, language),
            };
            check_browser.add(&subtitle, false);
        }
    }

    check_browser.redraw();
}

fn get_checked_from_checkbrowser(check_browser: &CheckBrowser) -> HashMap<String, String> {
    let nitems = check_browser.nitems() as i32;
    let mut result = Vec::new();
    let re = regex::Regex::new(r#"#(\d+) "#).unwrap();
    for item in 1..=nitems {
        if check_browser.checked(item) {
            let text = check_browser
                .text(item)
                .unwrap_or("Unknown Item".to_owned());
            if let Some(caps) = re.captures(&text) {
                let index = caps.get(1).unwrap().as_str();
                // println!("{}", index);
                result.push((text.clone(), index.to_owned()))
            }
        }
    }
    result.into_iter().collect()
}
