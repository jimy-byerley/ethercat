use ethercat::{Master, Sdo, Field};
use std::borrow::Cow;
use ndarray::{Array1, ArrayView1};
use packing::Packed;

/*
fn main() -> ethercat::Result<()> {
	let robot = RobotConfig::new(Cow::Borrowed([profile; 5].as_slice()));
	
	let config = master.config()?;
	robot.require(config);
	config.resolve()?;
	let robot = robot.request(config)?;
	
	master.activate()?;
	
	robot.target([0.; 5])?;
	Ok(())
}
*/

fn main() {}

/** standard operation modes (control loops) of a servo drive, 

	there might be only few of them actually implemented by one drive
*/
#[derive(Packed)]
pub enum OperationMode {
	Off = 0,
	ProfilePosition = 1,
	Velocity = 2,
	ProfileVelocity = 3,
	TorqueProfile = 4,
	Homing = 6,
	InterpolatedPosition = 7,
	
	SynchronousPosition = 8,
	SynchronousVelocity = 9,
	SynchronousTorque = 10,
	SynchronousTorqueCommutation = 11,
}

/**
Control word of a servo drive

| Bit	|	Category	|   Meaning	|
|-------|---------------|-----------|
| 0	|	M	|	Switch on |
| 1	|	M	|	Enable voltage |
| 2	|	O	|	Quick stop |
| 3	|	M	|	Enable operation |
| 4 – 6	|	O	|	Operation mode specific |
| 7	|	M	|	Fault reset |
| 8	|	O	|	Halt |
| 9	|	O	|	Operation mode specific |
| 10	|	O	|	reserved |
| 11 – 15	|	O	|	Manufacturer specific |
*/
#[derive(Packed)]
#[packed(big_endian, msb0)]
pub struct ControlWord {
	// pkd(start_bit, end_bit, start_byte, end_byte)   is zero-based and must have start==end to signify length 1 item
	
	#[pkd(0,0,0,0)]  switch_on: bool,
	#[pkd(1,1,0,0)]  enable_voltage: bool,
	#[pkd(2,2,0,0)]  quick_stop: bool,
	#[pkd(3,3,0,0)]  enable_operation: bool,
	#[pkd(7,7,0,0)]  reset_fault: bool,
	#[pkd(0,0,1,1)]  halt: bool,
}


/// needed data to control a joint
#[derive(Clone, Debug)]
pub struct Joint {
	pub slave: u16,
	
	/// SDOs for motion current values
	pub current: JointCurrent,
	/// SDOs for motion control
	pub control: JointControl,
	
	pub position_unit: f32,
	pub force_unit: f32,
	
	pub pmin: f32,
	pub pmax: f32,
	pub vmax: f32,
	pub amax: f32,
	pub fmax: f32,
	}
#[derive(Clone, Debug)]
pub struct JointCurrent {
	/// drive status word
	pub status: Sdo,
	/// mode of operation (motion control loop in use)
	pub mode: Sdo,
	
	pub position: Sdo,
	pub velocity: Sdo,
	pub force: Sdo,
}
#[derive(Clone, Debug)]
pub struct JointControl {
	/// drive control word
	pub control: Sdo,
	/// mode of operation (motion control loop to be used)
	pub mode: Sdo,
	
	pub position: Sdo,
	pub velocity: Sdo,
	pub acceleration: Sdo,
	pub force: Sdo,
	pub profile: JointControlProfile,
}
#[derive(Clone, Debug)]
pub struct JointControlProfile {
	pub velocity: Sdo,
	pub acceleration: Sdo,
	pub deceleration: Sdo,
}

impl Default for JointCurrent {
	fn default() -> Self {Self{
		status: Sdo::complete(0x6041),
		mode: Sdo::complete(0x605c),
		position: Sdo::complete(0x6064),
		velocity: Sdo::complete(0x606c),
		force: Sdo::complete(0x6071),
	}}
}
impl Default for JointControl {
	fn default() -> Self {Self{
		control: Sdo::complete(0x6040),
		mode: Sdo::complete(0x6060),
		position: Sdo::complete(0x602c),
		velocity: Sdo::complete(0x606b),
		.. Default::default()
	}}
}
impl Default for JointControlProfile {
	fn default() -> Self {Self{
		velocity: Sdo::complete(0x6081),
		acceleration: Sdo::complete(0x6083),
		deceleration: Sdo::complete(0x6084),
	}}
}

