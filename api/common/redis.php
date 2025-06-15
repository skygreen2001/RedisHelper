<?php

require_once("../../init.php");

header('Access-Control-Allow-Origin: *');
header("Referrer-Policy: no-referrer");
header('Content-Type:application/json;charset=UTF-8');

/****************************************  初始化: 是否安装必备  ***********************************/
$isPhpRedis = @$_GET["isPhpRedis"];
if (!empty($isPhpRedis)) {
    if (class_exists("Redis")) {
        echo true;
    } else {
        echo false;
    }
    return;
}

/****************************************  菜单操作: 核心逻辑  ***********************************/
$step = @$_GET["step"];     // 菜单操作
if (!empty($step)) {
    $isRemote  = @$_GET["isConfigRemote"];
    $server_id = @$_GET["server_id"];
    $db        = @$_GET["db"];
    $key       = @$_GET["key"];
    $val       = @$_GET["result"];

    if (isset($isRemote) && $isRemote == true) {
        if (!empty($server_id)) {
            // Redis服务器数据信息查询
            $config_need = server_config($server_id);
            if ($config_need) {
                extract($config_need);
            } else {
                return;
            }
        }
    } else {
        $server   = @$_GET["server"];
        $port     = @$_GET["port"];
        $password = @$_GET["password"];
    }

    if (!empty($server)) {
        $serverCache = BBCache::singleton()->redisServer($server, $port, $password);
    }

    $result = '';
    switch ($step) {
        // 查询Redis服务器设置列表
        case 100:
            $configs = server_configs();
            $result  = $configs["data"];
            $result  = array_column($result, 'name', 'id');
            $data    = array();
            foreach ($result as $key => $value) {
                $text = $value . "(" . $key . ")";
                $m = array(
                    "id" => $key,
                    "text" => $text
                );
                array_push($data, $m);
            }
            echo json_encode($data);
            return ;
            break;
            // 添加DB
        case 101:
            $dbs = $serverCache->dbInfos();
            $dbs = array_keys($dbs);
            $dbIndex = 0;
            foreach ($dbs as $db) {
                $serverCache->select($dbIndex);
                if ($serverCache->size() <= 0) {
                    break;
                }
                $dbIndex += 1;
            }
            $serverCache->select($dbIndex);
            $serverCache->save("createTime", UtilDateTime::now());
            $dbs = $serverCache->dbInfos();
            $result = array_keys($dbs);
            break;
            // 删除DB
        case 102:
            $dbIndex = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            $serverCache->clear();
            $dbs = $serverCache->dbInfos();
            if ($dbs && is_array($dbs) && count($dbs) > 0) {
                $result = array_keys($dbs);
            }
            break;
            // 查询指定服务器所有的DB
        case 1:
            $dbs = $serverCache->dbInfos();
            if (is_array($dbs) && count($dbs) == 0) {
                $dbIndex = 0;
                $serverCache->select($dbIndex);
                $serverCache->save("createTime", UtilDateTime::now());
                $dbs = $serverCache->dbInfos();
            }
            if ($dbs && is_array($dbs) && count($dbs) > 0) {
                $result = array_keys($dbs);
            }
            break;
            // 查询|分页查询指定DB所有的keys
        case 2:
            $dbIndex  = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            $countAll = $serverCache->size();
            $data     = $serverCache->keys();
            sort($data, SORT_STRING);
            $pageSize = @$_GET["pageSize"];
            if (!$pageSize) {
                $pageSize = 1000;
            }
            if ($countAll > $pageSize) {
                $page   = @$_GET["page"];
                if (!$page) {
                    $page = 1;
                }
                $start  = ($page - 1) * $pageSize;
                if ($start < $countAll) {
                    if ($start + $pageSize > $countAll) {
                        $pageSize = $countAll - $start;
                    }
                    $data = array_slice($data, $start, $pageSize);
                } else {
                    $data = array();
                }
            }
            $result              = array();
            $result["data"]      = $data;
            $result["countKeys"] = $countAll;
            break;
            // 查询指定key的value
        case 3:
            $dbIndex = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            $result         = array();
            $result["type"] = $serverCache->getKeyType($key);
            $result["data"] = $serverCache->get($key);
            // print_r($result);
            break;
            // 修改指定key的value
        case 4:
            $valType = @$_GET["valType"];
            $dbIndex = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            $serverCache->delete($key);
            $serverCache->set($key, $val, $valType);
            $result         = array();
            $result["data"] = $serverCache->get($key);
            $result["type"] = $serverCache->getKeyType($key);
            break;
            // 模糊查询指定关键词的所有key
        case 5:
            $queryKey = @$_GET["queryKey"];
            $dbIndex  = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            $data   = $serverCache->keys("*" . $queryKey . "*");
            $countAll = count($data);
            sort($data, SORT_STRING);
            $pageSize = @$_GET["pageSize"];
            if (!$pageSize) {
                $pageSize = 1000;
            }
            if ($countAll > $pageSize) {
                $page   = @$_GET["page"];
                if (!$page) {
                    $page = 1;
                }
                $start  = ($page - 1) * $pageSize;
                if ($start < $countAll) {
                    if ($start + $pageSize > $countAll) {
                        $pageSize = $countAll - $start;
                    }
                    $data = array_slice($data, $start, $pageSize);
                } else {
                    $data = array();
                }
            }
            $result              = array();
            $result["data"]      = $data;
            $result["countKeys"] = $countAll;
            break;
            // 新增key和value
        case 6:
            $addNewType  = @$_GET["addNewType"];
            $addNewKey   = @$_GET["addNewKey"];
            $addNewValue = @$_GET["addNewValue"];
            $dbIndex     = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            if (!empty($addNewType)) {
                $addNewType = $serverCache->getKeyTypeByShow($addNewType);
            }
            $serverCache->save($addNewKey, $addNewValue, $addNewType);
            $result      = $serverCache->keys();
            sort($result, SORT_STRING);
            break;
            // 删除指定key
        case 7:
            $dbIndex = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            $serverCache->delete($key);
            $result  = true;
            break;
            // 导出
        case 8:
            $queryKey = @$_GET["queryKey"];
            $dbIndex = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            if (!empty($queryKey)) {
                $keys = $serverCache->keys("*" . $queryKey . "*");
            } else {
                $keys = $serverCache->keys();
            }
            if ($keys && count($keys) > 0) {
                // $values = $serverCache->gets($keys);
                // $result = array_combine($keys, $values);
                $data = array();
                $index = 0;
                sort($keys, SORT_STRING);
                foreach ($keys as $key) {
                    $data[$index][0] = $key;
                    $data[$index][1] = $serverCache->get($key);
                    $data[$index][2] = $serverCache->getKeyType($key);
                    $data[$index][2] = $serverCache->getKeyTypeShow($data[$index][2]);
                    if ($data[$index][1] && is_array($data[$index][1])) {
                        $data[$index][1] = json_encode($data[$index][1], JSON_OBJECT_AS_ARRAY | JSON_UNESCAPED_UNICODE);
                    }
                    $index++;
                }
                $redisFile = "redis" . date("YmdHis") . ".xls";
                $redisPath = Gc::$upload_path . "redis" . DS . "export" . DS . $redisFile;
                $arr_output_header = array("key", "value", "type");
                UtilFileSystem::createDir(dirname($redisPath));
                UtilExcel::arraytoExcel($arr_output_header, $data, $redisPath, false);
                $result = Gc::$upload_url . "redis/export/" . $redisFile;
                echo $result;
                return;
            }
            // $result  = true;
            break;
            // 导入
        case 9:
            $ufile      = @$_GET["ufile"];
            $uploadPath = GC::$upload_path . "redis" . DS . "import" . DS . $ufile;
            $arr_import_header = array("key" => "key", "value" => "value", "type" => "type");
            $data    = UtilExcel::exceltoArray($uploadPath, $arr_import_header);
            $dbIndex = (int) str_replace("db", "", $db);
            $serverCache->select($dbIndex);
            if ($data && is_array($data) && count($data) > 0) {
                foreach ($data as $row) {
                    $addNewKey   = "";
                    $addNewValue = "";
                    $addNewType  = "";
                    if (array_key_exists("key", $row)) {
                        $addNewKey = $row["key"];
                    }
                    if (array_key_exists("value", $row)) {
                        $addNewValue = $row["value"];
                    }
                    if (array_key_exists("type", $row)) {
                        $addNewType = $row["type"];
                    }
                    if (!empty($addNewKey) && !empty($addNewValue)) {
                        if (!empty($addNewType)) {
                            $addNewType = $serverCache->getKeyTypeByShow($addNewType);
                            if ($addNewType == Redis::REDIS_SET ||
                                 $addNewType == Redis::REDIS_LIST ||
                                 $addNewType == Redis::REDIS_ZSET ||
                                 $addNewType == Redis::REDIS_HASH
                            ) {
                                $addNewValue = json_decode($addNewValue, JSON_OBJECT_AS_ARRAY);
                                $addNewValue = implode("<(||)>", $addNewValue);
                            }
                        }
                        $serverCache->delete($addNewKey);
                        $serverCache->set($addNewKey, $addNewValue, $addNewType);
                    }
                }
            }
            $result  = $serverCache->keys();
            sort($result, SORT_STRING);
            break;
        default:
            break;
    }

    $echo = json_encode($result);
    // 是经过序列化编码的Java对象
    if (json_last_error() == JSON_ERROR_UTF8) {
        $data = $result["data"];
        $rd   = array();
        foreach ($result["data"] as $key => $value) {
            $key = utf8_encode($key);
            $value = utf8_encode($value);
            $rd[$key] = $value;
        }
        $result["data"] = $rd;
        echo json_encode($result, JSON_FORCE_OBJECT | JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE);
    } else {
        echo $echo;
    }
    return ;
}

