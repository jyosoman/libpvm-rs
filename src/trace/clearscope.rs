use std::{collections::HashMap, fmt};

use data::{
    node_types::{ConcreteType, Name, PVMDataType::*},
    MetaStore,
};

use ingest::{
    pvm::{PVMError, PVM},
    Parseable,
};

use uuid::Uuid;

lazy_static! {
    static ref HOST_NAMESPACE: Uuid = Uuid::new_v5(&Uuid::nil(), "host");
    static ref PROGRAM_NAMESPACE: Uuid = Uuid::new_v5(&Uuid::nil(), "program");
    static ref PROVTYPE_NAMESPACE: Uuid = Uuid::new_v5(&Uuid::nil(), "provtype");

    static ref PROGRAM: ConcreteType = ConcreteType {
        pvm_ty: Actor,
        name: "program",
        props: hashmap!("host_uuid" => true,
                        "pname" => true,
                        "pid" => false,
                        "ppid" => false,
                        "uid" => true,
                        "start_time" => false,
                        ),
    };

    static ref FILE: ConcreteType = ConcreteType {
        pvm_ty: Store,
        name: "file",
        props: hashmap!("permissions" => true,
                        "file_type" => true,
                        "size_in_bytes" => true,
                        ),
    };
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvKind {
    AccessibilityService = 0,
    ActivityManagement = 1,
    AlarmService = 2,
    AndroidTv = 3,
    AudioIo = 4,
    BackupManager = 5,
    Binder = 6,
    Bluetooth = 7,
    BootEvent = 8,
    BroadcastReceiverManagement = 9,
    Camera = 10,
    Clipboard = 11,
    ComponentManagement = 12,
    ContentProvider = 13,
    ContentProviderManagement = 14,
    Database = 15,
    DeviceAdmin = 16,
    DeviceSearch = 17,
    DeviceUser = 18,
    Display = 19,
    Dropbox = 20,
    Email = 21,
    Experimental = 22,
    File = 23,
    FileSystem = 24,
    FileSystemManagement = 25,
    Fingerprint = 26,
    Flashlight = 27,
    Gatekeeper = 28,
    Hdmi = 29,
    IdleDockScreen = 30,
    Ims = 31,
    Infrared = 32,
    InstalledPackages = 33,
    JsseTrustManager = 34,
    Keychain = 35,
    Keyguard = 36,
    Location = 37,
    MachineLearning = 38,
    Media = 39,
    MediaCapture = 40,
    MediaLocalManagement = 41,
    MediaLocalPlayback = 42,
    MediaNetworkConnection = 43,
    MediaRemotePlayback = 44,
    Midi = 45,
    Native = 46,
    Network = 47,
    NetworkManagement = 48,
    Nfc = 49,
    Notification = 50,
    PacProxy = 51,
    Permissions = 52,
    PersistantData = 53,
    Posix = 54,
    PowerManagement = 55,
    PrintService = 56,
    ProcessManagement = 57,
    ReceiverManagement = 58,
    Rpc = 59,
    ScreenAudioCapture = 60,
    SerialPort = 61,
    ServiceConnection = 62,
    ServiceManagement = 63,
    SmsMms = 64,
    SpeechInteraction = 65,
    StatusBar = 66,
    SyncFramework = 67,
    Telephony = 68,
    Test = 69,
    TextServices = 70,
    Threading = 71,
    TimeEvent = 72, // associated with time ex. change date/time query date/time change timezone
    Ui = 73,
    UidEvent = 74,
    UiAutomation = 75,
    UiMode = 76,
    UiRpc = 77,
    UsageStats = 78,
    Usb = 79,
    UserAccountsManagement = 80,
    UserInput = 81,
    Vibrator = 82,
    WakeLock = 83,
    WallpaperManager = 84,
    Wap = 85,
    WebBrowser = 86,
    Widgets = 87,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValueType {
    Param = 0,
    Src = 1,
    Sink = 2,
    Ret = 3,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventFlow {
    Event = 0,
    Src = 1,
    Sink = 2,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvFlow {
    Src = 0,
    Sink = 1,
}

#[derive(Debug, Deserialize)]
pub struct RLETag {
    length: Option<i32>,
    tag: u32,
}

#[derive(Debug, Deserialize)]
enum EventDataValue {
    Bool(Vec<bool>),
    Byte(String), // decode base64
    Char(String),
    Short(Vec<i32>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<Option<f32>>),
    Double(Vec<Option<f64>>),
    String(Vec<String>),
    Pointer(i32),
    Object {
        obj_type: String,
        hash_code: i32,
        value: Vec<EventData>,
    },
}

#[derive(Debug, Deserialize)]
pub struct EventData {
    name: String,
    value_type: ValueType,
    is_array: Option<bool>,
    data: EventDataValue,
    tag: Vec<RLETag>,
    is_null: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    flow: EventFlow,
    prog_id: u32,
    app_ppt: u32,
    sys_call: u32,
    tid: i64,
    time: i64,
    event_data: Vec<EventData>,
    predicate1_id: Option<u32>, // prov type id
    predicate2_id: Option<u32>, // prov type id
}

#[derive(Debug, Deserialize)]
pub struct DefineAppPpt {
    id: u32,
    prog_id: u32,
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct DefineProgram {
    id: u32,
    host_id: u32,
    pname: String,
    pid: i32,
    ppid: i32,
    uid: i32, //user id assigned by OS
    start_time: i64,
}

impl DefineProgram{
    fn parse(&self, pvm: &mut PVM) -> Result<(), PVMError> {
        let mut p = pvm.declare(
            &PROGRAM,
            Uuid::new_v5(&PROGRAM_NAMESPACE, &self.id.to_string()),
            None,
        );
        pvm.meta(
            &mut p,
            "host_uuid",
            &Uuid::new_v5(&HOST_NAMESPACE, &self.host_id.to_string()).hyphenated(),
        )?;
        pvm.meta(&mut p, "pname", &self.pname)?;
        pvm.meta(&mut p, "pid", &self.pid)?;
        pvm.meta(&mut p, "ppid", &self.ppid)?;
        pvm.meta(&mut p, "uid", &self.uid)?;
        pvm.meta(&mut p, "start_time", &self.start_time)
    }
}

#[derive(Debug, Deserialize)]
pub struct DefineProv {
    flow: ProvFlow,
    id: u32,
    prog_id: u32,
    prov_type: i32,
    app_ppt: u32,
    sys_call: u32,
    prev_id: u32,
    prev_device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DefineProvSet {
    id: u32,
    prog_id: u32,
    child: Vec<u32>,
}

#[derive(Debug, Deserialize)]
enum ProvTypeObject {
    General {
        obj_type: String,
        properties: HashMap<String, String>,
    },
    File {
        path: String,
        permissions: i32,
        file_type: String,
        size_in_bytes: Option<i64>,
    },
    Network {
        local_address: String,
        local_port: i32,
        remote_address: Option<String>,
        remote_port: Option<i32>,
        protocol: i32,
    },
    PacketSocket {
        protocol: i32,
        ifindex: i32,
        hatype: i32,
        pkttype: i32,
        halen: i32,
        addr: String, //decode base64
    },
    Pipe {
        unique_id: String,
    },
    Binder {
        kind: ProvKind,
        properties: HashMap<String, String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct DefineProvType {
    id: u32,
    prog_id: u32,
    object: ProvTypeObject,
}

impl DefineProvType {
    fn parse(&self, pvm: &mut PVM) -> Result<(), PVMError> {
        let uuid = Uuid::new_v5(&PROVTYPE_NAMESPACE, &self.id.to_string());
        match &self.object {
            ProvTypeObject::File {
                path,
                permissions,
                file_type,
                size_in_bytes,
            } => {
                let mut f = pvm.declare(&FILE, uuid, None);
                pvm.name(&f, Name::Path(path.clone()));
                pvm.meta(&mut f, "permissions", permissions)?;
                pvm.meta(&mut f, "file_type", file_type)?;
                if let Some(sb) = size_in_bytes {
                    pvm.meta(&mut f, "size_in_bytes", sb)?;
                }
            },
            _ => {}
        };
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct DefineSysCall {
    id: u32,
    prog_id: u32,
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct HostInfo {
    id: u32,
    hostname: String,
    host_ids: HashMap<String, String>,
    interfaces: Vec<InterfaceInfo>,
    os_details: String,
}

#[derive(Debug, Deserialize)]
pub struct InterfaceInfo {
    name: String,
    mac_address: String,
    ip_addresses: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    user_id: i32,
    name: String,
    groups: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum ProvMessage {
    DefineAppPpt(DefineAppPpt),
    DefineProgram(DefineProgram),
    DefineProv(DefineProv),
    DefineProvSet(DefineProvSet),
    DefineProvType(DefineProvType),
    DefineSysCall(DefineSysCall),
    Event(Event),
    HostInfo(HostInfo),
    User(User),
}

impl fmt::Display for ProvMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Parseable for ProvMessage {
    fn init(pvm: &mut PVM) {
        pvm.new_concrete(&PROGRAM);
        pvm.new_concrete(&FILE);
    }

    fn parse(&self, pvm: &mut PVM) -> Result<(), PVMError> {
        match self {
            ProvMessage::DefineAppPpt(_) => Ok(()),
            ProvMessage::DefineProgram(m) => m.parse(pvm),
            ProvMessage::DefineProv(_) => Ok(()),
            ProvMessage::DefineProvSet(_) => Ok(()),
            ProvMessage::DefineProvType(m) => m.parse(pvm),
            ProvMessage::DefineSysCall(_) => Ok(()),
            ProvMessage::Event(_) => Ok(()),
            ProvMessage::HostInfo(_) => Ok(()),
            ProvMessage::User(_) => Ok(()),
        }
    }
}
