use crate::core::{DataType, LabJackDataType, LabJackDataValue, Quantity};
use crate::prelude::{Address, Error, LabJackEntity};
use num::traits::ToBytes;
use num::{FromPrimitive};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

macro_rules! impl_traits {
    ($($struct:ident => $value:ty),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct $struct;

            impl Coerce for $struct {
                fn coerce(&self, value: <$struct as DataType>::Value) -> LabJackDataValue {
                    LabJackDataValue::$struct(value)
                }
            }

            impl DataType for $struct {
                type Value = $value;

                fn data_type(&self) -> LabJackDataType {
                    LabJackDataType::$struct
                }

                fn bytes(&self, value: &<$struct as DataType>::Value) -> Vec<u8> {
                    value.to_be_bytes().to_vec()
                }
            }
        )*
    };
}

/// Allows for upcasting a given primitive into a [`LabJackDataValue`]
/// for intermediary handling of unknown/aggregated data instances.
pub trait Coerce: DataType {
    fn coerce(&self, value: <Self as DataType>::Value) -> LabJackDataValue; // Must not fail, ever.
}

impl_traits! {
    Uint16 => u16,
    Uint32 => u32,
    Uint64 => u64,
    Int32 => i32,
    Float32 => f32,
    Byte => u8,
}

pub trait Decoder {
    fn decode_primitive(&self, variant: LabJackDataType) -> Result<dyn FromPrimitive, Error>;
}

pub trait Decode: Coerce {
    fn try_decode(&self, v: &dyn Decoder) -> Result<<Self as DataType>::Value, Error>;
}

pub struct StandardDecoder<'a> {
    pub bytes: &'a [u8],
}

pub struct EmulatedDecoder {
    pub value: LabJackDataValue,
}

impl Decoder for EmulatedDecoder {
    fn decode_primitive(&self, _: LabJackDataType) -> Result<LabJackDataValue, Error> {
        Ok(self.value)
        // Apply indirection
        // let as_f64 = self.value.as_f64();
        // F::from_f64(as_f64).ok_or(Error::InvalidData(Reason::DecodingError))
    }
}

impl Decoder for StandardDecoder<'_> {
    fn decode_primitive(&self, variant: LabJackDataType) -> Result<LabJackDataValue, Error> {
        LabJackDataValue::from_bytes(variant, self.bytes)
    }
}

impl<T> Decode for T
where
    T: Coerce,
{
    fn try_decode(&self, v: &dyn Decoder) -> Result<T, Error> {
        v.decode_primitive(self.data_type())
    }
}

/// Defines the ability for a register to be written or read from
/// with the compile-time constraints of an access-control layer.
#[repr(u8)]
enum AccessControl {
    AllCtrl = 1,
    ReadableCtrl = 2,
    WritableCtrl = 3
}

/// A register with an associated constant pertaining to the
/// [`AccessControl`] enumeration which specifies the access
/// possibility of the register itself.
///
/// This means, when using the [`StrongClient`], it enforces
/// invariants in access control over registers with separate
/// reading and writing privileges.
pub struct AccessLimitedRegister<const ACCESS_CONTROL: u8> {
    register: Register,
}

impl<const N: u8> Deref for AccessLimitedRegister<N> {
    type Target = Register;

    fn deref(&self) -> &Self::Target {
        &self.register
    }
}

impl<const N: u8> AccessLimitedRegister<N> {
    /// Allows the consuming application to safely unravel an ACL
    /// locked register into its inner [`Register`]. This is useful
    /// for using the underlying traits
    fn register(self) -> Register {
        self.register
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Register {
    name: &'static str,
    address: u32,
    data_type: LabJackDataType,
    default_value: Option<f64>
}

trait Readable {}
trait Writable {}

use AccessControl::*;

impl Writable for AccessLimitedRegister<{ WritableCtrl as u8 }> {}
impl Readable for AccessLimitedRegister<{ ReadableCtrl as u8 }> {}
impl Writable for AccessLimitedRegister<{ AllCtrl as u8 }> {}
impl Readable for AccessLimitedRegister<{ AllCtrl as u8 }> {}

pub trait __RegisterTrait {
    fn entity(&self) -> LabJackEntity;

    fn address(&self) -> Address;

    fn name(&self) -> &'static str;

    fn width(&self) -> Quantity {
        self.data_type().size()
    }

    fn data_type(&self) -> LabJackDataType {
        self.entity().data_type
    }
}

// -- Elision --

// type BoxUntypedRegister<'a> = Box<
//     dyn 'a
//     + Register<
//         DataType = Box<dyn Decode<Value = dyn PrimInt>>,
//     >,
// >;
//
// struct UntypedRegister<'a, S, Dt>
// where
//     S: Register<DataType = Dt>,
//     Dt: Decode<Value = dyn Any>
// {
//     reg: S,
//     _phantom: PhantomData<Dt>,
// }
//
// pub fn elide<'a, Reg, Dt>(reg: Reg) -> BoxUntypedRegister
// where
//     Reg: Register<DataType = Dt>,
//     Reg::DataType: 'a,
//     Dt: Decode<Value = dyn Any>
// {
//     Box::new(UntypedRegister {
//         reg,
//         _phantom: PhantomData,
//     })
// }
//
// impl<S, Dt> Register for UntypedRegister<S, Dt> where S: Register<DataType = Dt>, Dt: Decode {
//     type DataType = Dt;
//
//     fn data_type(&self) -> Self::DataType {
//         self.reg.data_type()
//     }
//
//     fn entity(&self) -> LabJackEntity {
//         self.reg.entity()
//     }
//
//     fn address(&self) -> Address {
//         self.reg.address()
//     }
//
//     fn name(&self) -> &'static str {
//         self.reg.name()
//     }
// }
