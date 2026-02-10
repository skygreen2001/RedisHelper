<?php

/*
 +---------------------------------<br/>
 * 使用Redis作为系统缓存。<br/>
 * 使用方法: 添加以下内容到Config_Memcache中<br/>
 *     所有的缓存服务器Memcache 主机IP地址和端口配置<br/>
 *     保存数据是否需要压缩。 <br/>
 +---------------------------------
 * @see phpredis: https://github.com/phpredis/phpredis
 * @see install: https://github.com/phpredis/phpredis/blob/develop/INSTALL.markdown
 * @see 关于Mac安装PHP相关扩展出现Zend/zend_config.h缺失的问题记录: https://blog.51cto.com/vsfor/1892319
 *      - 在Mac系统目录下: /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX10.7.sdk/usr/include/php
 * @see Mac 下编译 PHP 扩展遇到的一些问题: http://www.ishenping.com/ArtInfo/3703557.html
 *      - 在Mac系统目录下: “/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/include/php”
 * @see redis服务的配置文件修改 bind ip,默认是bind 127.0.0.1 只允许本地连接 0.0.0.0允许任意ip,也可根据需要自己修改。
 * @see redis.conf 配置文件中设置: bind 127.0.0.1 192.168.64.2
 * [以下为老的解决方案，已作废，仅供学习参考]
 *      X@see Reference Method:http://code.google.com/p/phpredis/wiki/referencemethods
 *      X@see php-redis:http://code.google.com/p/php-redis/
 *      X@see PHP-redis中文说明:http://hi.baidu.com/%B4%AB%CB%B5%D6%D0%B5%C4%C8%CC%D5%DF%C3%A8/blog/item/c9ff4ac1898afa4fb219a8c7.html
 * @category betterlife
 * @package core.cache
 * @author skygreen
 */
class Cache_Redis extends Cache_Base
{
    private $redis;
    public static $SPLIT_ELEMENT = "<(||)>";

    /**
     * 测试体验Redis Cache
     */
    public function TestRun()
    {
        // $this->testData();
        $this->redis->hSet('h', 'key1', 'hello');

        $dbInfos = $this->dbInfos();
        print_pre($dbInfos, true, "获取Redis DB信息:");
        $i = 0;
        foreach ($dbInfos as $key => $value) {
            $this->select($i);
            // $count = $this->countKeys();
            // echo "[$key]共计:" . $count . "个keys<br/>";

            $allKeys = $this->keys();
            print_pre($allKeys, true, "[$key]清单如下:");
            $i++;
        }
    }

    /**
     * 验证测试数据
     */
    private function testData()
    {
        $this->set('key', 'value');
        $this->update('key', 'hello,girl');
        $test = $this->get('key');
        echo $test;
        $member = new User();
        $member->setName("skygreen2001");
        $member->setPassword("administrator");
        $member->setDepartmentId(3211);
        $this->save("Member1", $member);
        $user = $this->get("Member1");
        echo $user;
        $member = new User();
        $member->setName("学必填");
        $member->setPassword("password");
        $member->setDepartmentId(3212);
        $this->save("Member2", $member);

        $users = $this->gets(array('Member1', 'Member2'));
        print_r($users);
    }

    /**
     * 实例化初始化Redis服务器
     * @param string $host
     * @param string $port
     * @param string $password
     */
    public function __construct($host = '', $port = '', $password = '')
    {
        if (empty($host)) {
            $host = Config_Redis::$host;
        }
        if (empty($port)) {
            $port = Config_Redis::$port;
        }
        $this->redis = new Redis();
        if (Config_Redis::$is_persistent) {
            $this->redis->pconnect($host, $port);
        } else {
            $this->redis->connect($host, $port);
        }
        // LogMe::log( $this->redis );
        ob_clean();
        // if ( empty($password) ) {
        //     $password = Config_Redis::$password;
        // }
        if (!empty($password)) {
            $this->redis->auth($password);
        }
        if (!empty(Config_Redis::$prefix_key)) {
            $this->redis->setOption(Redis::OPT_PREFIX, Config_Redis::$prefix_key);
        }
    }

