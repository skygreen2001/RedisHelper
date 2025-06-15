# Linux 常用指令集

## 本地运行服务器

  > https://threejs.org/docs/index.html#manual/en/introduction/How-to-run-things-locally
  - PHP server
    > php -S localhost:8000

  - Python server
    //Python 2.x
    python -m SimpleHTTPServer
    python -m SimpleHTTPServer 80

    //Python 3.x
    python -m http.server

  - Npx in Node.js
    npx http-server

  - Node.js Server
    > npm install http-server -g
    > http-server . -p 8000

  - Ruby server
    > ruby -r webrick -e "s = WEBrick::HTTPServer.new(:Port => 8000, :DocumentRoot => Dir.pwd); trap('INT') { s.shutdown }; s.start"
 
  - Lighttpd
    > brew install lighttpd
    - 编辑配置文件:lighttpd.conf
      > http://redmine.lighttpd.net/projects/lighttpd/wiki/TutorialConfiguration

    > lighttpd -f lighttpd.conf

## 安装Ubuntu系统初始化

  - 初始化，安装编译环境
    ```
    sudo apt-get update
    sudo apt-get upgrade
    sudo apt-get install build-essential
    ```

## 优化性能

  - Linux下查看Web服务器当前的并发连接数和TCP连接状态
    ```
    netstat -n | awk '/^tcp/ {++S[$NF]} END {for(key in S) print key,"\t",S[key]}'
    ```

  - 查看空闲内存量
    ```
    grep MemFree /proc/meminfo
    watch free -m
    ```

  - 清除虚拟内存
    - [在 Linux 上如何清除内存的 Cache、Buffer 和交换空间](https://linux.cn/article-5627-1.html)
    - [11 Cron Scheduling Task Examples in Linux](https://www.tecmint.com/11-cron-scheduling-task-examples-in-linux/)
      ```
      sync; echo 1 > /proc/sys/vm/drop_caches
      ```

  - 查看内存占有率
      top


## 系统

  - 查看操作系统: cat /etc/os-release
  - 查看操作系统信息: sudo lsb_release -a
  - 关机: shutdown -h now
  - 重启: reboot -h now
  - 查看Linux发行版本: cat /proc/version
  - 查看CPU的核数、信息: lscpu
    - 查看CPU的核数: cat /proc/cpuinfo | grep 'processor' | wc -l
  - 查看内核/操作系统/CPU信息: uname -a
  - 查看操作系统版本: head -n 1 /etc/issue
    - 查看ubuntu版本号: cat /etc/issue
  - 查看CPU信息: cat /proc/cpuinfo
  - 查看计算机名: hostname
  - 列出所有PCI设备: lspci -tv
  - 列出所有USB设备: lsusb -tv
  - 列出加载的内核模块: lsmod
  - 查看环境变量: env
  - 只适合Redhat系的Linux: cat /etc/redhat-release

  - 服务
    - Ubuntu: service --status-all
    - 移除自启动服务: update-rc.d -f apache2 remove
    - Centos查看所有安装的软件包: rpm -qa
    - 列出所有系统服务 Centos: chkconfig --list
    - 列出所有启动的系统服务 Centos: chkconfig --list | grep on

  - 修改电脑名称
    ```
    sudo vi /etc/hostname
    hostname bb
    ```
  - 查看本机外网ip地址: curl http://icanhazip.com
  - 查看本机公网ip地址(包括头信息): curl -s http://myip.ipip.net -i

  - 修改服务器时间: date -s 23:59:59
  - 修改服务器日期: date -s 11/03/2021 (修改为2021年11月3日)
  - 日期时间重置: 
    - apt install ntpdate
    - ntpdate -u ntp.api.bz

  - 检查时间是否和本地一致，如果不正确，调整时区
    ```
    cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
    将当前时间和日期写入BIOS，避免重启后失效
    hwclock
    ```
  - 同步时间
    Alpine Linux: ntpd -d -q -n -p ntp3.aliyun.com，立即同步时间，不限制同步间隔
    CentOS/Ubuntu: sudo apt-get install ntpdate  # 或者 yum
                   ntpdate ntp3.aliyun.com
                   
  - Linux 查看连接数
    - 查询TCP连接数: netstat -ant|awk '/^tcp/ {++S[$NF]} END {for(a in S) print (a,S[a])}'
    - 80端口: netstat -na | grep ESTAB | grep 80 | wc -l
    - 8888端口:
      ```
      netstat -na | grep ESTAB | grep 8888 | wc -l
      netstat -ant | grep  8888 | grep TIME | wc -l
      ```
    - 3306连接数: netstat -ant | grep 3306 | wc -l

  - Linux下查看Web服务器当前的并发连接数和TCP连接状态
    ```
    netstat -n | awk '/^tcp/ {++S[$NF]} END {for(key in S) print key,"\t",S[key]}'
    ```

  - Linux 自动断开连接
    ```
    sudo vi /etc/ssh/ssh_config
    添加内容: ServerAliveInterval 60
    ```

## 资源

  - 查看服务的进程
    ps -ef|grep ssh
    - ps -aux ,然后再利用一个管道符号导向到grep去查找特定的进程,然后再对特定的进程进行操作

  - 查看服务安装的地址
    whereis iptables

  - 查看指定端口：
    lsof -i:port    lsof -i:8090
    netstat -ltnp | grep port | netstat -ltnp | grep 443

  - 查看进程占用的端口:
    netstat -antup |grep PID

  - 杀进程：
    kill -9 processId
    示例: 杀死端口为80的进程
         kill -9 $(lsof -ti:80)

  - 查看内存使用量和交换区使用量: free -m
  - 查看内存总量: grep MemTotal /proc/meminfo
  - 查看空闲内存量: grep MemFree /proc/meminfo
  - 查看系统运行时间、用户数、负载: uptime
  - 查看系统负载: cat /proc/loadavg

  - 虚拟内存
    - 参考: ubuntu下配置虚拟内存: https://blog.csdn.net/qq_38701476/article/details/83042668
    - 创建虚拟内存配置文件
      ```
      cd /root/
      mkdir swap
      cd swap
      sudo dd if=/dev/zero of=swapfile bs=2048 count=300000
      ```
    - 把生成的文件转换成 Swap 文件
      ```
      sudo mkswap swapfile
      ```
    - 激活swap文件
      ```
      sudo swapon swapfile
      ```
    - 如果要卸载
      ```
      sudo swapoff swapfile
      ```
    - 此时开的虚拟内存会在开机后消失,如果永久保持下去,在/etc/fstab文件尾添加一下信息:
      ```
      /root/swap/swapfile swap swap defaults 0 0
      ```

## 磁盘和分区

  - 查看各分区使用情况: df -h
  - 查看磁盘的使用情况以及文件系统被挂载的位置: df -lh
  - 查看挂接的分区状态: mount | column -t
  - 查看所有分区: fdisk -l
  - 查看所有交换分区: swapon -s
  - 查看磁盘参数(仅适用于IDE设备): hdparm -i /dev/hda
  - 查看启动时IDE设备检测状况: dmesg | grep IDE
  - 挂载数据盘
    ```
    sudo fdisk -l
    sudo mount /dev/xvdb1 /mnt
    ```

  - 查找Linux系统中的占用磁盘空间最大的前10个文件或文件夹
    ```
    cd /path/to/some/where
    du -hsx * | sort -rh | head -10
    ```

  - 查看指定目录占用空间大小: du -sh /opt/* | sort -nr
  - 查看指定目录的大小: du -sh <目录名>
  - 查看指定目录文件大小: du -h --max-depth=1 /mnt/www/bb/
  - 快速查看文件列表大小: du -ahd1cd /
  - 在根目录下检查磁盘占用情况: du -h --max-depth=1 /

  - 查看已删除文件占用磁盘: lsof | grep delete
  - 清空日志文件: echo '' > /var/log/rinetd.log
  - 重启rinetd: pkill rinetd && rinetd
  - linux磁盘清理: https://blog.csdn.net/debug6699/article/details/102140458 
  - 查找Linux系统中的占用磁盘空间最大的前10个文件或文件夹，如果需要输出可读性高的内容，请使用如下命令：
    ```
    cd /path/to/some/where
    du -hsx * | sort -rh | head -10
    ```

  - 清理日志: journalctl --vacuum-size=10M
    - Linux 系统 /var/log/journal/ 垃圾日志清理: https://blog.mimvp.com/article/30995.html

## 文件管理

  - 查找文件
    find . -name 'dd*'

  - 删除指定目录下所有文件带有._前缀的所有文件
    find . -name '\.\_*' | sudo xargs rm -rf

  - 删除目录文件
    - 递归删除文件: find . -name "._*"  | xargs rm -f
    - 递归删除目录: find . -name "._*"  | xargs rm -rf

  - 将当前目录及其子目录下所有最近 20 天内更新过的文件列出: find . -ctime -20

  - 统计当前目录下，子目录中文件的个数
    for a in `find .  -mindepth 1 -maxdepth 1 -type d|sort -r`; do echo "$a:`find $a |wc -l`"; done

  - [压缩与解压缩]: http://man.openbsd.org/tar
    - 压缩: tar -czvf ***.tar.gz 來源檔案1 來源檔案2 ... 來源檔案n
    - 解压: tar -zxvf 檔案名稱.tar.gz
           tar -jxvf ×××.tar.bz2
    - 查看压缩文件[不解压]: tar tvf ***.tar.gz
      - [How to Compress and Extract Files Using the tar Command on Linux](https://www.howtogeek.com/248780/how-to-compress-and-extract-files-using-the-tar-command-on-linux/)
    - tar 指令的常用語法: http://www.vixual.net/blog/archives/127
    - unzip -o -d ./ bb.zip   解压到当前路径
    - zip -r img.zip bak/images/  (apt install zip)

  - 复制文件夹到服务器
    ```
    scp -r 文件夹  root@debug.bb.com:~/www/
    rsync -P --rsh=ssh root@dev.bb.com:/var/www/html/www.tar.gz .
    alias scpr="rsync -P --rsh=ssh"
    scpr root@dev.bb.com:/var/www/html/www.tar.gz .
    ```

  - 下载
    ```
    curl -o zk.tar.gz https://mirrors.tuna.tsinghua.edu.cn/apache/zookeeper/zookeeper-3.4.11/zookeeper-3.4.11.tar.gz
    curl -O 下载文件网络地址
    wget 下载文件网络地址
    ```

  - 系统语言
    - sudo apt-get -y install language-pack-zh-hans
    - sudo apt-get -y install language-pack-zh-hans language-pack-zh-hans-base
    - sudo apt-get -y install language-pack-zh-hant
    - sudo apt-get -y install language-pack-zh-hant language-pack-zh-hant-base
    - 执行：locale-gen
    - vi /etc/profile  加入
      export LC_ALL="zh_CN.UTF-8"

    - 执行：source /etc/profile
      - export LC_ALL="zh_CN.UTF-8"
      - 执行 locale ，看是否显示 zh_CN.UTF-8

    - dpkg-reconfigure locales

  - 修改文件乱码问题
    ```
    vi /etc/vim/vimrc
      set fileencodings=utf-8,gb2312,gbk,gb18030
      set termencoding=utf-8
      set encoding=prc
      set enc=utf8
      set fencs=utf8,gbk,gb2312,gb18030
    ```

  - 字体
    - 查看 ubuntu 系统自带的字体: fc-list :lang-zh | sort
    - 安装中文字体
      - 获取icloud中dev中文字体font目录
        ```
        sudo chmod 644 /root/font/*
        sudo cp /root/font/* /usr/share/fonts
        cd /usr/share/fonts
        sudo mkfontscale && sudo mkfontdir
        sudo fc-cache -fsv
        ```
      - 下载文泉驿字体
        ```
        sudo apt-get install ttf-wqy-zenhei
        ```

    - 查看所有的中文字体: fc-list :lang=zh

## 日志

  - 查看日志: tail -f -n 2500 catalina.out

  - 查看日志文件时间超过 10秒的日志: cat prod/tomcat9/logs/catalina.2024-02-17.out | grep '[0-9][0-9][0-9][0-9][0-9]MS' -n

  - 实时查看日志文件时间超过 10秒的日志: tail -f -n 2500 prod/tomcat9/logs/catalina.2024-02-17.out | grep '[0-9][0-9][0-9][0-9][0-9]MS' -n

  - 查看日志[请求耗时]: tail -f -n 2500 prod/tomcat9/logs/catalina.2024-02-17.out | grep 'SPEND TIME'

  - Linux grep、egrep使用命令详解: https://www.jianshu.com/p/1aa58d24a4b1

  - Linux的正则表达式: https://blog.csdn.net/Luxus_sing/article/details/104711186

## 网络

  - 查看所有网络接口的属性: ifconfig
  - 查看防火墙设置: iptables -L

  - 修改防火墙并生效
    - 查看防火墙规则: iptables -L
    - 编辑文件: vi /etc/iptables.rules
    - 使防火墙规则生效: iptables-restore < /etc/iptables.rules

  - 查看路由表: route -n
  - 查看所有监听端口: netstat -lntp
  - 查看所有已经建立的连接: netstat -antp
  - 查看网络统计信息: netstat -s
  - 查看有多少远程的 IP 在连接本机: netstat -atn | awk '{print $5}'| awk '{print $1}'| sort -nr | uniq -c


## 进程

  - 查看所有进程: ps -ef
  - 实时显示进程状态: top
  - 过滤运行程序占用的进程: ps -ef | grep catalina.out | cut -c 9-15 | xargs
  - 杀死过滤运行程序占用的进程: ps -ef | grep catalina.out | cut -c 9-15 | xargs kill -9
  - 运行结束进程
    - 开始: nohup ./timer.py > timer.log 2>&1 & echo $! > run.pid
    - 结束:查看 ps -ef | grep timer.py

## 用户

  - 修改当前用户的密码: passwd
  - 查看活动用户: w
  - 可以查看所有用户的列表: cat /etc/passwd
  - 查看指定用户信息: id <用户名>
  - 查看用户登录日志: last
  - 查看系统所有用户: cut -d: -f1 /etc/passwd
  - 查看系统所有组: cut -d: -f1 /etc/group 
  - 创建用户: adduser username
  - 赋予新建用户管理员权限: usermod -g root username
  - 杀死线程:kill pid

## 定时任务

  - 查看当前用户的计划任务: crontab -l
  - 工具地址: https://crontab.guru
  - 编辑crontab文件: crontab -e
    - crontab 定时任务: http://linuxtools-rst.readthedocs.io/zh_CN/latest/tool/crontab.html
  - vi /etc/rsyslog.d/50-default.conf
    - 打开文件，在文件中找到cron.*，把前面的#去掉，保存退出
  - sudo service rsyslog restart
  - vi /root/clear.sh
    ```
    #!/bin/bash
    #echo "Hello Skygreen!" >> /root/test.txt;
    sync; echo 1 > /proc/sys/vm/drop_caches;
    ```
  - vi /root/msgserver.sh
    - 0 1 * * * /etc/init.d/msg-server restart #凌晨1点重启消息服务
  - 首先要设置环境变量EDITOR。cron进程根据它来确定使用哪个编辑器编辑crontab文件。UNIX和LINUX用户都使用vi，编辑$HOME目录下的~/.profile文件，在其中加入这样一行:
    EDITOR=vi; export EDITOR
  - source ~/.profile
  - crontab -e
    ```
    0 5 * * * /root/./clear.sh
    ```
  - service cron restart
  - 日志文件: vi /var/log/cron.log

## vi
  - 中文乱码修改文件: /etc/vim/vimrc
    :set encoding=utf-8
  - show line number command
    :set nu

  - hide line number command
    :set nu!

  - 顶部: :1 或 gg
  - 底部: :$ 或 G （大写。当前若大小写锁定直接按g，未锁定则按shift+g）
  - 向前: n
  - 向后: Shift+#
  - 行首: Shift+6   [I是定位到行首]
  - 行尾: Shift+4   [A是定位到行末]

## 开发运维

  - ssh 快速登录服务器
    ```
    Mac: sudo vi ~/.bash_profile
         source ~/.bash_profile
    Linux: vi ~/.bashrc
           source ~/.bashrc
    ```
    
    - 默认导航至指定目录
      ```
      alias www='ssh root@www.bb.com -t "cd /var/www/html/bb/ ; bash"'
      ```

  - Git

    - 生成ssh id_rsa 公钥私钥密钥
      ```
      ssh -t RSA -C "bb@bb.com"
      ```

    - 浏览器访问github快: 修改/etc/hosts,修改github和ip地址的映射
      - [国内加速访问Github的办法，超级简单](https://blog.51cto.com/wangshiyu/3050245)
      - http://ping.chinaz.com/github.com，查询最快节点 IP 进行访问。
        ```
        github.com
        github.global.ssl.fastly.net
        assets-cdn.github.com
        github-cloud.s3.amazonaws.com
        ```
        
    - 提速git clone: https://www.zhihu.com/question/27159393
      - 只需要将 www.github.com/后面为代码库 改为 www.github.com.cnpmjs.org/后面为代码库地址 就可以实现一键式加速。
      - 只能对https有效，对git无效

    - 安装git
      sudo apt-get update
      sudo apt-get install git
      
    - 未提交代码前:
      - 列表所有本次修改的文件: git status | grep modified | awk '{print $2}' > list.txt
      - 压缩所有本次修改的文件: sudo rm -rf target && sudo mkdir target && sudo chmod -R 0777 target && sudo git status | grep modified | awk '{print $2}' | xargs zip target/update.zip
      - 参考: 将git修改的文件按照原来的目录层次全部复制出来: https://blog.csdn.net/davidhopper/article/details/90410419

    - 提交代码后:
      - 比较两次提交修改的文件: sudo git diff 6fbc462 758c36caf0 --name-only
      - 压缩比较两次提交修改过的文件: sudo rm -rf target && sudo mkdir target && sudo chmod -R 0777 target && sudo git diff 6fbc462 758c36caf0 --name-only | xargs zip target/update.zip

    - Git基本操作、常用命令、代码打包及代码发布: https://blog.csdn.net/daojibruce/article/details/80292158

  - nginx
    - 安装: apt-get install nginx
    - 配置文件: /etc/nginx/sites-enabled/
    - 判断Nginx配置是否正确: nginx -t
    - 重启: nginx -s reload
    - 修改上传文件大小限制: /etc/nginx/nginx.conf 配置中添加client_max_body_size 10m;

  - mysql:
    - 查看mysql版本号: select version();
    - 查看当前连接数: show full processlist;
    - 查看允许最大连接数: show variables like 'max_connections';
    - wait_timeout: 86400
    - MySQL cheatsheet: https://devhints.io/mysql
    - 查看数据库大小
      SELECT table_schema,SUM(AVG_ROW_LENGTH*TABLE_ROWS+INDEX_LENGTH)/1024/1024 AS total_mb
      FROM information_schema.TABLES group by table_schema;

  - linux 安装前端环境
    - 安装sudo
      ```
      apt update && apt upgrade
      apt-get install sudo
      ```
      - 命令行执行: sudo visudo
      - 添加两行:
        - www-data ALL=(root) ALL
        - %www-data ALL=(ALL:ALL) NOPASSWD:ALL
      - 重启apache或者nginx
      
    - 安装Node:
      ```
      curl -sL https://deb.nodesource.com/setup_10.x | sudo bash -
      sudo apt-get install -y nodejs
      ```

  - ubuntu 安装前端环境
    - 安装Node:
      ```
      curl -sL https://deb.nodesource.com/setup_10.x | sudo bash -
      sudo apt-get install -y nodejs
      ```
    - 安装Npm: apt install npm
    - 安装n: 
      ```
      npm install -g n
      sudo n 10.10.0   (安装制定版本的node)
      ```
    - 安装pm2: npm install pm2 -g
    - 安装yarn:

      ```
      curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
      echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
      sudo apt-get update && sudo apt-get install yarn
      ```

    - 国内快速安装
      ```
      npm install -gd express --registry=http://registry.npm.taobao.org
      npm config set registry https://registry.npm.taobao.org
      ```

      - 安装sass
        ```
        sudo npm i -f
        sudo npm install --unsafe-perm node-sass
        sudo npm install -g --unsafe-perm node-sass
        ```

      - 使用老版本node
        - 安装nvm: https://www.jianshu.com/p/622ad36ee020
          - curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.1/install.sh | bash
          - export NVM_DIR="$([ -z "${XDG_CONFIG_HOME-}" ] && printf %s "${HOME}/.nvm" || printf %s "${XDG_CONFIG_HOME}/nvm")"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" # This loads nvm
        ```
        sudo npm install gulp --unsafe-perm -g
        sudo npm i natives
        nvm install 10.15.3
        ```

  - 修改Linux服务器更好的支持Timeout

    - vi /etc/sysctl.conf
      ```
      # net.ipv4.tcp_max_syn_backlog = 1024

      # 表示开启重用。允许将TIME-WAIT sockets重新用于新的TCP连接，默认为0，表示关闭；
      net.ipv4.tcp_tw_reuse = 1

      # 表示开启TCP连接中TIME-WAIT sockets的快速回收，默认为0，表示关闭。
      net.ipv4.tcp_tw_recycle = 1

      # 修改系统默认的 TIMEOUT 时间
      net.ipv4.tcp_fin_timeout = 30

      #表示当keepalive起用的时候，TCP发送keepalive消息的频度。缺省是2小时，改为20分钟。
      net.ipv4.tcp_keepalive_time = 1200

      # 表示SYN队列的长度，默认为1024，加大队列长度为8192，可以容纳更多等待连接的网络连接数。
      net.ipv4.tcp_max_syn_backlog = 8192

      # 表示用于向外连接的端口范围。缺省情况下很小：32768到61000，改为1024到65000。
      net.ipv4.ip_local_port_range = 1024 65000 


      # 详解Linux服务器最大tcp连接数: https://www.cnblogs.com/fjping0606/p/4729389.html
      # 查看tcp最大连接数: ulimit -n 
      fs.file-max = 1000000
      # net.ipv4.ip_conntrack_max = 1000000
      # net.ipv4.netfilter.ip_conntrack_max = 1000000
      ```

    - 重启 /sbin/sysctl -p
    - ulimit -SHn 1000000

## 压力测试

  - watch uptime
    what uptime

  - Siege [并发测试]
    https://drupalize.me/blog/201507/load-testing-your-site-siege

  - (转载)Mac下使用Web服务器性能/压力测试工具webbench、ab、siege

  - WeTest
    https://zhuanlan.zhihu.com/p/22530523


## java

- 查询占用CPU最高的线程所在的代码问题:jstack PID 
    1.top -p 找到java的PID
    2.top -H -p Java的PID
    3.找到cpu最高的PID2
    4.printf "0x%x\n" PID2 获得一个16进制的AB( 如：0x7280 )
    5. jstack PID|grep -A 20 AB(如：0x609b)

- linux下分析java程序占用CPU、内存过高: https://www.cnblogs.com/wu-wu/p/11923250.html



## 参考

  - 鳥哥的 Linux 私房菜: http://linux.vbird.org/
  - Linux 命令大全: https://www.runoob.com/linux/linux-command-manual.html
  - Bash scripting cheatsheet: https://devhints.io/bash
  - 命令行的艺术: https://github.com/jlevy/the-art-of-command-line
  - 奇妙的 Linux 世界: https://www.hi-linux.com/
  - linux运维: https://www.kancloud.cn/noahs/linux/878498

  - ☁️ 云产品
    - 阿里云: http://www.aliyun.com/
    - 腾讯云: https://cloud.tencent.com
             412731900
             skygreen2001@qq.com
    - 金山云: https://www.ksyun.com/
    - 百度云: https://cloud.baidu.com/
    - 美团云: https://www.mtyun.com/
    - 华为云: http://www.hwclouds.com/
    - 网易云: https://www.163yun.com/
    - 华为云: https://www.huaweicloud.com/
    - 小鸟云: https://www.niaoyun.com/
    - ucloud: https://console.ucloud.cn/dashboard
