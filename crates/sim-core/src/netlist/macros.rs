//! Declarative circuit macro.

/// Builds a `Circuit` from a compact declaration.
///
/// `ground` is provided by the macro. Do not declare a node named `ground`.
#[macro_export]
macro_rules! circuit {
    (
        nodes { $($node:ident),* $(,)? }
        ground $ground:ident;
        devices {
            $(
                $device:ident : $kind:expr => [$($pin:ident),* $(,)?]
                $(, params { $($key:ident : $value:expr),* $(,)? })?
                ;
            )*
        }
    ) => {{
        let mut builder = $crate::netlist::CircuitBuilder::new();
        #[allow(unused_variables)]
        let $ground = builder.ground();
        $(
            #[allow(unused_variables)]
            let $node = builder.node(stringify!($node))?;
        )*
        $(
            #[allow(unused_mut)]
            let mut parameters = ::std::collections::BTreeMap::new();
            $(
                $(
                    parameters.insert(stringify!($key).to_string(), $value);
                )*
            )?
            builder.device(
                stringify!($device),
                $kind,
                vec![$($pin),*],
                parameters,
            )?;
        )*
        builder.build()
    }};
}
