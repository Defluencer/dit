ffmpeg -i udp://localhost:2525?fifo_size=1146880^&overrun_nonfatal=1 ^
-map v:0 -c:v:0 libx264 -preset: ultrafast -tune: zerolatency -g:0 120 -sc_threshold: 0 -b:v:0 6000k -s:0 1920x1080 -sws_flags bilinear -r:0 60 ^
-map v:0 -c:v:1 libx264 -g:1 120 -b:v:1 4500k -s:1 1280x720 -r:1 60 ^
-map v:0 -c:v:2 libx264 -g:2 60 -b:v:2 3000k -s:2 1280x720 -r:2 30 ^
-map v:0 -c:v:3 libx264 -g:3 60 -b:v:3 2000k -s:3 852x480 -r:3 30 ^
-map a:0 -map a:0 -map a:0 -map a:0 -c:a: aac -b:a: 192k -ar: 48000 -ac: 2 ^
-f hls -var_stream_map "v:0,a:0,name:1080p60 v:1,a:1,name:720p60 v:2,a:2,name:720p30 v:3,a:3,name:480p30" ^
-hls_init_time 4 -hls_time 4 -hls_list_size 5 -hls_flags independent_segments -master_pl_name master.m3u8 ^
-hls_segment_filename http://localhost:2526/%%v/%%d.ts ^
-http_persistent 1 -ignore_io_errors 1 ^
-method PUT http://localhost:2526/%%v/index.m3u8
pause