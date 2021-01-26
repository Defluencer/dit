#!/bin/bash
# Testing hardware transcoding -> too slow
ffmpeg -hwaccel vaapi -hwaccel_device /dev/dri/renderD128 -hwaccel_output_format vaapi \
-listen 1 -i rtmp://127.0.0.1:2525 -rtmp_live live -rtmp_buffer 8000 \
-filter_complex \
"[0:v]split=3[1080p60][in1][in2]; \
[in1]scale_vaapi=w=1280:h=720,split=2[720p60][scaleout]; \
[scaleout]fps=30[720p30]; \
[in2]fps=30,scale_vaapi=w=852:h=480[480p30]" \
-map '[1080p60]' -c:v:0 h264_vaapi -g:0 240 -b:v:0 6000k \
-map '[720p60]' -c:v:1 h264_vaapi -g:1 240 -b:v:1 4500k \
-map '[720p30]' -c:v:2 h264_vaapi -g:2 120 -b:v:2 3000k \
-map '[480p30]' -c:v:3 h264_vaapi -g:3 120 -b:v:3 2000k \
-map a:0 -map a:0 -map a:0 -map a:0 -c:a: copy \
-f hls -var_stream_map "v:0,a:0,name:1080p60 v:1,a:1,name:720p60 v:2,a:2,name:720p30 v:3,a:3,name:480p30" \
-hls_init_time 4 -hls_time 4 -hls_list_size 5 -hls_flags independent_segments -master_pl_name master.m3u8 \
-hls_segment_type fmp4 -hls_segment_filename http://192.168.1.152:2526/%v/%d.fmp4 \
-http_persistent 1 -ignore_io_errors 1 \
-method PUT http://192.168.1.152:2526/%v/index.m3u8