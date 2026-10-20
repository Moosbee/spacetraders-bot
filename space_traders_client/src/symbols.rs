//! Strongly typed symbols used across the SpaceTraders API.
//!
//! The API exposes several stringly-typed identifiers that are easy to mix up
//! (ship symbols, waypoint symbols and system symbols). These newtypes make
//! the distinction explicit while still behaving like the underlying strings
//! where that is convenient.

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

macro_rules! symbol_newtype {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Creates a new symbol from anything that can be turned into a `String`.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the inner string value.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consumes the symbol and returns the inner string value.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = std::convert::Infallible;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Ok(Self(value.to_owned()))
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<String> for $name {
            fn eq(&self, other: &String) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<$name> for String {
            fn eq(&self, other: &$name) -> bool {
                *self == other.0
            }
        }

        impl PartialEq<$name> for &str {
            fn eq(&self, other: &$name) -> bool {
                *self == other.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

symbol_newtype!(ShipSymbol, "The globally unique identifier of a ship.");
symbol_newtype!(WaypointSymbol, "The symbol of a waypoint.");
symbol_newtype!(SystemSymbol, "The symbol of a system.");

impl WaypointSymbol {
    /// Returns the system this waypoint belongs to.
    ///
    /// Waypoint symbols have the form `[SECTOR]-[SYSTEM]-[WAYPOINT]` (for
    /// example `X1-AB12-C3`). The system symbol is the first two segments.
    pub fn get_system_symbol(&self) -> SystemSymbol {
        let mut parts = self.0.split('-');

        let sector = parts.next().unwrap_or_default();
        let system = parts.next().unwrap_or_default();

        SystemSymbol(format!("{sector}-{system}"))
    }
}

impl SystemSymbol {
    pub fn contains_waypoint(&self, waypoint_symbol: &WaypointSymbol) -> bool {
        // same as waypoint_symbol.starts_with(system_symbol)
        waypoint_symbol.starts_with(self.as_str())
    }
}

// GraphQL scalar support so the newtypes can be used in `SimpleObject` and
// resolver signatures as drop-in replacements for `String`.
mod graphql_support {
    use async_graphql::{InputValueError, InputValueResult, ScalarType, Value};

    macro_rules! impl_scalar {
        ($name:ident) => {
            #[async_graphql::Scalar]
            impl ScalarType for super::$name {
                fn parse(value: Value) -> InputValueResult<Self> {
                    match value {
                        Value::String(value) => Ok(super::$name(value)),
                        _ => Err(InputValueError::expected_type(Value::Null)),
                    }
                }

                fn to_value(&self) -> Value {
                    Value::String(self.0.clone())
                }
            }
        };
    }

    impl_scalar!(ShipSymbol);
    impl_scalar!(WaypointSymbol);
    impl_scalar!(SystemSymbol);
}

// sqlx support so the newtypes can be stored and loaded from PostgreSQL just
// like the underlying `String` values.
mod sqlx_support {
    use sqlx::{
        encode::IsNull,
        error::BoxDynError,
        postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
        Decode, Encode, Postgres, Type,
    };

    macro_rules! impl_sqlx {
        ($name:ident) => {
            impl Type<Postgres> for super::$name {
                fn type_info() -> PgTypeInfo {
                    <String as Type<Postgres>>::type_info()
                }

                fn compatible(ty: &PgTypeInfo) -> bool {
                    <String as Type<Postgres>>::compatible(ty)
                }
            }

            impl<'q> Encode<'q, Postgres> for super::$name {
                fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
                    <String as Encode<'q, Postgres>>::encode_by_ref(&self.0, buf)
                }
            }

            impl<'r> Decode<'r, Postgres> for super::$name {
                fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
                    <String as Decode<'r, Postgres>>::decode(value).map(super::$name)
                }
            }

            impl sqlx::postgres::PgHasArrayType for super::$name {
                fn array_type_info() -> PgTypeInfo {
                    <Vec<String> as Type<Postgres>>::type_info()
                }

                fn array_compatible(ty: &PgTypeInfo) -> bool {
                    <Vec<String> as Type<Postgres>>::compatible(ty)
                }
            }
        };
    }

    impl_sqlx!(ShipSymbol);
    impl_sqlx!(WaypointSymbol);
    impl_sqlx!(SystemSymbol);
}
