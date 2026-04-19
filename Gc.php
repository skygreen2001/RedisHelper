<?php
//加载枚举类型定义
class_exists('Enum') || require(__DIR__ . '/core/Enum.php');

/**
 +-----------------------------------<br/>
 * 定义全局使用变量<br/>
 +------------------------------------
 * @access public
 */
class Gc {
    //<editor-fold desc='网站使用设置'>
    /**
     * 是否打开Debug模式
     * @var bool
     * @static
     */
    public static $dev_debug_on = true;
    /**
     * 是否开放php提供的debug信息
     * @var bool
     * @static
     */
    public static $dev_php_debug_on = false;
    /**
     * 网站应用的名称<br/>
     * 展示给网站用户
     * @var string
     * @static
     */
    public static $site_name = 'Redis 小助手';
    /**
     * 网站应用的版本
     */
    public static $version = '1.0.0';
    /**
     * 网站根路径的URL路径
     * @var string
     * @static
     */
    public static $url_base;//='http://localhost/betterlife/';//获取网站URL根路径
    /**
     * 网站根路径的物理路径
     * @var string
     * @static
     */
    public static $nav_root_path;//='C:\\wamp\\www\\betterlife\\';
    /**
     * 框架文件所在的路径 <br/>
     * 有两种策略可以部署<br/>
     * 1.框架和应用整合在一起；则路径同$nav_root_path   <br/>
     * 2.框架和应用分开，在php.ini里设置include_path='';添加框架所在的路径<br/>
     *                   则可以直接通过  <br/>
     * @var string
     * @static
     */
    public static $nav_framework_path;//='C:\\wamp\\www\\betterlife\\';
    /**
     * 上传图片的网络路径
     *
     * @var mixed
     */
    public static $upload_url;//='http://localhost/betterlife/upload/';
    /**
     * 上传图片的路径
     *
     * @var mixed
     */
    public static $upload_path;//='C:\\wamp\\www\\betterlife\\upload\\';
    //</editor-fold>

    //<editor-fold desc='开发者使用设置'>
    /**
     * 网站应用的名称<br/>
     * 在网站运行程序中使用，不展示给网站用户；如标识日志文件的默认名称等等。
     * @var string
     * @static
     */
    public static $appName='Redis 小助手';
    /**
     * 应用名的缩写
     */
    public static $appName_alias='orm';
    /**
     * @string 页面字符集<br/>
     * 一般分为：<br/>
     * UTF-8<br/>
     * GBK<br/>
     * 最终可以由用户选择
     * @var string
     * @static
     */
    public static $encoding = 'UTF-8';
    /**
     * @var string 文字显示默认语言
     * @static
     * 最终可以由用户选择
     */
    public static $language = 'Zh_Cn';
    /**
     * 是否Session自动启动
     * @var bool
     * @static
     */
    public static $session_auto_start = true;
    //</editor-fold>
    /**
     * 日志的配置。
     * @var array 日志的配置。
     * @static
     */
    //<editor-fold defaultstate='collapsed' desc='日志的设置'>
    public static $log_config = array(
        /**
         * 默认日志记录的方式。<br/>
         * 一般来讲，日志都通过log记录，由本配置决定它在哪里打印出来。<br/>
         * 可通过邮件发送重要日志，可在数据库或者文件中记录日志。也可以通过Firebug显示日志。
         * EnumLogType::FILE : 3
         */
        'logType' => EnumLogType::FILE,
        /**
         * 允许记录的日志级别
         */
        'log_record_level' => array('EMERG','ALERT','CRIT','ERR','INFO'),
        /**
         * 日志文件路径<br/>
         * 可指定日志文件放置的路径<br/>
         * 如果为空不设置，则在网站应用根目录下新建一个log目录，放置在它下面
         */
        'logpath' => '',
        /**
         * 检测日志文件大小，超过配置大小则备份日志文件重新生成，单位为字节
         */
        'log_file_size' => 1024000000,
        /**
         * 日志记录的时间格式
         */
        'timeFormat' => '%Y-%m-%d %H:%M:%S',
        /**
         * 通过邮件发送日志的配置。
         */
        'config_mail_log' => array(
            'subject' => '重要的日志事件',
            'mailBackend' => '',
        ),
        'log_table' => 'bb_log_log',
    );
    //</editor-fold>
    //</editor-fold>

    /**
     * 无需配置自动注入网站的网络地址和物理地址。
     */
    //<editor-fold defaultstate='collapsed' desc='初始化设置'>
    public static $is_port = true;
    public static function init()
    {
        if ( empty(Gc::$nav_root_path) ) Gc::$nav_root_path = __DIR__ . DS;
        if ( empty(Gc::$nav_framework_path) ) Gc::$nav_framework_path = __DIR__ . DS;
        if ( empty(Gc::$upload_path) ) Gc::$upload_path = Gc::$nav_root_path . 'upload' . DS;
        if ( empty(Gc::$url_base) ) {
            $baseurl = '';
            if ( isset($_SERVER['HTTPS']) && ($_SERVER['HTTPS']=='on') ) {
                $baseurl = 'https://'.$_SERVER['SERVER_NAME'];
                if ( self::$is_port && ( $_SERVER['SERVER_PORT'] != 443 ) ) $baseurl .= ':'.$_SERVER['SERVER_PORT'];
            } else {
                if ( array_key_exists('SERVER_NAME', $_SERVER) ) $baseurl = 'http://'.$_SERVER['SERVER_NAME'];
                if ( array_key_exists('SERVER_PORT', $_SERVER) ) {
                    if (strpos($_SERVER['HTTP_HOST'], $_SERVER['SERVER_PORT']) !== false) {
                        if ( self::$is_port && $_SERVER['SERVER_PORT'] != 80 ) $baseurl .= ':' . $_SERVER['SERVER_PORT'];
                    }
                }
            }
            $baseDir = dirname($_SERVER['SCRIPT_NAME']);
            $baseurl .= ($baseDir == '\\' ? '' : $baseDir);
            if (strpos(strrev($baseurl), "/") !== 0) $baseurl .= '/';
            $file_sub_dir = str_replace(Gc::$nav_root_path, "", getcwd() . DS);
            $file_sub_dir = str_replace(DS, "/", $file_sub_dir);
            Gc::$url_base = str_replace(strtolower($file_sub_dir), "", strtolower($baseurl));
        }
        if ( empty(Gc::$upload_url) ) {
            Gc::$upload_url = Gc::$url_base;
            $same_part = explode(DS,Gc::$nav_root_path);
            if ( $same_part && (count($same_part) > 2) ) {
                $same_part = $same_part[count($same_part)-2];
                if ( strpos(strtolower(Gc::$upload_url), "/" . strtolower($same_part)."/") !== false ) {
                    Gc::$upload_url = substr(Gc::$upload_url, 0, (strrpos(Gc::$upload_url, $same_part . "/") + strlen($same_part) + 1)) . "upload/";
                } else {
                    $parse_url = parse_url(Gc::$upload_url);
                    if ( array_key_exists("scheme", $parse_url) ) {
                        if ( $parse_url ) Gc::$upload_url = $parse_url["scheme"] . "://" . $parse_url["host"];
                        if (  self::$is_port && !empty($parse_url["port"]) ) Gc::$upload_url .= ":" . $parse_url["port"];
                    }
                    Gc::$upload_url .= "/upload/";
                }
            }
        }
    }
    //</editor-fold>
}
//</editor-fold>

Gc::init();
