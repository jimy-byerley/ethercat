use ethercat::{
    AlState, Master, MasterAccess, Offset, 
    PdoCfg, Sdo, SdoItem, PdoEntryInfo, SlaveAddr, SlaveId, SmCfg,
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
        log::info!("PDO offsets of Slave {}:", *s);
        for (sdo, (bit_len, offset)) in o {
            log::info!(
                " - {:X}:{:X} - {:?}, bit length: {}",
                sdo.index,
                sdo.sub.unwrap(),
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
		HashMap<u16, HashMap<Sdo, (BitLen, Offset)>>,
	),
	io::Error,
> {
	
	let rx_pdos = vec![
		PdoCfg {
			index: 0x1704,
			entries: vec![
				PdoEntryInfo {
					entry: Sdo {index: 0x6040, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "control".to_owned(),
					pos: 0,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x607a, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "position".to_owned(),
					pos: 1,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60ff, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "velocity".to_owned(),
					pos: 2,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x6071, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "torque".to_owned(),
					pos: 3,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x6060, sub: SdoItem::Sub(0)},
					bit_len: 8,
					name: "mode".to_owned(),
					pos: 4,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60b8, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "touch".to_owned(),
					pos: 5,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x607f, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "max velocity".to_owned(),
					pos: 6,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60e0, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "positive torque limit".to_owned(),
					pos: 7,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60e1, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "negative torque limit".to_owned(),
					pos: 8,
					},
				],
			},
		];
		
	let tx_pdos = vec![
		PdoCfg {
			index: 0x1b04,
			entries: vec![
				PdoEntryInfo {
					entry: Sdo {index: 0x603f, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "error".to_owned(),
					pos: 0,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x6041, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "status".to_owned(),
					pos: 1,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x6064, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "position".to_owned(),
					pos: 2,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x6077, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "torque".to_owned(),
					pos: 3,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x6061, sub: SdoItem::Sub(0)},
					bit_len: 8,
					name: "mode".to_owned(),
					pos: 4,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60b9, sub: SdoItem::Sub(0)},
					bit_len: 16,
					name: "touch status".to_owned(),
					pos: 5,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60ba, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "touch value 1".to_owned(),
					pos: 6,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60bc, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "touch value 1".to_owned(),
					pos: 7,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x60fd, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "digital inputs".to_owned(),
					pos: 8,
					},
				PdoEntryInfo {
					entry: Sdo {index: 0x606c, sub: SdoItem::Sub(0)},
					bit_len: 32,
					name: "velocity".to_owned(),
					pos: 9,
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
		let mut entry_offsets: HashMap<Sdo, (u8, Offset)> = HashMap::new();
		
		let sm = SmCfg::output(2.into());
		config.config_sync_manager(&sm)?;
        config.clear_pdo_assignments(sm.index)?;
        for pdo in &rx_pdos {
            config.add_pdo_assignment(sm.index, pdo.index)?;
			config.clear_pdo_mapping(pdo.index)?;
			for entry in &pdo.entries {
				config.add_pdo_mapping(pdo.index, entry)?;
// 				let offset = config.register_pdo_entry(entry.entry, domain_idx)?;
// 				entry_offsets.insert(entry.entry, (entry.bit_len, offset));
			}
		}
		
		let sm = SmCfg::input(3.into());
		config.config_sync_manager(&sm)?;
        config.clear_pdo_assignments(sm.index)?;
        for pdo in &tx_pdos {
            config.add_pdo_assignment(sm.index, pdo.index)?;
			config.clear_pdo_mapping(pdo.index)?;
			for entry in &pdo.entries {
				config.add_pdo_mapping(pdo.index, entry)?;
// 				let offset = config.register_pdo_entry(entry.entry, domain_idx)?;
// 				entry_offsets.insert(entry.entry, (entry.bit_len, offset));
			}
		}
		
		for pdo in &rx_pdos {
			// Positions of RX PDO
			log::info!("Positions in RX PDO 0x{:X}:", pdo.index);
			for entry in &pdo.entries {
				let offset = config.register_pdo_entry(entry.entry, domain_idx)?;
// 				log::info!("  {:?}    {:?} {:?}", entry.entry, offset, entry_offsets[&entry.entry]);
				log::info!("  {:?}  {}", offset, entry.name);
				entry_offsets.insert(entry.entry, (entry.bit_len, offset));
			}
		}
		for pdo in &tx_pdos {
			// Positions of TX PDO
			log::info!("Positions in TX PDO 0x{:X}:", pdo.index);
			for entry in &pdo.entries {
				let offset = config.register_pdo_entry(entry.entry, domain_idx)?;
// 				log::info!("  {:?}    {:?} {:?}", entry.entry, offset, entry_offsets[&entry.entry]);
				log::info!("  {:?}  {}", offset, entry.name);
				entry_offsets.insert(entry.entry, (entry.bit_len, offset));
			}
		}

		let cfg_info = config.info()?;
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
