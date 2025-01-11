# Helpful commands
Find and remove test proccess
ps -ef | grep -v grep | grep Loki | awk '{print $2}' | xargs -I {} kill -9 {}
