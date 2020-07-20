ffmpeg -i udp://localhost:2525?fifo_size=114688^
 -c:v copy -c:a copy^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -hls_flags temp_file+delete_segments+independent_segments -strftime 1 -strftime_mkdir 1 1080_60/playlist.m3u8^
 -c:v libx264 -x264opts keyint=120:no-scenecut -s 1280x720 -r 60 -b:v 4500 -profile:v main -preset veryfast -sws_flags bilinear -c:a aac^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -hls_flags temp_file+delete_segments+independent_segments -strftime 1 -strftime_mkdir 1 720_60/playlist.m3u8^
 -c:v libx264 -x264opts keyint=60:no-scenecut -s 1280x720 -r 30 -b:v 3000 -profile:v main -preset veryfast -sws_flags bilinear -c:a aac^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -hls_flags temp_file+delete_segments+independent_segments -strftime 1 -strftime_mkdir 1 720_30/playlist.m3u8^
 -c:v libx264 -x264opts keyint=60:no-scenecut -s 852x480 -r 30 -b:v 2000 -profile:v main -preset veryfast -sws_flags bilinear -c:a aac ^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -hls_flags temp_file+delete_segments+independent_segments -strftime 1 -strftime_mkdir 1 480_30/playlist.m3u8
 pause