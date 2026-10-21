@echo off
D:\ffmpeg\ffmpeg\bin\ffmpeg.exe -re -i "D:\ffplayout-master\ffplayout-master\demo_data\storage\Channel 1\AI-60-sec-promo.mp4" -map 0:v:0 -map 0:a:0 -c:v libx264 -preset veryfast -tune zerolatency -c:a aac -f mpegts "udp://127.0.0.1:9001?pkt_size=1316"
