ffmpeg -i udp://localhost:2525?fifo_size=114688^
 -c:v copy -c:a copy^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -ignore_io_errors true -method PUT http://localhost:2424/live/1080_60/1920x1080_60_.m3u8^
 -c:v libx264 -x264opts keyint=120:no-scenecut -s 1280x720 -r 60 -b:v 4500 -profile:v main -preset veryfast -c:a aac -sws_flags bilinear^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -ignore_io_errors true -method PUT http://localhost:2424/live/720_60/1280x720_60_.m3u8^
 -c:v libx264 -x264opts keyint=60:no-scenecut -s 1280x720 -r 30 -b:v 3000 -profile:v main -preset veryfast -c:a aac -sws_flags bilinear^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -ignore_io_errors true -method PUT http://localhost:2424/live/720_30/1280x720_30_.m3u8^
 -c:v libx264 -x264opts keyint=60:no-scenecut -s 852x480 -r 30 -b:v 2000 -profile:v main -preset veryfast -c:a aac -sws_flags bilinear^
 -f hls -hls_init_time 4 -hls_time 4 -hls_list_size 5 -ignore_io_errors true -method PUT http://localhost:2424/live/480_30/852x480_30_.m3u8
 pause