# 安装 PhpRedis

主要目标是安装 PhpRedis

## 本地方式

### 安装 PhpRedis

  - 安装PhpRedis: https://github.com/phpredis/phpredis/blob/develop/INSTALL.markdown
    - 国内安装: https://github.com.cnpmjs.org/phpredis/phpredis/blob/develop/INSTALL.markdown
  - Mac安装PhpRedis
    - Mac安装Pecl: https://pear.php.net/manual/en/installation.getting.php
    - sudo pecl channel-update https://pecl.php.net/channel.xml
    - sudo pecl update-channels
    - pecl search redis
    - sudo pecl install redis
    - Mac开启关闭SIP（系统完整性保护）
      - 重启MAC，按住cmd+R直到屏幕上出现苹果的标志和进度条，进入Recovery模式
      - 在屏幕最上方的工具栏找到实用工具（左数第3个），打开终端，输入：
        - csrutil disable
      - 重启mac
    - sudo vi /php.ini
      - extension=redis.so
    - 重启Apache: sudo apachectl restart

## FAQ

### PHP 7 需知

  - Since PHP 7 is not in the official Ubuntu PPAs,使用Composer install 会提示错误: Call to undefined function: simplexml_load_string(),解决办法在服务器上执行以下指令

    ```
    > sudo apt-get install php7.0-xml
    > sudo service php7.0-fpm restart
    ```
