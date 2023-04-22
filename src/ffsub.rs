use std::path::PathBuf;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct StreamInfo {
    pub index: i32,
    pub codec_type: String,
    pub tags: StreamTags,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct StreamTags {
    pub language: String,
    pub title: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StreamOutput {
    streams: Vec<StreamInfo>,
}

pub fn get_sub_info(path: &std::path::PathBuf) -> Vec<StreamInfo> {
    let output = std::process::Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-of")
        .arg("json")
        .arg(path)
        .arg("-show_streams")
        .stdout(std::process::Stdio::piped())
        .output()
        .unwrap();
    let result = String::from_utf8_lossy(&output.stdout);
    let result: StreamOutput = serde_json::from_str(&result).unwrap();
    result.streams
}

pub fn lang_code_to_name(lang_code: &str) -> String {
    let language = if lang_code.len() == 3 {
        rust_iso639::from_code_3(&lang_code)
            .or(rust_iso639::from_code_2t(&lang_code))
            .or(rust_iso639::from_code_2b(&lang_code))
    } else {
        rust_iso639::from_code_1(&lang_code)
    };

    let language = match language {
        Some(lang) => lang.name.to_owned(),
        None => lang_code.to_owned(),
    };

    language
}

pub fn extract_subtitle(input: &PathBuf, sub: &str, output: &PathBuf) {
    let map_arg = format!("0:{}", sub);

    let _ffmpeg = std::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-map")
        .arg(&map_arg)
        .arg(output)
        .stderr(std::process::Stdio::piped())
        .status()
        .expect("failed to execute ffmpeg");
}
