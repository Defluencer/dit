use tokio::process::Command;

pub async fn start_transcoding(ffmpeg_addr: String, stream_app_addr: String) {
    let mut command = Command::new("ffmpeg");

    command
        .args(&[
            "-listen",
            "1",
            "-i",
            &format!("rtmp://{}", ffmpeg_addr),
            "-rtmp_live",
            "live",
            "-rtmp_buffer",
            "8000",
        ])
        .args(&[
            "-map",
            "v:0",
            "-c:v:0",
            "libx264",
            "-preset:",
            "ultrafast",
            "-tune:",
            "zerolatency",
            "-g:0",
            "240",
            "-sc_threshold:",
            "0",
            "-b:v:0",
            "6000k",
            "-s:0",
            "1920x1080",
            "-sws_flags",
            "bilinear",
            "-r:0",
            "60",
        ])
        .args(&[
            "-map", "v:0", "-c:v:1", "libx264", "-g:1", "240", "-b:v:1", "4500k", "-s:1",
            "1280x720", "-r:1", "60",
        ])
        .args(&[
            "-map", "v:0", "-c:v:2", "libx264", "-g:2", "120", "-b:v:2", "3000k", "-s:2",
            "1280x720", "-r:2", "30",
        ])
        .args(&[
            "-map", "v:0", "-c:v:3", "libx264", "-g:3", "120", "-b:v:3", "2000k", "-s:3",
            "852x480", "-r:3", "30",
        ])
        .args(&[
            "-map", "a:0", "-map", "a:0", "-map", "a:0", "-map", "a:0", "-c:a:", "aac", "-b:a:",
            "192k", "-ar:", "48000", "-ac:", "2",
        ])
        .args(&[
            "-f",
            "hls",
            "-var_stream_map",
            "v:0,a:0,name:1080p60 v:1,a:1,name:720p60 v:2,a:2,name:720p30 v:3,a:3,name:480p30",
            "-hls_init_time",
            "4",
            "-hls_time",
            "4",
            "-hls_list_size",
            "5",
            "-hls_flags",
            "independent_segments",
            "-master_pl_name",
            "master.m3u8",
            "-hls_segment_type",
            "fmp4",
            "-hls_segment_filename",
            &format!("http://{}/%v/%d.fmp4", stream_app_addr),
            "-http_persistent",
            "1",
            "-ignore_io_errors",
            "1",
            "-method",
            "PUT",
            &format!("http://{}/%v/index.m3u8", stream_app_addr),
        ]);

    #[cfg(target_os = "windows")]
    command.creation_flags(0x00000010); //https://docs.microsoft.com/en-us/windows/win32/procthread/process-creation-flags

    let handle = match command.spawn() {
        Ok(result) => {
            println!("Transcoding System Online");

            result
        }
        Err(e) => {
            eprintln!("FFMPEG command failed to start. {}", e);
            return;
        }
    };

    if let Err(e) = handle.await {
        eprintln!("FFMPEG command failed to run. {}", e);
    }
}
