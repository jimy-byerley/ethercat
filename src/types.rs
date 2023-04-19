// Part of ethercat-rs. Copyright 2018-2022 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

use crate::ec;
use derive_new::new;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No devices available")]
    NoDevices,
    #[error("Sync manager index is too large")]
    SmIdxTooLarge,
    #[error("Invalid domain index {0}")]
    DomainIdx(usize),
    #[error("Kernel module version mismatch: expected {0}, found {1}")]
    KernelModule(u32, u32),
    #[error("Domain is not available")]
    NoDomain,
    #[error("Master is not activated")]
    NotActivated,
    #[error("Invalid AL state 0x{0:X}")]
    InvalidAlState(u8),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

pub use ethercat_types::{SdoEntryAccess, Access, AlState, Offset, DataType};

pub type Result<T> = std::result::Result<T, Error>;
pub type MasterIdx = u32;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DomainDataPlacement {
    pub offset: usize,
    pub size: usize,
}

pub type SlaveConfigIdx = u32;

/// An EtherCAT slave identification, consisting of vendor ID and product code.
#[derive(Debug, Clone, Copy, new)]
pub struct SlaveId {
    pub vendor_id: u32,
    pub product_code: u32,
}

/// An EtherCAT slave revision identification.
#[derive(Debug, Clone, Copy, new)]
pub struct SlaveRev {
    pub revision_number: u32,
    pub serial_number: u32,
}

/// An EtherCAT slave, which is specified either by absolute position in the
/// ring or by offset from a given alias.
#[derive(Debug, Clone, Copy)]
pub enum SlaveAddr {
    ByPos(u16),
    ByAlias(u16, u16),
}