    /**
     * 选择指定第几个数据库
     */
    public function select($index)
    {
        $this->redis->select($index);
    }

    /**
     * 所有键值清单
     */
    public function keys($pattern = '*')
    {
        $result = $this->redis->keys($pattern);
        return $result;
    }

    /**
     * 计数: 所选择的DB所有键计数
     */
    public function size()
    {
        $result = $this->redis->dbSize();
        return $result;
    }


    /**
     * 计数:
     */
    public function countKeys()
    {
        $result = $this->redis->dbSize();
        return $result;
    }

    /**
     * 所有的数据库db
     */
    public function dbInfos()
    {
        $info = $this->redis->info();
        // print_pre($info, true, "Redis系统信息:");
        $result = UtilArray::like($info, "^db\d+");
        return $result;
    }

    /**
     * 获取指定$key的类型
     * @return
     *     1: string: Redis::REDIS_STRING
     *     2: set: Redis::REDIS_SET
     *     3: list: Redis::REDIS_LIST
     *     4: zset: Redis::REDIS_ZSET
     *     5: hash: Redis::REDIS_HASH
     *     0: other: Redis::REDIS_NOT_FOUND
     */
    public function getKeyType($key)
    {
        $result = $this->redis->type($key);
        return $result;
    }

    /**
     * 根据类型显示转换成Redis的类型值
     */
    public function getKeyTypeByShow($typeShow)
    {
        $type = "";
        $typeShow = strtoupper($typeShow);
        switch ($typeShow) {
            case "STRINGS":
                $type = Redis::REDIS_STRING;
                break;
            case "SETS":
                $type = Redis::REDIS_SET;
                break;
            case "LISTS":
                $type = Redis::REDIS_LIST;
                break;
            case "ZSETS":
            case "SORTED SETS":
                $type = Redis::REDIS_ZSET;
                break;
            case "HASHES":
                $type = Redis::REDIS_HASH;
                break;
            case "OTHER":
                $type = Redis::REDIS_NOT_FOUND;
                break;
            default:
                $type = $typeShow;
                break;
        }
        return $type;
    }
    /**
     * 类型显示字符串
     */
    public function getKeyTypeShow($type)
    {
        $typeOfKey = "";
        switch ($type) {
            case Redis::REDIS_STRING:
                $typeOfKey = "Strings";
                break;
            case Redis::REDIS_SET:
                $typeOfKey = "Sets";
                break;
            case Redis::REDIS_LIST:
                $typeOfKey = "Lists";
                break;
            case Redis::REDIS_ZSET:
                $typeOfKey = "Sorted Sets"; // "Sorted Sets";
                break;
            case Redis::REDIS_HASH:
                $typeOfKey = "Hashes";
                break;
            case Redis::REDIS_NOT_FOUND:
                $typeOfKey = "Other";
                break;
        }
        return $typeOfKey;
    }

    /**
     * 查看键key是否存在。
     * @param string $key
     */
    public function contains($key)
    {
        if (isset($this->redis)) {
            return  $this->redis->exists($key);
        }
        return false;
    }

