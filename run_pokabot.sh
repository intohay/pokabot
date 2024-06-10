export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH

cd /home/ubuntu/pokabot
/home/ubuntu/.cargo/bin/cargo run >> /home/ubuntu/pokabot/log/log.txt
