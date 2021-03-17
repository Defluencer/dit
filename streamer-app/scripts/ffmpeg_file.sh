#!/bin/bash
# BASH script example.
# FFMPEG configured to output multi quality HLS.
# Variants ordering MUST be highest to lowest quality.

echo -e "Where is the video file you would like to process?" 
read file

ffmpeg -i $file \
-filter_complex \
"[0:v]split=3[1080p60][in1][in2]; \
[in1]scale=w=1280:h=720,split=2[720p60][scaleout]; \
[scaleout]fps=30[720p30]; \
[in2]fps=30,scale=w=854:h=480[480p30]" \
-map '[1080p60]' -c:v:0 libx264 -preset: fast -g:0 240 -sc_threshold: 0 -b:v:0 12000k \
-map '[720p60]' -c:v:1 libx264 -g:1 240 -b:v:1 7500k \
-map '[720p30]' -c:v:2 libx264 -g:2 120 -b:v:2 5000k \
-map '[480p30]' -c:v:3 libx264 -g:3 120 -b:v:3 2500k \
-map a:0 -map a:0 -map a:0 -map a:0 -c:a: copy \
-f hls -var_stream_map "v:0,a:0,name:1080p60 v:1,a:1,name:720p60 v:2,a:2,name:720p30 v:3,a:3,name:480p30" \
-hls_init_time 4 -hls_time 4 -hls_flags independent_segments -master_pl_name master.m3u8 \
-hls_segment_type fmp4 -hls_segment_filename http://localhost:2526/%v/%d.fmp4 \
-http_persistent 0 -ignore_io_errors 1 -method PUT http://localhost:2526/%v/index.m3u8