#[macro_export]
/// Simple macro to create readers types with one value
macro_rules! impl_read {
    (U8, $name: ident, $rwdata:expr) => {
        #[derive(
            Debug, Copy, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Eq, PartialEq,
        )]
        pub struct $name(u8);

        $crate::deref!($name(u8));

        impl $crate::traits::ReadEC for $name {
            fn data_to_read() -> Vec<$crate::RWData> {
                vec![$rwdata]
            }
        }

        impl $name {
            pub async fn current_state(
                ec: &mut impl $crate::traits::ECHandler,
            ) -> ::anyhow::Result<$name> {
                let read_data = ec.read_data::<Self>().await?;
                let RWData::U8 { reg: _, value } = read_data[0] else {
                    unreachable!("Check impl {} for CPUTemp", stringify!($name))
                };
                Ok(Self(value))
            }
        }
    };
    (U16, $name: ident, $rwdata:expr) => {
        #[derive(
            Debug, Copy, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Eq, PartialEq,
        )]
        pub struct $name(u16);

        $crate::deref!($name(u16));

        impl $crate::traits::ReadEC for $name {
            fn data_to_read() -> Vec<$crate::RWData> {
                vec![$rwdata]
            }
        }

        impl $name {
            pub async fn current_state(
                ec: &mut impl $crate::traits::ECHandler,
            ) -> ::anyhow::Result<$name> {
                let read_data = ec.read_data::<Self>().await?;
                let RWData::U16 { reg: _, value } = read_data[0] else {
                    unreachable!()
                };
                Ok(Self(value))
            }
        }
    };
}

#[macro_export]
/// Simple macro to generate impl Deref for new type pattern
macro_rules! deref {
    ($val:ident($type:ty)) => {
        impl ::std::ops::Deref for $val {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