/// offsets to process data for a joint, fields are matching sdos in [Joint]
#[derive(Clone, Debug)]
pub struct Offsets {
	pub current: OffsetsCurrent,
	pub control: OffsetsControl,
}
#[derive(Clone, Debug)]
struct OffsetsCurrent {
	pub status: Field<u16>,
	pub mode: Field<u8>,
	pub position: Field<i32>,
	pub velocity: Field<i32>,
	pub force: Field<i16>,
}
#[derive(Clone, Debug)]
pub struct OffsetsControl {
	pub control: Field<u16>,
	pub mode: Field<u8>,
	pub position: Field<i32>,
	pub velocity: Field<i32>,
	pub acceleration: Field<i32>,
	pub force: Field<i16>,
}

/// report a motion control error, due to the control loop or external events.
enum ControlError {
	Ethercat(ethercat::Error),
	PositionBounds(PyArray1<f32>),
	Trajectory(f32),
	Aborted,
}
type ControlResult = Result<(), ControlError>;

/// robot control structure
struct Robot<'a> {
	joints: Cow<'a, [Joint]>,
	offsets: Vec<Offsets>,
	period: f32,
	
	master: Master,
	enable_limits: bool,
	fault_freeze: bool,
	interrupt: AtomicBool,
}

/*
/// robot constructor
struct RobotConfig<'a> {
	master: MasterConfig,
	joints: Cow<'a, [Joint]>,
	period: f32,
}

impl<'a> RobotConfig<'a> {
	fn new(joints: Cow<[Joint]>, period: f32) -> Self {
		Self{joints, period}
	}
	/// set sdos to be received in pdos
	fn require(&self, config: &MasterConfig) {
		for joint in self.joints {
			config.require(joint.slave, joint.status);
			config.require(joint.slave, joint.control);
			todo!()
		}
	}
	/// obtain offsets to sdos
	fn request(&self, config: &MasterConfig) -> Result<Robot<'a>> {
		Ok(Robot {
			period: self.period,
			master: config.master,
			joints: self.joints,
			offsets: self.joints.map(|joint| Offsets {
				status: config.request(joint.slave, joint.status),
				control: config.request(joint.slave, joint.control),
				.. todo!()
				}),
		})
	}
}
*/

/* FEATURES
	- en cas d'arret de la boucle de mouvement:
		+ le robot continue meme vitesse puis attenue
		+ en cas d'augmentation de couple, decelere
	- la boucle de mouvement courante peut etre interrompue et une autre lancée sans irrégularité sur la trajectoire
	
*/

struct Master {
	master: ethercat::Master,
	thread: thread::Thread,
	tasks: Mutex<HashMap<u16, Box<dyn Fn(&Self)> >>,
}

impl<'a> Robot<'a> {
	fn cycle<F>(&self, task: F) -> ControlResult {
		self.abort.lock().set(false);
		while !self.abort.lock().get() {
			self.master.cycle();
			if task()?  {return Ok(())}
		}
		Err(Aborted)
	}
	fn abort(&self) {
		self.interrupt = true;
	}
	fn task<F>(self: Rc<Self>, task: F) {
		self.abort();
		self.taskid = Some(self.master.task(task));
	}
	
