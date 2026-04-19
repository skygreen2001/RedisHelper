# Online Redis Manager

符合中国开发者思维方式的在线Redis管理工具的框架，设计初衷快捷、简单、实用。

主要采用betterlife框架、betterlife.front框架中[web/vuejs]模块快速开发而成。

底层前端主要使用了Vuejs、iView框架；后端使用了PhpRedis、PhpSpreadsheet框架。

## 帮助文档

- [帮助文档](help/HELP.md)

## 下载源码

下载地址: https://gitee.com/skygreen2015/RedisManager.git

## 通常安装

### 运行环境安装

  以下工具任选一种即可

  - [ampps](http://www.ampps.com)

    可以直接在它上面下载安装(Wamp|Lamp|Mamp)

  - [Wamp](http://www.wampserver.com/en/)

    Windows下的Apache + Mysql + PHP
    [PhpStudy]: http://www.phpstudy.net/

  - [Lamp](https://lamp.sh/)

    LAMP指的Linux、Apache，MySQL和PHP的第一个字母
    [安装详细说明]: http://blog.csdn.net/skygreen_2001/article/details/19912159

  - [Mamp](http://www.mamp.info/en/)

    Mac环境下搭建 Apache/Nginx、MySQL、Perl/PHP/Python 平台。

  - [Xampp](https://www.apachefriends.org/zh_cn/index.html)

    XAMPP是完全免费且易于安装的Apache发行版，其中包含MariaDB、PHP和Perl。

  - [宝塔](https://www.bt.cn/)

  - [PhpStud](https://www.xp.cn/)

  - 本地运行PHP server: php -S localhost:8000)

### 其它安装

  - [安装 PhpRedis和PHP第三方库](install/README.md)
    - 安装 PhpRedis

## **使用说明**

- 浏览器打开网页: http://localhost/www/index.html
- 菜单选择: 设备 -> 设置 -> 新增
  - 服务器地址: redis
  - 端口     : 6379
  - 密码     : orm

## 框架目录定义

  - core   : 框架核心支持文件
    - core/config      : 配置文件[各个功能模块]
    - core/include     : 常用的函数库
  - install: 安装目录
  - api    : ajax请求服务端服务支持[手机或Web前端ajax请求返回json数据]
  - www    : redis在线编辑器前端页面
  - log    : 日志目录，每天一个调试测试日志文件放在这里
  - upload : 后台上传下载文件(如excel)放置目录 

## 服务器配置模式

  - 在 www/js/main.js 里配置 isConfigLocal
    - isConfigLocal: true, 意味着服务器配置都持久化存储在本地
    - isConfigLocal: false, 意味着服务器配置都持久化存储在服务器上，建议内部本地使用，可只需配置一次。
    - 默认存储在本地浏览器localStorage里，确保使用工具千人千面。


## 目录权限设置

  - 如果是Linux、Mac 系统，需要设置以下目录权限为完全可读写。
    - log     
    - upload 
    - 执行命令如下:
      > sudo mkdir log/ upload/
      > sudo chmod -R 0777 log/ upload/

## 开发工具

* [TRAE CN](https://www.trae.cn/)
* [Visual Studio Code](https://code.visualstudio.com/)
* [Atom](https://atom.io)
* [Atom IDE](https://ide.atom.io/)
* [Sublime](http://www.sublimetext.com)

## 参考资料

* **Betterlife**
  > https://gitee.com/skygreen2015/betterlife
* **Betterlife.Front**
  > https://gitee.com/skygreen2015/betterlife.front
* **Redis**
  > https://redis.io
* **本地运行服务器**
  > install/LINUX.md
