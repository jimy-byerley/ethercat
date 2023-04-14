use ethercat::{
    AlState, DomainIdx as DomainIndex, Idx, Master, MasterAccess, Offset, PdoCfg, PdoEntryIdx,
    PdoEntryIdx as PdoEntryIndex, PdoEntryInfo, PdoEntryPos, PdoIdx, SlaveAddr, SlaveId, SlavePos,
    SmCfg, SubIdx,
};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, prelude::*},
    thread,
    time::Duration,
};

type BitLen = u8;

pub fn main() -> Result<(), io::Error> {
    env_logger::init();
    
    let (mut master, domain_idx, offsets) = init_master()?;
    for (s, o) in &offsets {
        log::info!("PDO offsets of Slave {}:", u16::from(*s));
        for (pdo, (bit_len, offset)) in o {
            log::info!(
                " - {:X}:{:X} - {:?}, bit length: {}",
                u16::from(pdo.idx),
                u8::from(pdo.sub_idx),
                offset,
                bit_len
            );
        }
    }
    let cycle_time = Duration::from_micros(50_000);
    master.activate()?;
    log::info!("master activated");

    loop {
        master.receive()?;
        master.domain(domain_idx).process()?;
        master.domain(domain_idx).queue()?;
        master.send()?;
        let m_state = master.state()?;
        let d_state = master.domain(domain_idx).state();
        log::debug!("Master state: {:?}", m_state);
        log::debug!("Domain state: {:?}", d_state);
        if m_state.link_up && m_state.al_states == 8 {
            let raw_data = master.domain_data(domain_idx);
            log::debug!("{:?}", raw_data);
        }
        thread::sleep(cycle_time);
    }
}

pub fn init_master() -> Result<
	(
		Master,
		usize,
		HashMap<u16, HashMap<PdoEntryIndex, (BitLen, Offset)>>,
	),
	io::Error,
