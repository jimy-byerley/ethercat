use std::{
	marker::PhantomData,
	convert::TryInto,
	fmt,
	};

/// locate some data in a datagram, which must be extracted to type `T` to be processed in rust
#[derive(Default, Clone)]
pub struct Field<T: DType> {
	dtype: PhantomData<T>,
	/// start byte index of the object
	pub byte: usize,
	/// start bit index in the start byte
	pub bit: u8,
	/// bit length of the object
	pub bitlen: usize,
}
impl<T: DType> Field<T> {
	/// build a Field from its content
	pub fn new(byte: usize, bit: u8, bitlen: usize) -> Self {
		Self{dtype: PhantomData, byte, bit, bitlen}
	}
	/// extract the value pointed by the field in the given byte array
	pub fn get(&self, data: &[u8]) -> T       {T::from_dfield(self, data)}
	/// dump the given value to the place pointed by the field in the byte array
	pub fn set(&self, data: &mut [u8], value: T)   {value.to_dfield(self, data)}
}
impl<T: DType> fmt::Debug for Field<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Field")
			.field("byte", &self.byte)
			.field("bit", &self.bit)
			.field("bitlen", &self.bitlen)
			.finish()
	}
}
pub trait DType: Sized {
	fn id() -> TypeId;
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self;
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]);
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TypeId {
	CUSTOM,
	BOOL,
	I8, I16, I32,
	U8, U16, U32,
	F32, F64,
}

impl DType for f32 {
	fn id() -> TypeId 	{TypeId::F32}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned floats are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned floats are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
impl DType for f64 {
	fn id() -> TypeId 	{TypeId::F64}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned floats are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned floats are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
impl DType for u32 {
	fn id() -> TypeId 	{TypeId::U32}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
impl DType for u16 {
	fn id() -> TypeId 	{TypeId::U16}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
impl DType for u8 {
	fn id() -> TypeId 	{TypeId::U8}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
impl DType for i32 {
	fn id() -> TypeId 	{TypeId::I32}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
impl DType for i16 {
	fn id() -> TypeId 	{TypeId::I16}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
impl DType for i8 {
	fn id() -> TypeId 	{TypeId::I8}
	
	fn from_dfield(field: &Field<Self>, data: &[u8]) -> Self {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		Self::from_le_bytes(data[field.byte..].try_into().expect("wrong data size"))
	}
	fn to_dfield(&self, field: &Field<Self>, data: &mut [u8]) {
		assert_eq!(field.bit, 0, "bit aligned integers are not supported");
		assert_eq!(field.bitlen, std::mem::size_of::<Self>(), "wrong field size");
		data[field.byte..].copy_from_slice(&self.to_le_bytes());
	}
}
