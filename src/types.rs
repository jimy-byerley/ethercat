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

pub use ethercat_types::*;

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

#[derive(Debug, Clone, Copy)]
pub enum WatchdogMode {
    Default,
    Enable,
    Disable,
}

/// Sync Manager Info
#[derive(Debug, Copy, Clone)]
pub struct SmInfo {
    pub idx: u8,
    pub start_addr: u16,
    pub default_size: u16,
    pub control_register: u8,
    pub enable: bool,
    pub pdo_count: u8,
}

/// Sync Manager Config
#[derive(Debug, Clone, Copy)]
pub struct SmCfg {
    pub idx: u8,
    pub watchdog_mode: WatchdogMode,
    pub direction: SyncDirection,
}

impl SmCfg {
    pub const fn input(idx: u8) -> Self {
        Self {
            idx,
            direction: SyncDirection::Input,
            watchdog_mode: WatchdogMode::Default,
        }
    }
    pub const fn output(idx: u8) -> Self {
        Self {
            idx,
            direction: SyncDirection::Output,
            watchdog_mode: WatchdogMode::Default,
        }
    }
}

/// PDO Config
#[derive(Debug, Clone)]
pub struct PdoCfg {
    pub idx: PdoIdx,
    pub entries: Vec<PdoEntryInfo>,
}

impl PdoCfg {
    pub const fn new(idx: PdoIdx) -> PdoCfg {
        Self {
            idx,
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