> {
	
	let rx_pdos = vec![
		PdoCfg {
			idx: PdoIdx::from(0x1704),
			entries: vec![
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x6040), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "control".to_owned(),
					pos: PdoEntryPos::from(0),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x607a), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "position".to_owned(),
					pos: PdoEntryPos::from(1),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60ff), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "velocity".to_owned(),
					pos: PdoEntryPos::from(2),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x6071), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "torque".to_owned(),
					pos: PdoEntryPos::from(3),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x6060), sub_idx: SubIdx::from(0)},
					bit_len: 8,
					name: "mode".to_owned(),
					pos: PdoEntryPos::from(4),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60b8), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "touch".to_owned(),
					pos: PdoEntryPos::from(5),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x607f), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "max velocity".to_owned(),
					pos: PdoEntryPos::from(6),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60e0), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "positive torque limit".to_owned(),
					pos: PdoEntryPos::from(7),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60e1), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "negative torque limit".to_owned(),
					pos: PdoEntryPos::from(8),
					},
				],
			},
		];
		
	let tx_pdos = vec![
		PdoCfg {
			idx: PdoIdx::from(0x1b04),
			entries: vec![
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x603f), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "error".to_owned(),
					pos: PdoEntryPos::from(0),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x6041), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "status".to_owned(),
					pos: PdoEntryPos::from(1),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x6064), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "position".to_owned(),
					pos: PdoEntryPos::from(2),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x6077), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "torque".to_owned(),
					pos: PdoEntryPos::from(3),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x6061), sub_idx: SubIdx::from(0)},
					bit_len: 8,
					name: "mode".to_owned(),
					pos: PdoEntryPos::from(4),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60b9), sub_idx: SubIdx::from(0)},
					bit_len: 16,
					name: "touch status".to_owned(),
					pos: PdoEntryPos::from(5),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60ba), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "touch value 1".to_owned(),
					pos: PdoEntryPos::from(6),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60bc), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "touch value 1".to_owned(),
					pos: PdoEntryPos::from(7),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x60fd), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "digital inputs".to_owned(),
					pos: PdoEntryPos::from(8),
					},
				PdoEntryInfo {
					entry_idx: PdoEntryIdx {idx: Idx::from(0x606c), sub_idx: SubIdx::from(0)},
					bit_len: 32,
					name: "velocity".to_owned(),
					pos: PdoEntryPos::from(9),
					},
				],
			},
		];


	let mut master = Master::open("/dev/EtherCAT0", MasterAccess::ReadWrite)?;
	log::info!("Reserve master");
	master.reserve()?;
	log::info!("Create domain");
	let domain_idx = master.create_domain()?;
	
	let mut offsets = HashMap::new();
	
	for slave_pos in 0 .. 1 {
        
        log::info!("Request PreOp state for {:?}", slave_pos);
        master.request_state(slave_pos, AlState::PreOp)?;
        let slave_info = master.get_slave_info(slave_pos)?;
        log::info!("Found device {:?}", slave_info);
        
		
		let mut config = master.configure_slave(
				SlaveAddr::ByPos(slave_pos as u16), 
				slave_info.id)?;
		let mut entry_offsets: HashMap<PdoEntryIndex, (u8, Offset)> = HashMap::new();
		
		let sm = SmCfg::output(2.into());
		config.config_sync_manager(&sm)?;
        config.clear_pdo_assignments(sm.idx)?;
        for pdo in &rx_pdos {
            config.add_pdo_assignment(u8::from(sm.idx), u16::from(pdo.idx))?;
			config.clear_pdo_mapping(u16::from(pdo.idx))?;
			for entry in &pdo.entries {
				config.add_pdo_mapping(u16::from(pdo.idx), entry)?;
				let offset = config.register_pdo_entry(entry.entry_idx, domain_idx)?;
				entry_offsets.insert(entry.entry_idx, (entry.bit_len, offset));
			}
		}
		
		let sm = SmCfg::input(3.into());
		config.config_sync_manager(&sm)?;
        config.clear_pdo_assignments(sm.idx)?;
        for pdo in &tx_pdos {
            config.add_pdo_assignment(u8::from(sm.idx), u16::from(pdo.idx))?;
			config.clear_pdo_mapping(u16::from(pdo.idx))?;
			for entry in &pdo.entries {
				config.add_pdo_mapping(u16::from(pdo.idx), entry)?;
				let offset = config.register_pdo_entry(entry.entry_idx, domain_idx)?;
				entry_offsets.insert(entry.entry_idx, (entry.bit_len, offset));
			}
		}
		
// 		for pdo in &rx_pdos {
// 			// Positions of RX PDO
// 			log::info!("Positions in RX PDO 0x{:X}:", u16::from(pdo.idx));
// 			for entry in &pdo.entries {
// 				let offset = config.register_pdo_entry(entry.entry_idx, domain_idx)?;
// 				log::info!("  {:?}    {:?} {:?}", entry.entry_idx, offset, entry_offsets[&entry.entry_idx]);
// // 				log::info!("  {:?}  {}", offset, entry.name);
// // 				entry_offsets.insert(entry.entry_idx, (entry.bit_len, offset));
// 			}
// 		}
// 		for pdo in &tx_pdos {
// 			// Positions of TX PDO
// 			log::info!("Positions in TX PDO 0x{:X}:", u16::from(pdo.idx));
// 			for entry in &pdo.entries {
// 				let offset = config.register_pdo_entry(entry.entry_idx, domain_idx)?;
// 				log::info!("  {:?}    {:?} {:?}", entry.entry_idx, offset, entry_offsets[&entry.entry_idx]);
// // 				log::info!("  {:?}  {}", offset, entry.name);
// // 				entry_offsets.insert(entry.entry_idx, (entry.bit_len, offset));
// 			}
// 		}

		let cfg_index = config.index();
		let cfg_info = master.get_config_info(cfg_index)?;
		log::info!("Config info: {:#?}", cfg_info);
		if cfg_info.slave_position.is_none() {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Unable to configure slave",
			));
			continue;
		}
		offsets.insert(slave_pos, entry_offsets);
		master.request_state(slave_pos, AlState::Op)?;
	}
	Ok((master, domain_idx, offsets))
}