	fn trajectory<F>(self: Rc<Self>, trajectory: F) -> ControlResult
	where F: Fn(f32) -> Option<Array1<f32>> {
		self.interrupt = true;
		let mut t = 0.;
		
		let data = self.master.data();
		for joint in self.offsets {
			joint.control.position = trajectory(t).expect("a trajectory must have instant 0") * joint.position_unit;
			joint.control.mode.set(data, OperationMode::SynchronousPosition);
		}
		self.task(Box::new(|| {
			let data = self.master.data();
			if self.interrupt  {
				let start = self.master.time();
				let initial = zip(zip(self.joints, self.offsets), targets)
					.map(|((joint, offsets), target)| {
						let position = offsets.current.position.get(data).into() / joint.position_unit;
						let velocity = offsets.current.velocity.get(data).into() / joint.position_unit;
						(position, velocity)
					})
					.collect::<Vec<_>>();
				self.task(Box::new(|| {
					let data = self.master.data();
					let position = offsets.current.position.get(data).into() / joint.position_unit;
					let velocity = offsets.current.velocity.get(data).into() / joint.position_unit;
					for ((joint, offsets), (pinit, vinit)) in self.joints.iter().zip(self.offsets).zip(initial) {
						if self.master.date() - start > self.transition.keep {
							offsets.control.position.set(data, todo!());
						}
						else {
							offsets.control.position.set(data, position + vinit * (self.master.date() - start));
						}
					}
				}));
			}
			match trajectory(t) {
				None => true,
				Some(targets) => {
					for ((joint, offsets), target) in self.joints.iter().zip(self.offsets).zip(targets) {
						let position = offsets.current.position.get(data).into() / joint.position_unit;
						let velocity = offsets.current.velocity.get(data).into() / joint.position_unit;
						
						let pinc = joint.amax * self.period;
						let mut target = target
										// enforce position limits
										.clamp(joint.pmin, joint.pmax)
										// enforce velocity limits
										.clamp(position - pinc, position + pinc);
						
						// enforce low speed near the position limits
						let dzone = velocity.ipow(2)/(2.*joint.amax);
						if velocity <= 0. && target < joint.pmin + dzone {
							target = target.max(position + velocity*self.period + 0.5*joint.amax*self.period.ipow(2));
						}
						if velocity >= 0. && target > joint.pmax + dzone {
							target = target.min(position + velocity*self.period - 0.5*joint.amax*self.period.ipow(2));
						}
						
						offsets.control.position.set(data, (target * joint.position_unit).into());
					}
					t += self.period;
					false
				},
			}
		}))
	}
	fn target(self: Rc<Self>, pose: ArrayView1<f32>, vfactor: f32, afactor: f32) -> ControlResult {
		for (offsets, position) in self.offsets.zip(pose) {
			let data = self.master.data();
			offsets.control.mode.set(data, OperationMode::ProfilePosition);
			offsets.control.position.set(data, (position.clamp(joint.pmin, joint.pmax) * joint.position_unit).into());
			offsets.control.profile.velocity.set(data, (joint.vmax * vfactor.clamp(0., 1.) * joint.position_unit).into());
			offsets.control.profile.acceleration.set(data, (joint.amax * afactor.clamp(0., 1.) * joint.position_unit).into());
		}
		Ok(())
	}
	fn push(self: Rc<Self>, force: ArrayView1<f32>) -> ControlResult {
		self.task(|| {
			let data = self.master.data();
			for ((joint, offsets), force) in self.joints.iter().zip(self.offsets).zip(force) {
				offsets.control.mode.set(data, OperationMode::SynchronousTorque);
				offsets.control.force.set(data, (force * joint.force_unit).into());
			}
			false
		});
		Ok(())
	}
	
	fn wait(&self) {
		todo!()
	}
	
	fn pose(&self) -> Array1<f32> {
		let data = self.master.data();
		self.joints.iter().zip(self.offsets).map(|(joint, offset)|  
			offset.current.position.get(data).into() / joint.position_unit
			).collect()
	}
	fn velocity(&self) -> Array1<f32> {
		let data = self.master.data();
		self.joints.iter().zip(self.offsets).map(|(joint, offset)|  
			offset.current.velocity.get(data).into() / joint.position_unit
			).collect()
	}
	fn force(&self) -> Array1<f32> {
		let data = self.master.data();
		self.joints.iter().zip(self.offsets).map(|(joint, offset)|  
			offset.current.force.get(data).into() / joint.force_unit
			).collect()
	}
}
/*
struct StateCheck {}
impl Step for StateCheck {
	fn step(&mut self, master: &Master) -> ControlResult {
		if self.offsets.find(|offset|   offset.current.status.get(data).fault) {
			for (joint, offset) in zip(self.joints, self.offsets) {
				offset.control.position = offset.current.position.get(data) - offset.current.velocity.get(data).signum() * 
			}
		}
	}
}
struct Trajectory {}
impl Step for Trajectory {
	fn step(&mut self, master: &Master) -> ControlResult {
		match trajectory(t) {
			None => true,
			Some(targets) => {
				for ((joint, offset), target) in self.joints.iter().zip(self.offsets).zip(targets) {
					let pinc = joint.amax * self.period;
					let mut target = position
									.clamp(joint.pmin, joint.pmax)
									.clamp(position - pinc, position + pinc);
					
					// force a low speed near the position limits
					let position = offsets.current.position.get(data).into() / joint.position_unit;
					let velocity = offsets.current.velocity.get(data).into() / joint.position_unit;
					if velocity <= 0. && target < joint.pmin + velocity.ipow(2)/(2.*joint.amax) {
						target = target.max(position + velocity*self.period + 0.5*joint.amax*self.period.ipow(2));
					}
					if velocity >= 0. && target > joint.pmax + velocity.ipow(2)/(2.*joint.amax) {
						target = target.min(position + velocity*self.period - 0.5*joint.amax*self.period.ipow(2));
					}
					
					joint.control.position.set(data, (target * joint.position_unit).into());
				}
				self.t += self.period;
			},
		}
	}
}
*/
