use crate::{
	master::*,
	types::*,
	field::*,
	Sdo, SyncDirection,
	};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// error in mapping resolution
#[derive(Debug, Error)]
pub enum MappingError {
	#[error("There is not enough configurable PDOs to map these objects")]
	LackOfPdo,
	#[error("There is not enough sync managers to transmit these PDOs")]
	LackOfSync,
}

/*
	- gerer les vitesses des sync managers
	- preference pour les directions par defaut des sync managers
	
	- essayer une classe Robot avec une configuration manuelle des pdos
*/

struct MasterConfigurator<'a> {
	dictionnary: HashMap<Sdo, (u8, TypeId)>,
	inventory: &'a MappingInventory,
	master: &'a Master,
	domain: usize,
	entries: HashMap<u16, ConfigEntry>,
}
#[derive(Default, Debug)]
struct ConfigEntry {
	inputs: Vec<Sdo>,
	outputs: Vec<Sdo>,
	offsets: HashMap<Sdo, (usize, u8)>,
}

impl<'a> MasterConfigurator<'a> {
	pub fn new(master: &'a Master, inventory: &'a MappingInventory, dictionnary: HashMap<Sdo, (u8, TypeId)>) -> Result<Self> {
		Ok(MasterConfigurator{
			dictionnary,
			inventory,
			master,
			domain: master.create_domain()?,
			entries: HashMap::new(),
			})
	}
	/// declare that the given SDO is needed
	pub fn require(&mut self, slave: u16, sdo: &Sdo, direction: SyncDirection) {
		let entry = self.entries
					.entry(slave)
					.or_insert_with(|| ConfigEntry::default());
		match direction {
			SyncDirection::Output => entry.outputs.push(sdo.clone()),
			SyncDirection::Input => entry.inputs.push(sdo.clone()),
			SyncDirection::Invalid => unimplemented!("sync direction must be defined for an SDO requirement"),
		}
	}
			
	/// find a way to map all the previously required SDOs to PDOs and PDOs to sync managers
	pub fn resolve(&mut self, fixed: &[u16], configurable: &[u16], syncs: &[u8]) -> Result<()> {
		for (&slave, entries) in &self.entries {
			// determine which pdos will be used and which sync managers they will be assigned to
			let mapping: MappingInventory = todo!();
// 			let mapping = Self::solve(self.inventory.clone(), configurable, &entries.outputs)?;
			
			// operate mapping on the slaves
			let mut config = self.master.configure_slave(SlaveAddr::ByPos(slave), self.master.get_slave_info(slave)?.id)?;
			for (sync, pdos) in mapping.syncs {
				config.config_sync_manager(&SmCfg::output(sync))?;
				config.clear_pdo_assignments(sync)?;
				for pdo in pdos {
					config.add_pdo_assignment(sync, pdo)?;
					config.clear_pdo_mapping(pdo)?;
					for (i, &entry) in mapping.pdos[&pdo].iter().enumerate() {
						config.add_pdo_mapping(pdo, &PdoEntryInfo{
							pos: i as u8, 
							entry: entry,
							bit_len: self.dictionnary[&entry].0,
							name: String::new(),
							})?;
					}
				}
			}
			for &sdo in entries.inputs.iter().chain(&entries.outputs) {
				let offset = config.register_pdo_entry(sdo, self.domain)?;
				entries.offsets.insert(sdo, (offset.byte, offset.bit as u8));
			}
			
			todo!("gerer les inputs");
		}
		Ok(())
	}
	/// retreive the field offset of the previously required SDO in the resolved mapping
	pub fn request<T: DType>(&self, slave: u16, sdo: Sdo) -> Result<Field<T>> {
		assert!(T::id() == self.dictionnary[&sdo].1);
		let (byte, bit) = self.entries[&slave].offsets[&sdo];
		Ok(Field::new(byte, bit, self.dictionnary[&sdo].0.into()))
	}
	
	/*
	fn inventorize(master: &Master, slave: u16, fixed: &[u16], configurable: &[u16], syncs: &[u8]) -> Result<MappingInventory> {
		// mappings
		let mut mapping = MappingInventory {
			pdos: HashMap::<u16, Vec<Sdo>>::new(),
			syncs: HashMap::<u8, Vec<u16>>::new(),
			};
		
		// inventorize pdos
		for pdo in fixed.iter()
					.chain(configurable)
					.map(|i|  master.get_pdo(slave, i)?) {
			let entries = (0 .. pdo.entry_count)
					.map(|i| master.get_pdo_entry(slave, pdo.index, i)?.entry)
					.collect();
			mapping.pdos.insert(pdo.index, entries);
		}
		
		// inventorize syncs
		for sync in syncs.iter()
					.map(|&i| master.get_sync(slave, i)?) {
			let entries = Vec::with_capacity(sync.default_size.into());
			mapping.syncs.insert(sync, entries);
		}
		
		Ok(mapping)
	}
	*/
	
	fn solve(mapping: MappingInventory, configurable: &[u16], entries: &[Sdo]) -> core::result::Result<MappingInventory, MappingError> {
		// configurable pdos, we will use them when fixed pdos are not fitted
		// we will start by trying to use fixed PDOs and then complete with configurable ones
		let configurable = configurable.iter().cloned().collect::<HashSet<u16>>();
		let mut mapping = mapping;
		
		// select pdos on their exclusive coverage
		let mut used = HashSet::<u16>::new();
		let mut reached = entries.iter().cloned().map(|e| (e,false) ).collect::<HashMap::<Sdo, bool>>();
		
		// find the remaining pdo with maximum coverage
		// complexity: O(n**2)
		while let Some((pdo, entries)) = mapping.pdos.iter()
							.filter(|(pdo, entries)|  !configurable.contains(pdo))
							.max_by_key(|(pdo, entries)| entries
									.iter()
									.map(|entry| reached.get(entry) == Some(&false))
									.count()
									) {
			used.insert(*pdo);
		}
		
		// assign remaining items to configurable pdos
		let mut it = reached.iter().filter_map(|(pdo, done)|  if ! done {Some(pdo)} else {None});
		'assign: for pdo in configurable.iter() {
			for item in mapping.pdos
							.get_mut(pdo)
							.unwrap()
							.iter_mut() {
				match it.next() {
					Some(sdo) => {used.insert(*pdo); *item = *sdo},
					None => break 'assign,
				}
			}
		}
		if it.next().is_some()  {return Err(MappingError::LackOfPdo)}
		
		// assign pdos to sync managers
		let mut it = used.iter();
		'assign: for sync in mapping.syncs.values_mut() {
			for _ in 0 .. sync.capacity() {
				match it.next() {
					Some(pdo) => sync.push(*pdo),
					None => break 'assign,
				}
			}
		}
		if it.next().is_some()  {return Err(MappingError::LackOfSync)}
		
		Ok(mapping)
	}
}

#[derive(Clone)]
struct MappingInventory {
	/// sdo configured in pdos
	pdos: HashMap<u16, Vec<Sdo>>,
	/// pdo configured in sync managers
	syncs: HashMap<u8, Vec<u16>>,
}
