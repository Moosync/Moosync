curl -s --compressed "https://moosync.app/ppa/ubuntu/KEY.gpg" | sudo apt-key add -
sudo curl -s --compressed -o /etc/apt/sources.list.d/moosync.list "https://moosync.app/ppa/ubuntu/moosync.list"
sudo apt update
sudo apt install moosync