Vue.config.debug = true;
Vue.config.devtools = true;

var app = new Vue({
    el: '#app',
    data(){
        return{
            isConfigLocal: true, // 服务器配置是否保存在本地浏览器 Local Storage 里
            apiUrl       : '../api/common/redis.php',
            server: '', // 选中的服务器
            db    : '', // 选中的db
            key   : '', // 选中的key
            sel_server: '', // 选中服务器显示[标题栏]
            sel_db    : '', // 选中DB显示[标题栏]
            sel_key   : '', // 选中key显示[标题栏]
            serves  : [], // 所有服务器列表
            dbs     : [], // 选中的服务器所有的db列表
            keys    : [], // redis里所有的主键
            page    : 1,    // 当前页数
            pageSize: 1000, // 每页键数量
            result         : '', // 选中key后的value
            resultTypeShow : "",    // 键值类型显示
            valType        : 1,     // 指定Redis键值的类型，默认是1:字符串； 2: set
            isLoadingShow  : false, // 是否显示DB菜单
            addserverModel : false, // 是否显示新增服务器窗口
            addNewKeyModel : false, // 是否显示新增键值设置窗口
            deleteDbModel  : false, // 是否显示删除Db窗口
            configModel    : false, // 是否显示设置窗口
            uploadDbModel  : false, // 是否上传DB键值Excel数据
            isConfirmAction: false, // 是否显示确认操作窗口
            isConfActionDb : false, // 是否显示确认删除DB操作窗口
            addNewType     : '',    // 新增的键类型
            addNewKey      : '',    // 新增的键
            addNewValue    : '',    // 新增的值
            valueSample    : '',    // 新增的值示例
            countKeys      : 0,     // 共计主键
            latestKeys     : 0,     // 当前键最新总数
            deleteOne      : '',    // 需要删除的DB
            customIP       : '',    // 自定义IP
            queryKey       : '',    // 查询键
            upload_file    : [],    // 上传文件
            unModify       : false, // 一般是经过序列化编码的Java对象
            css            : {
                r_lf_ht: '600px',
                r_rt_ht: '600px'
            },
            configs        : {
                data   : [], // 所有设备(服务器)配置数据
                columns: [
                    {
                        title: '标识',
                        slot : 'id',
                        width: 80,
                        sortable: true
                    },
                    {
                        title: '名称',
                        slot : 'name'
                    },
                    {
                        title: '服务器地址',
                        slot : 'server'
                    },
                    {
                        title: '端口',
                        width: 80,
                        slot : 'port'
                    },
                    {
                        title: '密码',
                        slot : 'password'
                    },
                    {
                        title: '操作',
                        slot : 'action',
                        width: 180,
                        align: 'center'
                    }
                ],
                addConfigForm: {
                    configsForm: {
                        editId    : '',  // 第一列输入框，当然聚焦的输入框的输入内容，与 data 分离避免重构的闪烁
                        editName  : '',  // 第二列输入框
                        editServer: '',  // 第三列输入框
                        editPort  : '',  // 第四列输入框
                        editPasswd: '',  // 第五列输入框
                    },
                    ruleValidate: {
                        editId: [
                            { required: true, message: '请输入标识', trigger: 'blur' }
                        ],
                        editName: [
                            { required: true, message: '请输入名称', trigger: 'blur' }
                        ],
                        editServer: [
                            { required: true, message: '请输入服务器', trigger: 'blur' },
                        ]
                    }
                },
                editConfig: {
                    editIndex : -1,  // 当前聚焦的输入框的行数
                    editId    : '',  // 第一列输入框，当然聚焦的输入框的输入内容，与 data 分离避免重构的闪烁
                    editName  : '',  // 第二列输入框
                    editServer: '',  // 第三列输入框
                    editPort  : '',  // 第四列输入框
                    editPasswd: '',  // 第五列输入框
                },
                deleteOne : -1,  // 被删除的行数
            },
            actionInfo : "",   // 确认操作窗口内容
        }
    },
    created: function() {
        this.getServers();
    },
    methods: {
        menuSelect: function(name) {
            // console.log(name);
            switch (name) {
                case "1":
                    this.configModel = true;
                    this.getConfigs();
                    break;
                case "8":
                    this.addNewKeyModel = true;
                    break;
                default:
                    break;
            }
            if ( name != "1" && name!= "8" ) {
                this.server     = name
                let index       = this.serves.findIndex(e => e.id == name);
                this.sel_server = this.serves[index].text;
                this.sel_db     = "";
                this.sel_key    = "";
                this.countKeys  = 0;
                this.getDbs();
            }
        },
        loading () {
            this.$Spin.show({
                render: (h) => {
                    return h('div', [
                        h('Icon', {
                            'class': 'spin-icon-load',
                            props: {
                                type: 'ios-loading',
                                size: 18
                            }
                        }),
                        h('div', 'Loading')
                    ])
                }
            });
        },
        getServers: function() {
            var ctrl = this;
            let params = {
                step: 100
            };
            if ( this.isConfigLocal ) {
                this.serves = [];
                let configs = localStorage.configs;
                if ( configs ) {
                    configs = JSON.parse(configs);
                    configs.forEach(function(config) {
                        let text = config.name + "(" + config.id + ")";
                        let server = {
                            id: config.id,
                            text: text
                        };
                        ctrl.serves.push(server);
                    });
                }
                if ( this.serves.length == 0 ) {
                    this.configModel = true;
                }
            } else {
                this.loading();
                axios.get(this.apiUrl, {
                          params: params
                       })
                     .then(function (response) {
                         // console.log(response);
                         if ( Array.isArray(response.data) ) {
                             ctrl.serves = response.data;
                             localStorage.serves = JSON.stringify(ctrl.serves);
                             ctrl.$Spin.hide();
                         } else {
                             ctrl.$Spin.hide();
                             ctrl.$Modal.error({
                                title: '服务端错误',
                                content: '请确认服务端是否配置正确！'
                             });
                         }
                     })
                     .catch(function (error) {
                         console.log(error);
                     });
            }
        },
        getConfigs: function() {
            var ctrl = this;
            let params = {
                ca: 100
            };
            this.configs.editIndex = -1;

            if ( this.isConfigLocal ) {
                if ( localStorage.configs ) {
                    this.configs.data = JSON.parse(localStorage.configs);
                } else {
                    this.configs.data = [];
                }
            } else {
                this.loading();
                axios.get(this.apiUrl, {
                          params: params
                       })
                     .then(function (response) {
                         if ( Array.isArray(response.data) ) {
                             ctrl.configs.data = response.data;
                             // console.log(ctrl.configs.data);
                             if ( !localStorage.configs ) {
                                 localStorage.configs = JSON.stringify(ctrl.configs.data);
                             }
                         } else {
                             if ( response.data ) {
                                 ctrl.$Modal.error({
                                    title: '服务端错误',
                                    content: '请确认服务端是否配置正确！'
                                 });
                             }
                         }
                         ctrl.$Spin.hide();
                     })
                     .catch(function (error) {
                         console.log(error);
                     });
            }
        },
        onEditConfig:function(row, index) {
            this.configs.editConfig.editId     = row.id;
            this.configs.editConfig.editName   = row.name;
            this.configs.editConfig.editServer = row.server;
            this.configs.editConfig.editPort   = row.port;
            this.configs.editConfig.editPasswd = row.password;
            this.configs.editConfig.editIndex  = index;
        },
        onAddConfig() {
            this.addserverModel = true;
            this.configs.addConfigForm.configsForm.editId     = "";
            this.configs.addConfigForm.configsForm.editName   = "";
            this.configs.addConfigForm.configsForm.editServer = "";
            this.configs.addConfigForm.configsForm.editPort   = "";
            this.configs.addConfigForm.configsForm.editPasswd = "";
        },
        onCancelAddConfig: function() {
            this.addserverModel = false;
        },
        onSaveConfig: function() {
            let params = {
                ca      : 1,
                id      : this.configs.addConfigForm.configsForm.editId.trim(),
                name    : this.configs.addConfigForm.configsForm.editName.trim(),
                server  : this.configs.addConfigForm.configsForm.editServer.trim(),
                port    : this.configs.addConfigForm.configsForm.editPort.trim(),
                password: this.configs.addConfigForm.configsForm.editPasswd.trim()
            };
            if ( !params.id || !params.name || !params.server ) {
                this.$Modal.error({
                    title: '新增错误',
                    content: '请确认设备(服务器)配置是否填写正确！'
                });
                return ;
            }
            if ( !params.port ) params.port = "6379";

            if ( this.isConfigLocal ) {
                let data = []
                if ( localStorage.configs ) {
                    data = JSON.parse(localStorage.configs);
                }
                delete params.ca;
                data.push(params);

                localStorage.configs = JSON.stringify(data);
                this.getServers();
            } else {
                axios.get(this.apiUrl, {
                          params: params
                       })
                     .then(function (response) {
                         // console.log(response);
                     })
                     .catch(function (error) {
                         console.log(error);
                     });
            }

            let record = params;
            delete record.action;
            this.configs.data.push(record);

            this.configs.addConfigForm.configsForm.editId     = "";
            this.configs.addConfigForm.configsForm.editName   = "";
            this.configs.addConfigForm.configsForm.editServer = "";
            this.configs.addConfigForm.configsForm.editPort   = "";
            this.configs.addConfigForm.configsForm.editPasswd = "";
            this.addserverModel = false;
        },
        onSaveEditConfig: function(index) {
            if ( index < 0 ) {
                return ;
            }
            let params = {
                ca      : 2,
                id      : this.configs.editConfig.editId.trim(),
                oid     : this.configs.data[index].id,
                name    : this.configs.editConfig.editName.trim(),
                server  : this.configs.editConfig.editServer.trim(),
                port    : this.configs.editConfig.editPort.trim(),
                password: this.configs.editConfig.editPasswd.trim()
            };
            if ( !params.id || !params.name || !params.server ) {
                this.$Modal.error({
                    title: '编辑错误',
                    content: '请确认设备(服务器)配置是否填写正确！'
                });
                return ;
            }
            if ( !params.port.trim() ) {
                params.port = "6379";
                this.configs.editConfig.editPort = 6379;
            }

            if ( this.isConfigLocal ) {
                let data = []
                if ( localStorage.configs ) {
                    data = JSON.parse(localStorage.configs);
                }
                delete params.ca;
                let index = data.findIndex(e => e.id == params.oid);
                delete params.oid;
                data[index] = params;

                localStorage.configs = JSON.stringify(data);
                this.getServers();
            } else {
                axios.get(this.apiUrl, {
                          params: params
                       })
                     .then(function (response) {
                         // console.log(response);
                     })
                     .catch(function (error) {
                         console.log(error);
                     });
            }

            this.configs.data[index].id       = this.configs.editConfig.editId;
            this.configs.data[index].name     = this.configs.editConfig.editName;
            this.configs.data[index].server   = this.configs.editConfig.editServer;
            this.configs.data[index].port     = this.configs.editConfig.editPort;
            this.configs.data[index].password = this.configs.editConfig.editPasswd;
        
            this.configs.editConfig.editIndex  = -1;
            this.configs.editConfig.editId     = "";
            this.configs.editConfig.editName   = "";
            this.configs.editConfig.editServer = "";
            this.configs.editConfig.editPort   = "";
            this.configs.editConfig.editPasswd = "";
        },
        onDeleteConfig:function(row, index) {
            this.actionInfo        = "删除后将不能恢复，确认要删除" + row.name + "[" + row.id + "]?";
            this.isConfirmAction   = true;
            this.configs.deleteOne = index;
        },
        onConfirmDeleteConfig:function() {
            let deleteOne = this.configs.data[this.configs.deleteOne];
            let id        = deleteOne.id;
            if ( this.isConfigLocal ) {
                let data = []
                if ( localStorage.configs ) {
                    data = JSON.parse(localStorage.configs);
                }
                let index = data.findIndex(e => e.id == id);
                if ( data && data.length ) {
                    data.splice(index, 1);
                }
                localStorage.configs = JSON.stringify(data);
                this.getServers();
            } else {
                axios.get(this.apiUrl, {
                          params: {
                              ca: 3,
                              id: id
                          }
                       })
                     .then(function (response) {
                         // console.log(response);
                     })
                     .catch(function (error) {
                         console.log(error);
                     });
            }
            this.configs.data.splice(this.configs.deleteOne, 1);
            this.configs.deleteOne = -1;
        },
        initParamServer: function(step) {
            let params = {
                step: step
            };
            let server_id = this.server;
            if ( this.isConfigLocal ) {
                if ( localStorage.configs ) {
                    let data = JSON.parse(localStorage.configs);
                    if ( data && data.length >0 ) {
                        let index = data.findIndex(e => e.id == server_id);
                        if ( index >= 0 ) {
                            let serverConfig = data[index];
                            params.server   = serverConfig.server;
                            params.port     = serverConfig.port;
                            params.password = serverConfig.password;
                        } else {
                            this.configModel = true;
                        }
                    } else {
                        this.configModel = true;
                    }
                } else {
                    this.configModel = true;
                }
            } else {
                params.isConfigRemote = true;
                params.server_id = server_id;
            }
            return params;
        },
        getDbs: function() {
            var ctrl   = this;
            let params = this.initParamServer(1);
            this.dbs    = [];
            this.keys   = [];
            this.result = '';
            this.countKeys  = 0;
            this.latestKeys = 0;
            this.resultTypeShow = "";
            this.isLoadingShow  = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     // console.log(response);
                     if ( Array.isArray(response.data) ) {
                         ctrl.dbs = response.data;
                         // console.log(ctrl.dbs);
                     } else {
                         ctrl.$Modal.error({
                            title: '服务端错误',
                            content: '请确认服务端是否配置正确！'
                         });
                     }
                     ctrl.isLoadingShow = false;
                 })
                 .catch(function (error) {
                     console.log(error);
                     ctrl.isLoadingShow = false;
                 });
        },
        addDb: function() {
            var ctrl = this;
            let params = this.initParamServer(101);
            this.dbs = [];
            this.initKeys();
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     if ( Array.isArray(response.data) ) {
                         ctrl.dbs = response.data;
                         ctrl.getDbs();
                         ctrl.sel_db = "";
                         // console.log(ctrl.dbs);
                     } else {
                         ctrl.$Modal.error({
                            title: '服务端错误',
                            content: '请确认服务端是否配置正确！'
                         });
                     }
                     ctrl.isLoadingShow = false;
                     // console.log(response);
                 })
                 .catch(function (error) {
                     console.log(error);
                     ctrl.isLoadingShow = false;
                 });
        },
        toDeleteDb: function() {
            this.deleteDbModel = true;
        },
        onConfirmDeleteDb: function() {
            this.actionInfo     = "删除后将不能恢复，确认要删除[" + this.deleteOne + "]?";
            this.isConfActionDb = true;
        },
        deleteDb: function() {
            var ctrl   = this;
            let params = this.initParamServer(102);
            params.db  = this.deleteOne;
            this.dbs   = [];
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     if ( Array.isArray(response.data) ) {
                         ctrl.dbs = response.data;
                         ctrl.getDbs();
                         ctrl.sel_db = "";
                         // console.log(ctrl.dbs);
                     } else {
                         ctrl.$Modal.error({
                            title: '服务端错误',
                            content: '请确认服务端是否配置正确！'
                         });
                     }
                     ctrl.isLoadingShow = false;
                     console.log(response);
                 })
                 .catch(function (error) {
                     console.log(error);
                     ctrl.isLoadingShow = false;
                 });
        },
        initKeys: function() {
            this.countKeys  = 0;
            this.latestKeys = 0;
            this.page       = 1;
            this.keys       = [];
            this.sel_key = "";
            this.result  = '';
            this.resultTypeShow = "";
        },
        dbDo: function(name) {
            switch (name) {
                case "10":
                    this.addDb();
                    return ;
                    break;
                case "11":
                    this.toDeleteDb();
                    return ;
                    break;
            }
            this.queryKey = "";
            this.db       = name;
            this.sel_db   = name;
            this.initKeys();
            this.getKeys();
        },
        getKeys: function(name) {
            var ctrl    = this;
            let params  = this.initParamServer(2);
            params.db   = this.db;
            params.page     = this.page;
            params.pageSize = this.pageSize;
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                    if ( response.data ) {
                        ctrl.countKeys = response.data.countKeys;
                        if ( ctrl.countKeys > ctrl.pageSize && response.data.data.length > 0 ) {
                            ctrl.latestKeys += response.data.data.length;
                            ctrl.keys = ctrl.keys.concat(response.data.data);
                        } else {
                            ctrl.latestKeys = ctrl.countKeys;
                            ctrl.keys       = response.data.data;
                        }
                    }
                    ctrl.isLoadingShow = false;
                     // console.log(ctrl.keys);
                 })
                 .catch(function (error) {
                    console.log(error);
                    ctrl.isLoadingShow = false;
                 });
        },
        doQueryKey: function() {
            var ctrl = this;
            let params  = this.initParamServer(5);
            params.db   = this.db;
            params.queryKey = this.queryKey;
            params.page     = this.page;
            params.pageSize = this.pageSize;
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     if ( response.data ) {
                         ctrl.countKeys = response.data.countKeys;
                         if ( ctrl.countKeys > ctrl.pageSize && response.data.data.length > 0 ) {
                             ctrl.latestKeys += response.data.data.length;
                             ctrl.keys = ctrl.keys.concat(response.data.data);
                         } else {
                             ctrl.latestKeys = ctrl.countKeys;
                             ctrl.keys       = response.data.data;
                         }
                     }
                     ctrl.isLoadingShow = false;
                     // console.log(ctrl.keys);
                 })
                 .catch(function (error) {
                     console.log(error);
                     ctrl.isLoadingShow = false;
                 });
        },
        getKeyDetail: function(key) {
            this.key = key;
            var ctrl = this;
            this.valType = 1;
            this.sel_key = key;
            let params = this.initParamServer(3);
            params.db  = this.db;
            params.key = this.key;
            this.isLoadingShow = true;
            this.unModify      = false;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     var data = response.data;
                     var result = "";
                     if ( data && data.data ) {
                         if ( data.type ) {
                             ctrl.valType = data.type;
                             ctrl.resultTypeShow = ctrl.valTypeShow(ctrl.valType);
                         }
                         if ( data.data ) {
                             if ( Array.isArray(data.data) ) {
                                 result = data.data.join("\r\n");
                             } else if ( typeof(data.data) == 'object' ) {
                                 result = JSON.stringify(data.data, null, 2);
                                 ctrl.unModify = true;
                             } else {
                                 result = data.data.trim();
                                 if ( ctrl.isJson(result) ) {
                                     result = JSON.stringify(JSON.parse(result), null, 2);
                                     result = result.replace(/ /g, "  ");
                                 }
                             }
                         }
                     }
                     ctrl.result = result;
                     ctrl.isLoadingShow = false;
                 })
                 .catch(function (error) {
                     ctrl.isLoadingShow = false;
                     console.log(error);
                 });
        },
        updateKey: function() {
            var ctrl = this;
            let result = this.result;
            if ( result ) {
                if ( this.valType == 1 ) {
                    // result = result.replace(/\s+/g, "");
                } else {
                    result = result.trim().replace(/\n/g, "<(||)>");
                }
            }
            let params     = this.initParamServer(4);
            params.db      = this.db;
            params.key     = this.key;
            params.valType = this.valType;
            params.result  = result;
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     var data = response.data;
                     var result = "";
                     if ( data && data.data ) {
                         if ( Array.isArray(data.data) ) {
                             result = data.data.join("\r\n");
                         } else {
                             result = data.data.trim();
                             if ( ctrl.isJson(result) ) {
                                 result = JSON.stringify(JSON.parse(result), null, 2);
                                 result = result.replace(/ /g, "  ");
                             }
                         }
                         ctrl.valType = data.type;
                         ctrl.resultTypeShow = ctrl.valTypeShow(ctrl.valType);
                     }
                     ctrl.result = result;
                     ctrl.isLoadingShow = false;
                     ctrl.$Modal.success({
                        title: '信息',
                        content: '修改[' + ctrl.key + ']的值成功！'
                     });
                 })
                 .catch(function (error) {
                     ctrl.isLoadingShow = false;
                     console.log(error);
                 });
        },
        changeKeyType: function() {
            if ( this.addNewType == "Sets" || this.addNewType == "Lists" || this.addNewType == "Sorted Sets" ) {
                this.valueSample = "列表集合每个元素之间以换行作为分割";
            } else if ( this.addNewType == "Hashes" ) {
                this.valueSample = "<p style='color:#515a6e;'>HASH的键值格式如下:<br/></p><p style='width:20%;text-align:left;margin-left:10%;'>hashKey: val<br/>hashKey1: val1<br/>hashKey2: val2<br/></sp>";
            } else {
                this.valueSample = "";
            }
        },
        onCancelAddKey: function() {
            this.addNewKey     = "";
            this.addNewValue   = "";
        },
        onAddNewKey: function() {
            var ctrl        = this;
            let addNewValue = this.addNewValue;
            if ( addNewValue ) {
                if ( this.addNewType == "Strings" ) {
                    // addNewValue = addNewValue.replace(/\s+/g, "");
                } else {
                    addNewValue = addNewValue.trim().replace(/\n/g, "<(||)>");
                    // console.log(result);
                }
            }
            let params  = this.initParamServer(6);
            params.db          = this.db;
            params.addNewType  = this.addNewType;
            params.addNewKey   = this.addNewKey;
            params.addNewValue = addNewValue;
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                    //  var data  = response.data;
                     ctrl.keys = response.data;
                     ctrl.isLoadingShow = false;
                     ctrl.$Modal.success({
                        title: '信息',
                        content: '新增[' + ctrl.addNewKey + ']的值成功！'
                     });
                     ctrl.addNewKey     = "";
                     ctrl.addNewValue   = "";
                 })
                 .catch(function (error) {
                     ctrl.isLoadingShow = false;
                     console.log(error);
                 });
        },
        onDeleteKey() {
          var ctrl = this;
          let params  = this.initParamServer(7);
          params.db   = this.db;
          params.key  = this.sel_key;
          this.isLoadingShow = true;
          let del_key = this.sel_key;
          axios.get(this.apiUrl, {
                    params: params
                 })
               .then(function (response) {
                   var data = response.data;
                   ctrl.getKeys(ctrl.db);
                   ctrl.queryKey = "";
                   ctrl.isLoadingShow = false;
                   ctrl.$Modal.success({
                      title: '信息',
                      content: '删除[' + del_key + ']的值成功！'
                   });
               })
               .catch(function (error) {
                   ctrl.isLoadingShow = false;
                   console.log(error);
               });
        },
        onUpload: function() {
            this.uploadDbModel = true;
        },
        beforeUpload: function() {
            this.$refs.upload.clearFiles();
        },
        uploadFileFinish: function(response, file, fileList) {
            this.upload_file = response.file_name;
            console.log(response);
            console.log(file);
            console.log(fileList);
        },
        onUploadDbData: function(data) {
            var ctrl = this;
            let params   = this.initParamServer(9);
            params.db    = this.db;
            params.ufile = this.upload_file;
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     if ( response && response.data ) {
                        ctrl.keys = response.data;
                     }
                     ctrl.$refs.upload.clearFiles();
                     ctrl.isLoadingShow = false;
                 })
                 .catch(function (error) {
                     ctrl.isLoadingShow = false;
                     console.log(error);
                 });
        },
        onDownload: function() {
            var ctrl = this;
            let params      = this.initParamServer(8);
            params.db       = this.db;
            params.queryKey = this.queryKey;
            this.isLoadingShow = true;
            axios.get(this.apiUrl, {
                      params: params
                   })
                 .then(function (response) {
                     ctrl.isLoadingShow = false;
                     var data = response.data;
                     window.open(data);
                 })
                 .catch(function (error) {
                     ctrl.isLoadingShow = false;
                     console.log(error);
                 });
        },
        /**
         * 判断是否json
         */
        isJson: function(content) {
            try {
                if ( typeof JSON.parse(content) == 'object' )
                    return true;
                return false;
            } catch (e) {
                console.log(e);
                return false;
            }
        },
        valTypeShow: function(valType) {
            let result = "Strings";
            switch (valType) {
                case 1:
                    result = "Strings";
                    break;
                case 2:
                    result = "Sets";
                    break;
                case 3:
                    result = "Lists";
                    break;
                case 4:
                    result = "Sorted Sets";
                    break;
                case 5:
                    result = "Hashes";
                    break;
                default:
                    result = "Other";
                    break;
            }
            return result;
        },
        goGit: function() {
            window.open("https://github.com/skygreen2001/RedisManager", "_blank");
        },
        isPhpRedis: function() {
            var ctrl = this;
            let params        = {}
            params.isPhpRedis = true;
            this.isLoadingShow = true;

            axios.get(this.apiUrl, {
                params: params
            })
            .then(function (response) {
                ctrl.isLoadingShow = false;
                var data = response.data;
                if ( !data ) {
                    ctrl.$Modal.success({
                        title: '信息',
                        content: '需要安装php redis，详情查看core/cache/redis/Cache_Redis.php类注解！'
                    });
                }
            })
            .catch(function (error) {
                ctrl.isLoadingShow = false;
                console.log(error);
            });
        },
        initLayout: function() {
            let offsetH  = 190;
            let c_height = (window.innerHeight - offsetH) +'px';
            // 内容区域左侧、右侧高度
            this.css.r_lf_ht = c_height;
            this.css.r_rt_ht = c_height + 5;
            // 内容区域左侧内容高度
            let l_height = (window.innerHeight - 283) +'px !important';
            $(".left_area .ivu-list-container").css("cssText", "min-height:" + l_height);
            $(".fixed-header").css("width", $(".ivu-list-bordered").width() + 2);
            // $(".left_area .ivu-list-container").css("cssText", "height:auto");
            // 内容区域右侧内容高度
            let r_height = (window.innerHeight - 236) +'px !important';
            $(".right_area textarea").css("cssText", "min-height:" + r_height + ";max-height:" + r_height);
            if (window.innerHeight < 800) {
                $(".right_area").css("cssText", "position:relative;width:59%;right:5px;");
            }
        }
    },
    watch : {
        reportInfo: function(newValue) {
            this.reportInfo = newValue;
        }
    },
    mounted() {
        this.initLayout();

        this.isPhpRedis();

        window.onresize = () => {
            return (() => {
                this.initLayout();
            })()
        }

        let ctrl = this;
        //滚动翻页: https://stackoverflow.com/questions/14035180/jquery-load-more-data-on-scroll
        $(".left_area").scroll(function() {
            var scrollTop = $(".left_area").scrollTop();
            var destTop   = $(".left_area").prop('scrollHeight');
            var offsetH   = $(".left_area").height();
            if ( (scrollTop == destTop - offsetH) && (ctrl.latestKeys < ctrl.countKeys) ) {
                getMore();
            }
        });

        function getMore() {
            ctrl.page += 1;

            if ( ctrl.queryKey ) {
                ctrl.doQueryKey();
            } else {
                ctrl.getKeys();
            }
        }
    }
});