/****************************************  设置服务器  ***********************************/
$action = @$_GET["ca"];  // 设置操作
if ($action) {
    $configs = server_configs();
    $result  = $configs["data"];
    switch ($action) {
        // 查询Redis服务器设置列表
        case 100:
            echo json_encode($result);
            return ;
            break;
            // 新增Redis服务器设置
        case 1:
            $id       = @$_GET["id"];
            $name     = @$_GET["name"];
            $server   = @$_GET["server"];
            $port     = @$_GET["port"];
            $password = @$_GET["password"];
            $record   = array(
                "id"       => $id,
                "name"     => $name,
                "server"   => $server,
                "port"     => $port,
                "password" => $password,
            );
            if ($result) {
                $result[] = $record;
                $configs["data"] = $result;
                save_server_configs($configs);
            }
            echo true;
            return ;
            break;
            // 修改Redis服务器设置
        case 2:
            $id       = @$_GET["id"];
            $oid      = @$_GET["oid"];
            $name     = @$_GET["name"];
            $server   = @$_GET["server"];
            $port     = @$_GET["port"];
            $password = @$_GET["password"];
            if (!empty($oid)) {
                $qid = $oid;
            } else {
                $qid = $id;
            }
            $id_key    = array_search($qid, array_column($result, 'id'));
            $updateOne = $result[$id_key];
            $updateOne["id"]       = $id;
            $updateOne["name"]     = $name;
            $updateOne["server"]   = $server;
            $updateOne["port"]     = $port;
            $updateOne["password"] = $password;
            $result[$id_key]       = $updateOne;

            if ($result) {
                $configs["data"] = $result;
                save_server_configs($configs);
            }
            echo true;
            return ;
            break;
            // 删除Redis服务器设置
        case 3:
            $id     = @$_GET["id"];
            $id_key = array_search($id, array_column($result, 'id'));
            array_splice($result, $id_key, 1);
            if ($result) {
                $configs["data"] = $result;
                save_server_configs($configs);
            }
            echo true;
            return ;
            break;
    }
}

