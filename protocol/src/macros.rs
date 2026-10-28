macro_rules! events {
    ($($ty:ident : $name:literal => $direction:ident $(-> $response:ty)?;)*) => {
        $(
            impl $crate::event::Event for $ty {
                const NAME: &'static str = $name;
            }
            impl $crate::event::$direction for $ty {}
            $(
                impl $crate::event::Request for $ty {
                    type Response = $response;
                }
            )?
        )*
    };
}

macro_rules! empty_payload {
    ($($(#[$meta:meta])* $vis:vis struct $name:ident;)*) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
            $vis struct $name;

            impl serde::Serialize for $name {
                fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    serializer.serialize_unit()
                }
            }

            impl<'de> serde::Deserialize<'de> for $name {
                fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    <serde::de::IgnoredAny as serde::Deserialize>::deserialize(deserializer)?;
                    Ok(Self)
                }
            }
        )*
    };
}

macro_rules! inbound {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident($($path:ident)::+),)*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq)]
        $vis enum $name {
            $($variant($($path)::+),)*
        }

        impl $name {
            pub const EVENTS: &'static [&'static str] =
                &[$(<$($path)::+ as $crate::event::Event>::NAME,)*];

            pub fn decode(
                message: &$crate::message::EventMessage,
            ) -> Result<Option<Self>, serde_json::Error> {
                fn inbound<T: $crate::event::ForEmbedder>() {}
                $(inbound::<$($path)::+>();)*
                $(
                    if message.event == <$($path)::+ as $crate::event::Event>::NAME {
                        return message.payload::<$($path)::+>().map(|payload| Some(Self::$variant(payload)));
                    }
                )*
                Ok(None)
            }
        }
    };
}

macro_rules! int_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $repr:ty {
            $($variant:ident = $value:literal,)*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $($variant,)*
        }

        impl $name {
            pub const fn code(self) -> $repr {
                match self {
                    $(Self::$variant => $value,)*
                }
            }

            pub const fn from_code(code: $repr) -> Option<Self> {
                match code {
                    $($value => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serde::Serialize::serialize(&self.code(), serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let code = <$repr as serde::Deserialize>::deserialize(deserializer)?;
                Self::from_code(code).ok_or_else(|| {
                    <D::Error as serde::de::Error>::custom(format_args!(
                        "{code} is not a known {}",
                        stringify!($name)
                    ))
                })
            }
        }
    };
}

macro_rules! flags {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident: $repr:ty {
            $(const $flag:ident = $value:expr;)*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        $vis struct $name($repr);

        impl $name {
            pub const NONE: Self = Self(0);
            $(pub const $flag: Self = Self($value);)*

            pub const fn from_bits(bits: $repr) -> Self {
                Self(bits)
            }

            pub const fn bits(self) -> $repr {
                self.0
            }

            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0;
            }
        }
    };
}