    /**
    * 在缓存里保存指定$key的数据<br/>
    * 仅当存储空间中不存在键相同的数据时才保存<br/>
    * @param string $key
    * @param string|array|object $value
    *        - 如果redis键值类型为string: 直接保存
    *        - 如果redis键值类型为set   : 字符串每个元素之间以 ｜ 分割
    *        - 如果redis键值类型为list  : 字符串每个元素之间以 ｜ 分割
    *        - 如果redis键值类型为zset  : 字符串每个元素之间以 ｜ 分割
    *        - 如果redis键值类型为hash  : 数组; array(hashKey, val); hashKey和Val之间以: 隔开; 字符串每个元素之间以 ｜ 分割
    * @param string $type 键类型
    *     1: string: Redis::REDIS_STRING
    *     2: set: Redis::REDIS_SET
    *     3: list: Redis::REDIS_LIST
    *     4: zset: Redis::REDIS_ZSET
    *     5: hash: Redis::REDIS_HASH
    *     0: other: Redis::REDIS_NOT_FOUND
    * @param int $expired 过期时间，默认是1天；最高设置不能超过2592000(30天)
    * @return bool
    */
    public function save($key, $value, $type = Redis::REDIS_STRING, $expired = 86400)
    {
        if (empty($key)) {
            return;
        }
        if (is_object($value)) {
            $value = serialize($value);
        }
        switch ($type) {
            case Redis::REDIS_STRING:
                $this->redis->setNx($key, $value);
                break;
            case Redis::REDIS_SET:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            foreach ($content as $ivalue) {
                                $this->redis->sAdd($key, $ivalue);
                            }
                        }
                    } else {
                        $this->redis->sAdd($key, $value);
                    }
                }
                break;
            case Redis::REDIS_LIST:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            foreach ($content as $ivalue) {
                                $this->redis->rPush($key, $ivalue);
                            }
                        }
                    } else {
                        $this->redis->rPush($key, $value);
                    }
                }
                break;
            case Redis::REDIS_ZSET:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            $i = 0;
                            foreach ($content as $ivalue) {
                                $this->redis->zAdd($key, $i, $ivalue);
                                $i++;
                            }
                        }
                    } else {
                        $i = $this->redis->zSize($key) + 1;
                        $this->redis->zAdd($key, $i, $value);
                    }
                }
                break;
            case Redis::REDIS_HASH:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            foreach ($content as $ivalue) {
                                if (!empty($ivalue) & contain($ivalue, ":")) {
                                    $icontent = explode(":", $ivalue);
                                    $hashKey  = trim($icontent[0]);
                                    $hashVal  = trim($icontent[1]);
                                    $this->redis->hSet($key, $hashKey, $hashVal);
                                }
                            }
                        }
                    } else {
                        $this->redis->hSetNx($key, $key, $value);
                    }
                } else {
                    $this->redis->hSetNx($key, $key, "");
                }
                break;
            default:
                $this->redis->setNx($key, $value);
                break;
        }
        // $now = time(NULL); // current timestamp
        $now = time(); // current timestamp
        $this->redis->expireAt($key, $now + $expired);
    }

    /**
     * 在缓存里保存指定$key的数据 <br/>
     * 与save和update不同，无论何时都保存 <br/>
     * @param string $key
     * @param string|array|object $value
     *        - 如果redis键值类型为string: 直接保存
     *        - 如果redis键值类型为set   : 字符串每个元素之间以 ｜ 分割
     *        - 如果redis键值类型为list  : 字符串每个元素之间以 ｜ 分割
     *        - 如果redis键值类型为zset  : 字符串每个元素之间以 ｜ 分割
     *        - 如果redis键值类型为hash  : 数组; array(hashKey, val); hashKey和Val之间以: 隔开; 字符串每个元素之间以 ｜ 分割
     * @param int $expired 过期时间，默认是1天；最高设置不能超过2592000(30天)
     * @return bool
     */
    public function set($key, $value, $type = Redis::REDIS_NOT_FOUND, $expired = 86400)
    {
        if (empty($key)) {
            return;
        }
        if (is_object($value)) {
            $value = serialize($value);
        }
        if ($type == Redis::REDIS_NOT_FOUND) {
            if ($type == Redis::REDIS_NOT_FOUND) {
                $type = Redis::REDIS_STRING;
            }
        }
        // $type = Redis::REDIS_SET;
        switch ($type) {
            case Redis::REDIS_STRING:
                $this->redis->setEx($key, $expired, $value);
                break;
            case Redis::REDIS_SET:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            foreach ($content as $ivalue) {
                                $this->redis->sAdd($key, $ivalue);
                            }
                        }
                    } else {
                        $this->redis->sAdd($key, $value);
                    }
                } else {
                    $this->redis->sAdd($key, "");
                }
                break;
            case Redis::REDIS_LIST:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            foreach ($content as $ivalue) {
                                $this->redis->rPush($key, $ivalue);
                            }
                        }
                    } else {
                        $this->redis->rPush($key, $value);
                    }
                } else {
                    $this->redis->rPush($key, "");
                }
                break;
            case Redis::REDIS_ZSET:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            $i = 0;
                            foreach ($content as $ivalue) {
                                $this->redis->zAdd($key, $i, $ivalue);
                                $i++;
                            }
                        }
                    } else {
                        $i = $this->redis->zSize($key) + 1;
                        $this->redis->zAdd($key, $i, $value);
                    }
                } else {
                    $this->redis->zAdd($key, 0, "");
                }
                break;
            case Redis::REDIS_HASH:
                if (!empty($value)) {
                    if (contain($value, self::$SPLIT_ELEMENT)) {
                        $content = explode(self::$SPLIT_ELEMENT, $value);
                        if ($content && is_array($content) && count($content) > 0) {
                            foreach ($content as $ivalue) {
                                if (!empty($ivalue) & contain($ivalue, ":")) {
                                    $icontent = explode(":", $ivalue);
                                    $hashKey  = trim($icontent[0]);
                                    $hashVal  = trim($icontent[1]);
                                    $this->redis->hSet($key, $hashKey, $hashVal);
                                }
                            }
                        }
                    } else {
                        $this->redis->hSet($key, $key, $value);
                    }
                } else {
                    $this->redis->hSet($key, $key, "");
                }
                break;
            default:
                $this->redis->setEx($key, $expired, $value);
                break;
        }
    }

    /**
     * 在缓存里更新指定key的数据<br/>
     * 仅当存储空间中存在键相同的数据时才保存<br/>
     * @param string $key
     * @param string|array|object $value
     * @return bool
     */
    public function update($key, $value, $type = Redis::REDIS_STRING, $expired = 86400)
    {
        $this->set($key, $value, $type, $expired);
    }

    /**
     * 在缓存里删除所有指定$key的数据
     * @param string|array $key
     * @return bool
     */
    public function delete($key)
    {
        $this->redis->delete($key);
    }

    /**
     * 获取指定key的值
     * @param string $key
     * @return string|array|object
     */
    public function get($key)
    {
        if (empty($key)) {
            return;
        }
        $type = $this->getKeyType($key);
        switch ($type) {
            case Redis::REDIS_STRING :
                $data = $this->redis->get($key);
                break;
            case Redis::REDIS_SET :
                $data = $this->redis->sMembers($key);
                break;
            case Redis::REDIS_LIST :
                $data = $this->redis->lRange($key, 0, -1);
                break;
            case Redis::REDIS_ZSET :
                $data = $this->redis->zRange($key, 0, -1);
                break;
            case Redis::REDIS_HASH :
                $data   = $this->redis->hGetAll($key);
                $result = json_encode($data);
                // 是经过序列化编码的Java对象
                if (json_last_error() == JSON_ERROR_NONE) {
                    $rd   = array();
                    foreach ($data as $skey => $svalue) {
                        $rd[] = $skey . ": " . $svalue;
                    }
                    $data = $rd;
                }
                break;
            default:
                $data = $this->redis->get($key);
                break;
        }

        if (is_array($data)||is_object($data)) {
            return $data;
        } else {
            if ( @unserialize($data) ) {
                $data = unserialize($data);
            }
        }
        return $data;
    }

    /**
     * 获取指定keys的值们。<br/>
     * 只取字符串
     * 允许一次查询多个键值，减少通讯次数。
     * @param array $key
     * @return array
     */
    public function gets($keyArr)
    {
        // $data = $this->redis->getMultiple($keyArr);
        $data = $this->redis->mGet($keyArr);
        if ($data) {
            $result = array();
            foreach ($data as $element) {
                if (@unserialize($element)) {
                    $element = unserialize($element);
                }
                $result[] = $element;
            }
        }
        return $result;
    }

    /**
     * 清除当前选中DB所有的键值
     *
     */
    public function clear()
    {
        // $allKeys = $this->redis->keys('*');
        // $this->delete( $allKeys );
        $this->redis->flushDb();
    }

    /**
     * 清除所有DB所有的键值
     *
     */
    public function clearAll()
    {
        $this->redis->flushAll();
    }

    /**
     * 关闭Redis服务器
     */
    public function close()
    {
        $this->redis->close();
    }
}