impl SlaveAddr {
    pub(crate) fn as_pair(self) -> (u16, u16) {
        match self {
            SlaveAddr::ByPos(x) => (0, x),
            SlaveAddr::ByAlias(x, y) => (x, y),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MasterInfo {
	/// Number of slaves in the bus. 
    pub slave_count: u32,
    /// true, if the network link is up. 
    pub link_up: bool,
    pub scan_busy: bool,
    /// Application time. 
    pub app_time: u64,
}

#[derive(Debug, Clone)]
pub struct MasterState {
	/// Sum of responding slaves on all Ethernet devices. 
    pub slaves_responding: u32,
    /** Application-layer states of all slaves.

	The states are coded in the lower 4 bits. If a bit is set, it means that at least one slave in the bus is in the corresponding state:

	| Bit | State  |
	|-----|--------|
	| 0   | INIT   |
	| 1   | PREOP  |
	| 2   | SAFEOP |
	| 3   | OP     |
	*/
    pub al_states: u8,
    /// true, if at least one Ethernet link is up. 
    pub link_up: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigInfo {
    pub alias: u16,
    pub position: u16,
    pub id: SlaveId,
    pub slave_position: Option<u16>,
    pub sdo_count: u32,
    pub idn_count: u32,
    // TODO: more attributes are returned:
    // syncs[*], watchdog_*, dc_*
}

/// Slave information
#[derive(Debug, Clone)]
pub struct SlaveInfo {
	/// Display name of the slave
    pub name: String,
	/// Offset of the slave in the ring
    pub ring_pos: u16,
    /// Vendor-ID and product code stored on the slave
    pub id: SlaveId,
    /// Revision-Number stored on the slave
    pub rev: SlaveRev,
    /// The slaves alias if not equal to 0
    pub alias: u16,
    /// Used current in mA
    pub current_on_ebus: i16,
    /// Current state of the slave
    pub al_state: AlState,
    /// Error flag for that slave
    pub error_flag: u8,
    /// Number of sync managers
    pub sync_count: u8,
    /// Number of SDOs
    pub sdo_count: u16,
    /// Port information, statically sized to the max number of ports allowed by this library
    pub ports: [SlavePortInfo; ec::EC_MAX_PORTS as usize],
}

/// EtherCAT slave port descriptor
#[derive(Debug, Clone, Copy)]
pub enum SlavePortType {
	/// Port is not implemented
    NotImplemented,
    /// Port is not configured
    NotConfigured,
    /// Port is an E-Bus
    EBus,
    /// Port is a MII
    MII,
}

impl Default for SlavePortType {
    fn default() -> Self {
        SlavePortType::NotImplemented
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SlavePortLink {
	/// Link detected
    pub link_up: bool,
    /// Loop closed
    pub loop_closed: bool,
    /// Detected signal on RX port
    pub signal_detected: bool,
}

/// port information that can be retreived with a `SlaveInfo`
#[derive(Debug, Default, Clone, Copy)]
pub struct SlavePortInfo {
	/// Physical port type
    pub desc: SlavePortType,
    /// Port link state
    pub link: SlavePortLink,
    /// Receive time on DC transmission delay measurement
    pub receive_time: u32,
    /// Ring position of next DC slave on that port
    pub next_slave: u16,
    /// Delay [ns] to next DC slave
    pub delay_to_next_dc: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SlaveConfigState {
    pub online: bool,
    pub operational: bool,
    pub al_state: AlState,
}

#[derive(Debug, Clone, Copy)]
pub enum SyncDirection {
    Invalid,
    Output,
    Input,
}

/** 
	Watchdog mode for sync manager configuration. 
	
	Used to specify, if a sync manager's watchdog is to be enabled. 
*/
#[derive(Debug, Clone, Copy)]
pub enum WatchdogMode {
	/// Whether it is enabled or not depends n the default setting of the sync manager. 
    Default,
	/// Enable the watchdog. 
    Enable,
    /// Disable the watchdog. 
    Disable,
}

/// Sync Manager Info
#[derive(Debug, Copy, Clone)]
pub struct SmInfo {
	/// index of the SDO that configures the sync manager
    pub index: u8,
    pub start_addr: u16,
    pub default_size: u16,
    pub control_register: u8,
    pub enable: bool,
    /// number of PDO that can be set on this sync manager
    pub pdo_count: u8,
}

/// Sync Manager Config
#[derive(Debug, Clone, Copy)]
pub struct SmCfg {
	/// index of the sync manager on the slave
    pub index: u8,
    pub watchdog_mode: WatchdogMode,
    pub direction: SyncDirection,
}

impl SmCfg {
    pub const fn input(index: u8) -> Self {
        Self {
            index,
            direction: SyncDirection::Input,
            watchdog_mode: WatchdogMode::Default,
        }
    }
    pub const fn output(index: u8) -> Self {
        Self {
            index,
            direction: SyncDirection::Output,
            watchdog_mode: WatchdogMode::Default,
        }
    }
}

/// PDO Config
#[derive(Debug, Clone)]
pub struct PdoCfg {
	/// PDO index on the slave
    pub index: u16,
    /// entries defining the mapping of the PDO to SDO items
    pub entries: Vec<PdoEntryInfo>,
}

impl PdoCfg {
    pub const fn new(index: u16) -> PdoCfg {
        Self {
            index,
            entries: vec![],
        }
    }
}

pub trait SdoData {
    fn data_ptr(&self) -> *const u8 {
        self as *const _ as _
    }
    fn data_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

impl SdoData for u8 {}
impl SdoData for u16 {}
impl SdoData for u32 {}
impl SdoData for u64 {}
impl SdoData for i8 {}
impl SdoData for i16 {}
impl SdoData for i32 {}
impl SdoData for i64 {}
impl SdoData for f32 {}
impl SdoData for f64 {}

impl SdoData for &'_ [u8] {
    fn data_ptr(&self) -> *const u8 {
        self.as_ptr()
    }
    fn data_size(&self) -> usize {
        self.len()
    }
}

#[derive(Debug, Clone)]
pub struct DomainState {
	/// Value of the last working counter. 
    pub working_counter: u32,
    /// Working counter interpretation. 
    pub wc_state: WcState,
    /// Redundant link is in use. 
    pub redundancy_active: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum WcState {
    Zero = 0,
    Incomplete,
    Complete,
}

pub(crate) fn get_sdo_entry_access(read: [u8; 3], write: [u8; 3]) -> SdoEntryAccess {
    SdoEntryAccess {
        pre_op: access(read[0], write[0]),
        safe_op: access(read[1], write[1]),
        op: access(read[2], write[2]),
    }
}

fn access(read: u8, write: u8) -> Access {
    match (read, write) {
        (1, 0) => Access::ReadOnly,
        (0, 1) => Access::WriteOnly,
        (1, 1) => Access::ReadWrite,
        _ => Access::Unknown,
    }
}

impl From<u32> for WcState {
    fn from(st: u32) -> Self {
        match st {
            0 => WcState::Zero,
            1 => WcState::Incomplete,
            2 => WcState::Complete,
            x => panic!("invalid state {}", x),
        }
    }
}

/// SDO Entry Address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SdoEntryAddr {
    ByPos(u16, u8),
    ByIdx(Sdo),
}
/** address of an SDO (Service Data Object)

	An SDO is a variable on a slave that can be read/written through
	
	- the service objects dictionnary in asynchronous (non-realtime) mode
	- the process data objects (PDO) in synchronous (realtime) mode
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Sdo {
	/// index in the dictionnary
	pub index: u16,
	/// part of the SDO to access, can be `Sub(u8)` or `Complete`
	pub sub: SdoItem,
}
/// designate a part or the whole of an SDO
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SdoItem {
	/// designate the subindex of an SDO item
	Sub(u8),
	/// designate the whole SDO
	Complete,
}
impl Sdo {
	/// fast hand to create a SDO address with complete access
	pub fn complete(index: u16) -> Self  {
		Self{index, sub: SdoItem::Complete}
	}
	/// fast hand to create a SDO address with sub item access
	pub fn subitem(index: u16, sub:u8) -> Self {
		Self{index, sub: SdoItem::Sub(sub)}
	}
}
impl SdoItem {
	pub fn unwrap(&self) -> u8 {
		match self {
			SdoItem::Sub(i) => *i,
			SdoItem::Complete => 0,
			}
	}
	pub fn is_complete(&self) -> bool {
		match self {
			SdoItem::Sub(_) => false,
			SdoItem::Complete => true,
		}
	}
}

/// SDO Meta Information
#[derive(Debug, Clone, PartialEq)]
pub struct SdoInfo {
    pub pos: u16, // TODO: do we need this info here?
    /// SDO index in the object dictionnary
    pub index: u16,
    /// number of SDO entries (aka subitems)
    pub entry_count: u8,
    pub object_code: Option<u8>,
    pub name: String,
}
/// SDO Entry Information
#[derive(Debug, Clone, PartialEq)]
pub struct SdoEntryInfo {
	/// type of the data in this entry
    pub data_type: DataType,
    /// bit length of the entry data
    pub bit_len: u16,
    /// access type
    pub access: SdoEntryAccess,
    /// description of the entry, this value is unspecified and vendor-specific
    pub description: String,
}

/// PDO Meta Information
#[derive(Debug, Clone, PartialEq)]
pub struct PdoInfo {
	/// index of the sync manager the PDO belongs to
    pub sm: u8,
    pub pos: u8,
    /// index identifying the PDO
    pub index: u16,
    /// number of entries in the PDO, each entry is an SDO item
    pub entry_count: u8,
    /// description of the PDO, this value is unspecified and vendor-specific
    pub name: String,
}
/// PDO entry information
#[derive(Debug, Clone, PartialEq)]
pub struct PdoEntryInfo {
	/// position in the mapping
    pub pos: u8,
    /// SDO mapped
    pub entry: Sdo,
    /// bit length of the mapped SDO data
    pub bit_len: u8,
    /// name of the SDO data
    pub name: String,
}
