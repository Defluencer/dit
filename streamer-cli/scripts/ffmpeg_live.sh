#!/bin/bash
# BASH script example.

# FFMPEG configured to output live multi quality HLS with standalone audio track.

# Tips: forcing key frame at interval make it easy to cut segment of exact length and allow scene change detection.

ffmpeg -listen 1 -i rtmp://localhost:2525 -rtmp_live live -rtmp_buffer 8000 \
-filter_complex \
"[0:v]split=3[1080p60][in1][in2]; \
[in1]scale=w=1280:h=720,split=2[720p60][scaleout]; \
[scaleout]fps=30[720p30]; \
[in2]fps=30,scale=w=854:h=480[480p30]" \
-map '[1080p60]' -c:v:0 libx264 -preset: ultrafast -rc-lookahead:0 60 -g:0 120 -keyint_min:0 60 -force_key_frames:0 "expr:eq(mod(n,60),0)" -b:v:0 6000k \
-map '[720p60]' -c:v:1 libx264 -rc-lookahead:1 60 -g:1 120 -keyint_min:1 60 -force_key_frames:1 "expr:eq(mod(n,60),0)" -b:v:1 4500k \
-map '[720p30]' -c:v:2 libx264 -rc-lookahead:2 30 -g:2 60 -keyint_min:2 30 -force_key_frames:2 "expr:eq(mod(n,30),0)" -b:v:2 3000k \
-map '[480p30]' -c:v:3 libx264 -rc-lookahead:3 30 -g:3 60 -keyint_min:3 30 -force_key_frames:3 "expr:eq(mod(n,30),0)" -b:v:3 2000k \
-map a:0 -c:a:0 copy \
-f hls -var_stream_map "v:0,name:1080p60 v:1,name:720p60 v:2,name:720p30 v:3,name:480p30 a:0,name:audio" \
-hls_init_time 1 -hls_time 1 -hls_flags independent_segments -master_pl_name master.m3u8 \
-hls_segment_type fmp4 -hls_segment_filename http://localhost:2526/%v/%d.m4s \
-http_persistent 0 -ignore_io_errors 1 -method PUT http://localhost:2526/%v/index.m3u8