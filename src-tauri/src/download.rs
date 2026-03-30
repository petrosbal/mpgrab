// Path is for borrowed path references (like &str is to String)
// i use it here because i dont need to own the paths, just read them
use std::path::Path;

// everything in here is pure and has no side effects, so it's all unit testable
// the actual process spawning and event emitting lives in lib.rs

// derive tells the compiler to auto-implement these traits:
//   Debug    allows printing the value with {:?} in logs and test output
//   PartialEq allows comparing two ProgressEvents with ==, needed in tests
//   Clone    allows duplicating the value, needed because i send it across threads
#[derive(Debug, PartialEq, Clone)]
// in Rust, enums can carry data inside each variant, unlike Java enums
// this lets us express "downloading at 50%" as a single typed value
pub enum ProgressEvent {
    Downloading { percent: f32 },
    // yt-dlp hands off to ffmpeg at this point
    Converting,
    Error(String),
}

// Result<(), String> means success carries no value, failure carries an error message
// () is Rusts unit type, roughly equivalent to void
pub fn validate_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("url cannot be empty".to_string());
    }
    // i only want to support YouTube for now, keeping scope tight
    let is_youtube = url.contains("youtube.com/watch?v=")
        || url.contains("youtu.be/")
        || url.contains("youtube.com/shorts/");
    if !is_youtube {
        return Err("only YouTube urls are supported".to_string());
    }
    Ok(())
}

// takes references (&Path) instead of owned values because i only need to read them
// Vec<String> is a growable list of owned strings, which is what Command::args expects
pub fn build_args(url: &str, output_dir: &Path, ffmpeg_path: &Path) -> Vec<String> {
    vec![
        // extract audio and convert to mp3
        "-x".to_string(),
        "--audio-format".to_string(),
        "mp3".to_string(),
        // quality 0 means best available, yt-dlp passes it straight to ffmpeg
        "--audio-quality".to_string(),
        "0".to_string(),
        // without this flag progress overwrites itself on one line, i need separate lines to parse
        "--newline".to_string(),
        // use the bundled ffmpeg, not whatever is on the system path
        "--ffmpeg-location".to_string(),
        // to_string_lossy handles paths with non-UTF8 characters safely
        // into_owned converts the result from a borrowed Cow<str> into an owned String
        ffmpeg_path.to_string_lossy().into_owned(),
        // avoid accidentally downloading an entire playlist if a playlist url slips through
        "--no-playlist".to_string(),
        "-P".to_string(),
        output_dir.to_string_lossy().into_owned(),
        "-o".to_string(),
        // yt-dlp output template: use the video title as the filename
        "%(title)s.%(ext)s".to_string(),
        url.to_string(),
    ]
}

// parses a single line from yt-dlp stderr into something the frontend can use
// Option<>: either Some(value) or None, never a null pointer
pub fn parse_progress(line: &str) -> Option<ProgressEvent> {
    if line.starts_with("[download]") && line.contains('%') {
        let percent_str = line
            .split_whitespace() //splits on any whitespace and gives us an iterator over the pieces
            .find(|s| s.ends_with('%'))? // walks the iterator and returns the first piece that ends with '%'
            .trim_end_matches('%');

        // parse() tries to convert the string into f32
        // ok() turns a Result into an Option, then ? returns None if parsing failed
        let percent: f32 = percent_str.parse().ok()?;
        return Some(ProgressEvent::Downloading { percent });
    }
    // ExtractAudio is yt-dlp, ffmpeg is the converter it delegates to
    if line.starts_with("[ExtractAudio]") || line.starts_with("[ffmpeg]") {
        return Some(ProgressEvent::Converting);
    }
    if line.starts_with("ERROR:") {
        return Some(ProgressEvent::Error(line.to_string()));
    }
    None // returns None for lines i dont care about (info, verbose, etc.)
}


#[cfg(test)]
mod tests {
    use super::*;

    // validate_url
    
    #[test]
    fn accepts_standard_watch_url() {
        assert!(validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_ok());
    }

    #[test]
    fn accepts_short_youtu_be_url() {
        assert!(validate_url("https://youtu.be/dQw4w9WgXcQ").is_ok());
    }

    #[test]
    fn accepts_shorts_url() {
        assert!(validate_url("https://www.youtube.com/shorts/abc123").is_ok());
    }

    #[test]
    fn rejects_empty_string() {
        assert!(validate_url("").is_err());
    }

    #[test]
    fn rejects_whitespace_only() {
        assert!(validate_url("   ").is_err());
    }

    #[test]
    fn rejects_non_youtube_url() {
        assert!(validate_url("https://vimeo.com/123456").is_err());
    }

    // build_args

    #[test]
    fn args_include_the_url() {
        let url = "https://youtu.be/abc";
        let args = build_args(url, Path::new("/tmp"), Path::new("/usr/bin/ffmpeg"));
        assert!(args.contains(&url.to_string()));
    }

    #[test]
    fn args_extract_to_mp3() {
        let args = build_args("https://youtu.be/abc", Path::new("/tmp"), Path::new("/usr/bin/ffmpeg"));
        assert!(args.contains(&"-x".to_string()));
        // position finds the index of --audio-format so i can check the value that follows it
        // unwrap is safe here because if the arg is missing the test should fail loudly
        let fmt_idx = args.iter().position(|a| a == "--audio-format").unwrap();
        assert_eq!(args[fmt_idx + 1], "mp3");
    }

    #[test]
    fn args_point_to_correct_output_dir() {
        let args = build_args("https://youtu.be/abc", Path::new("/music"), Path::new("/usr/bin/ffmpeg"));
        let p_idx = args.iter().position(|a| a == "-P").unwrap();
        assert_eq!(args[p_idx + 1], "/music");
    }

    #[test]
    fn args_use_bundled_ffmpeg() {
        let args = build_args("https://youtu.be/abc", Path::new("/tmp"), Path::new("/opt/ffmpeg"));
        let loc_idx = args.iter().position(|a| a == "--ffmpeg-location").unwrap();
        assert_eq!(args[loc_idx + 1], "/opt/ffmpeg");
    }

    #[test]
    fn args_disable_playlist() {
        let args = build_args("https://youtu.be/abc", Path::new("/tmp"), Path::new("/usr/bin/ffmpeg"));
        assert!(args.contains(&"--no-playlist".to_string()));
    }

    // parse_progress

    #[test]
    fn parses_mid_download_percentage() {
        let line = "[download]  50.3% of 5.00MiB at 1.00MiB/s ETA 00:02";
        assert_eq!(parse_progress(line), Some(ProgressEvent::Downloading { percent: 50.3 }));
    }

    #[test]
    fn parses_100_percent() {
        let line = "[download] 100% of 5.00MiB in 00:05";
        assert_eq!(parse_progress(line), Some(ProgressEvent::Downloading { percent: 100.0 }));
    }

    #[test]
    fn parses_extract_audio_as_converting() {
        let line = "[ExtractAudio] Destination: /tmp/song.mp3";
        assert_eq!(parse_progress(line), Some(ProgressEvent::Converting));
    }

    #[test]
    fn parses_ffmpeg_line_as_converting() {
        let line = "[ffmpeg] Merging formats into \"song.mp3\"";
        assert_eq!(parse_progress(line), Some(ProgressEvent::Converting));
    }

    #[test]
    fn parses_error_line() {
        let line = "ERROR: unable to download video";
        assert_eq!(parse_progress(line), Some(ProgressEvent::Error(line.to_string())));
    }

    #[test]
    fn ignores_info_lines() {
        assert_eq!(parse_progress("[info] some info message"), None);
    }
}