/****************************************  上传文件  ***********************************/
$action = @$_GET["action"];  // 是否上传文件
if (!empty($action)) {
    $result = '';
    $files = $_FILES;
    if (!empty($files["upload_file"])) {
        $diffpart   = date("YmdHis");
        $tmptail    = explode('.', $files["upload_file"]["name"]);
        $tmptail    = end($tmptail);
        $uploadFile = "redis$diffpart.$tmptail";
        $uploadPath = GC::$upload_path . "redis" . DS . "import" . DS . $uploadFile;
        $result     = UtilFileSystem::uploadFile($files, $uploadPath);
    }
    echo json_encode($result);
    return;
}

/****************************************  自定义函数  ***********************************/
/**
 * 获取配置文件路径
 * 首先检查upload目录下是否有该配置文件，如果没有，就从api默认文件复制到upload路径下
 */
function config_path()
{
    $config_redis_file = Gc::$nav_root_path . "api" . DS ."web" . DS . "data" . DS . "redis.json";
    $config_dest       = Gc::$upload_path . "redis" . DS ."config" . DS . "redis.json";
    UtilFileSystem::createDir(dirname($config_dest));
    if (!file_exists($config_dest)) {
        @copy($config_redis_file, $config_dest);

        if (!file_exists($config_dest)) {
            return $config_redis_file;
        }
    }
    return $config_dest;
}

/**
 * 保存服务器配置
 */
function save_server_configs($configs)
{
    if ($configs) {
        $config_redis_file = config_path();
        $contents = json_encode($configs, JSON_OBJECT_AS_ARRAY | JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE);
        if (!empty($contents)) {
            file_put_contents($config_redis_file, $contents);
        }
    }
}

/**
 * 获取服务器配置
 */
function server_configs()
{
    $config_redis_file = config_path();
    $config_content    = file_get_contents($config_redis_file);
    $result            = json_decode($config_content, true);
    return $result;
}

/**
 * 根据指定id获取指定的服务器配置
 */
function server_config($server_id)
{
    $result      = array();
    $configs     = server_configs();
    $config_need = $configs["data"];
    $id_key      = array_search($server_id, array_column($config_need, 'id'));
    if ($id_key >= 0) {
        $result = $config_need[$id_key];
    }
    return $result;
}
