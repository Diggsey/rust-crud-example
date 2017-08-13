
// This trait is used to automatically upgrade old versions of JSON
// objects when they are fetched from the database.
pub trait Upgrade {
    // The "Wrapper" is the unversioned type used throughout the code,
    // and just wraps the latest version of the JSON object.
    type Wrapper;
    // The "Target" is the *next* version of this JSON object in the upgrade
    // chain, so the associated Wrapper must be the same.
    type Target: Upgrade<Wrapper=Self::Wrapper>;

    // Upgrade to the next version.
    fn upgrade(self) -> Self::Target;
    // Upgrade all the way to the latest version.
    fn upgrade_full(self) -> Self::Wrapper;
}

// This automatically implements the required traits to allow storing a serializable
// type in the database, using a JSONB column.
macro_rules! register_json_type {
    ($t:ty) => (
        // Implement "FromSql" for the type by deserializing with serde_json
        impl<'a> $crate::diesel::types::FromSql<$crate::diesel::types::Jsonb, $crate::diesel::pg::Pg> for $t {
            fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<::std::error::Error+Send+Sync>> {
                let bytes = match bytes {
                    Some(bytes) => bytes,
                    None => return Err(Box::new($crate::diesel::types::impls::option::UnexpectedNullError {
                        msg: "Unexpected null for non-null column".to_string(),
                    })),
                };
                if bytes[0] != 1 {
                    return Err("Unsupported JSONB encoding version".into());
                }
                serde_json::from_slice(&bytes[1..])
                    .map_err(|e| Box::new(e) as Box<::std::error::Error+Send+Sync>)
            }
        }

        // Implement "ToSql" for the type by serializing with serde_json
        impl $crate::diesel::types::ToSql<$crate::diesel::types::Jsonb, $crate::diesel::pg::Pg> for $t {
            fn to_sql<W: ::std::io::Write>(&self, out: &mut $crate::diesel::types::ToSqlOutput<W, $crate::diesel::pg::Pg>) -> Result<$crate::diesel::types::IsNull, Box<::std::error::Error+Send+Sync>> {
                try!(out.write_all(&[1]));
                serde_json::to_writer(out, self)
                    .map(|_| $crate::diesel::types::IsNull::No)
                    .map_err(|e| Box::new(e) as Box<::std::error::Error+Send+Sync>)
            }
        }

        // Defer to diesel's macros to make this type act like a JSONB column
        queryable_impls!(Jsonb -> $t);
        expression_impls!(Jsonb -> $t);
    )
}

// Automatically define a sequence of versions for a type, and the
// "Upgrade" machinery for converting between them.
macro_rules! version_json_type {
    (@inner
        #[derive($($der:ident),*)]
        $module:ident $name:ident {
            $oldest_version:ident => $oldest_type:ty,
            $(
                {
                    $($conv:tt)+
                } $older_version:ident => $older_type:ty,
                $later_version:ident => $later_type:ty,
            )*
            {} $latest_version:ident => $latest_type:ty,
        }
    ) => (
        // Put everything in a module to avoid contaminating the parent namespace
        mod $module {
            use super::*;
            use $crate::serde::{Serializer, Deserializer, Serialize, Deserialize};
            use $crate::macros::json::Upgrade;
            use $crate::diesel::types::Jsonb;

            // This enum is used when deserializing. It contains a variant for
            // each historical version of the type.
            #[derive(Deserialize, Debug)]
            enum Version {
                $(
                    $older_version($older_type),
                )*
                $latest_version($latest_type)
            }
            // This enum is used when serializing. It contains a single variant
            // containing the latest version.
            #[derive(Serialize, Debug)]
            enum RefVersion<'a> {
                $latest_version(&'a $latest_type)
            }

            // This is the version-less wrapper type used elsewhere in the code.
            // It's a simple public new-type around the latest version.
            #[derive(Debug $(, $der)*)]
            pub struct $name(pub $latest_type);
            register_json_type!($name);

            // Implement "Upgrade" for each historic version
            $(
                impl Upgrade for $older_type {
                    type Wrapper = $name;
                    type Target = $later_type;
                    fn upgrade(self) -> Self::Target {
                        // Use provides conversion code
                        let $module = self;
                        { $($conv)* }
                    }
                    fn upgrade_full(self) -> Self::Wrapper {
                        // Since this is not the latest version, we can do
                        // a full upgrade by upgrading once, and then
                        // recursively calling "upgrade_full".
                        self.upgrade().upgrade_full()
                    }
                }
            )*
            // Implement "Upgrade" for the latest version
            impl Upgrade for $latest_type {
                type Wrapper = $name;
                type Target = $latest_type;
                fn upgrade(self) -> Self::Target {
                    // Reached the end of the upgrade chain
                    panic!("Cannot upgrade latest version")
                }
                fn upgrade_full(self) -> Self::Wrapper {
                    // We are already the latest version, so just
                    // wrap in the wrapper type.
                    $name(self)
                }
            }

            // Serialize just delegates to `RefVersion::serialize"
            impl Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer
                {
                    let inner = RefVersion::$latest_version(&self.0);
                    inner.serialize(serializer)
                }
            }
            // Deserialize delegates to `Version::deserialize", and then
            // calls "upgrade_full".
            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where D: Deserializer<'de>
                {
                    let inner = try!(Version::deserialize(deserializer));
                    Ok(match inner {
                        $(
                            Version::$older_version(x) => x.upgrade_full(),
                        )*
                        Version::$latest_version(x) => x.upgrade_full()
                    })
                }
            }
        }
        // Export only the wrapper type
        pub use self::$module::$name;
    );
    (
        #[derive(Debug $(, $der:ident)*)]
        $module:ident $name:ident {
            $(
                $old_version:ident => $old_type:ty {
                    $($conv:tt)*
                }
            ),*
        }
    ) => (
        version_json_type!(@inner
            #[derive($($der),*)]
            $module $name {
                $(
                    $old_version => $old_type,
                    {
                        $($conv)*
                    } $old_version => $old_type,
                )*
            }
        );
    )
}
