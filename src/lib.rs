//! Entity Component System Library (ECS)
//!
//! For info about why an ECS may be beneficial, see some of these articles:
//!
//! - http://gameprogrammingpatterns.com/component.html
//! - http://t-machine.org/index.php/2007/09/03/entity-systems-are-the-future-of-mmog-development-part-1/
//! - http://www.gamedev.net/page/resources/_/technical/game-programming/understanding-component-entity-systems-r3013
//! - http://cowboyprogramming.com/2007/01/05/evolve-your-heirachy/
//!
//! There is a large variety of ways an ECS may work. This particular one is similar to
//! [Artemis](http://gamadu.com/artemis/).
//! Although this isn't a port to Rust, most functionality should be similar, and the
//! tutorials/manual there should be able to make up for the current lack of documentation [FIXME]
//!
//! Here's the basic structure:
//!
//! - An `Entity` is just an identifier. It contains no data or logic whatsoever.
//! - A `Component` is a piece of data (eg: Position, Velocity, Colour). While containing logic can
//! sometimes be useful, it's best practice to avoid it wherever possible.
//! - A `System` runs all the logic. Most of the time, it filters out entities based on their
//! components, and only runs it's logic on the entities it's interested in. These filters are
//! called `Aspect`s. Some systems ignore entities, and just apply logic to the world itself.
//! - An `Aspect` is a simple helper to filter entities based on their components.
//! - The `World` organises all the above items together to make sure everything runs as it should.

#![crate_name = "ecs"]
#![crate_type = "lib"]

#![cfg_attr(feature="nightly", feature(drain))]

#[cfg(feature="serialisation")]
#[macro_use]
extern crate cereal;
extern crate vec_map;

pub use aspect::Aspect;
pub use component::{Component, ComponentList};
pub use component::{EntityBuilder, EntityModifier};
pub use entity::{Entity, IndexedEntity, EntityIter};
pub use system::{System, Process};
pub use world::{ComponentManager, ServiceManager, SystemManager, DataHelper, World};

use std::ops::Deref;

pub mod aspect;
pub mod component;
pub mod entity;
pub mod system;
pub mod world;

pub struct BuildData<'a, T: ComponentManager>(&'a IndexedEntity<T>);
pub struct ModifyData<'a, T: ComponentManager>(&'a IndexedEntity<T>);
pub struct EntityData<'a, T: ComponentManager>(&'a IndexedEntity<T>);
impl<'a, T: ComponentManager> Deref for EntityData<'a, T>
{
    type Target = IndexedEntity<T>;
    fn deref(&self) -> &IndexedEntity<T>
    {
        &self.0
    }
}

impl<'a, T: ComponentManager> Copy for BuildData<'a, T> {}
impl<'a, T: ComponentManager> Copy for ModifyData<'a, T> {}
impl<'a, T: ComponentManager> Copy for EntityData<'a, T> {}

impl<'a, T: ComponentManager> Clone for BuildData<'a, T> { fn clone(&self) -> BuildData<'a, T> { *self } }
impl<'a, T: ComponentManager> Clone for ModifyData<'a, T> { fn clone(&self) -> ModifyData<'a, T> { *self } }
impl<'a, T: ComponentManager> Clone for EntityData<'a, T> { fn clone(&self) -> EntityData<'a, T> { *self } }

#[doc(hidden)]
pub trait EditData<T: ComponentManager> { fn entity(&self) -> &IndexedEntity<T>; }
impl<'a, T: ComponentManager> EditData<T> for ModifyData<'a, T> { fn entity(&self) -> &IndexedEntity<T> { &self.0 } }
impl<'a, T: ComponentManager> EditData<T> for EntityData<'a, T> { fn entity(&self) -> &IndexedEntity<T> { &self.0 } }

// XXX: Eventually make these syntax extensions, once they are stabilised
mod macros
{
    #[macro_export]
    macro_rules! process {
        {
            $world:expr, $system:ident
        } => {
            $crate::Process::process(&mut $world.systems.$system, &mut $world.data)
        };
        {
            $world:expr, $system:ident . $function:ident ($($args:expr),*)
        } => {
            $world.systems.$system.$function($($args,)* &mut $world.data)
        };
    }

    #[macro_export]
    macro_rules! components {
        {
            $(#[$attr:meta])*
            struct $Name:ident;
        } => {
            $(#[$attr])*
            pub struct $Name;

            impl $crate::ComponentManager for $Name
            {
                fn __new() -> $Name
                {
                    $Name
                }

                fn __remove_all(&mut self, _: &$crate::IndexedEntity<$Name>)
                {

                }
            }
        };
        {
            #[builder($Builder:ident)]
            $(#[$attr:meta])*
            struct $Name:ident {
                $(#[$kind:ident] $field_name:ident : $field_ty:ty),+
            }
        } => {
            components!($(#[$attr])* struct $Name { $(#[$kind] $field_name : $field_ty),+ });

            #[derive(Default)]
            pub struct $Builder {
                $(
                    pub $field_name : Option<$field_ty>,
                )+
            }

            impl $crate::EntityBuilder<$Name> for $Builder
            {
                fn build(self, e: $crate::BuildData<$Name>, c: &mut $Name)
                {
                    $(
                        self.$field_name.map(|cmpt| c.$field_name.add(&e, cmpt))
                    );+;
                }
            }
        };
        {
            $(#[$attr:meta])*
            struct $Name:ident {
                $(#[$kind:ident] $field_name:ident : $field_ty:ty),+
            }
        } => {
            $(#[$attr])*
            pub struct $Name {
                $(
                    pub $field_name : $crate::ComponentList<$Name, $field_ty>,
                )+
            }

            impl $crate::ComponentManager for $Name
            {
                fn __new() -> $Name
                {
                    $Name {
                        $(
                            $field_name : $crate::ComponentList::$kind()
                        ),+
                    }
                }

                fn __remove_all(&mut self, entity: &$crate::IndexedEntity<$Name>)
                {
                    $(
                        self.$field_name.__clear(entity)
                    );+
                }
            }
        };
        {
            #[builder($Builder:ident)]
            $(#[$attr:meta])*
            struct $Name:ident {
                $(#[$kind:ident] $field_name:ident : $field_ty:ty),+,
            }
        } => {
            components!(
                #[builder($Builder)]
                $(#[$attr])*
                struct $Name {
                    $(#[$kind] $field_name : $field_ty),+
                }
            );
        };
        {
            $(#[$attr:meta])*
            struct $Name:ident {
                $(#[$kind:ident] $field_name:ident : $field_ty:ty),+,
            }
        } => {
            components!(
                $(#[$attr])*
                struct $Name {
                    $(#[$kind] $field_name : $field_ty),+
                }
            );
        };
    }

    #[macro_export]
    macro_rules! systems {
        {
            $(#[$attr:meta])*
            struct $Name:ident<$components:ty, $services:ty>;
        } => {
            $(#[$attr])*
            pub struct $Name;

            impl $crate::SystemManager for $Name
            {
                type Components = $components;
                type Services = $services;
                fn __new() -> $Name
                {
                    $Name
                }

                fn __activated(&mut self, _: $crate::EntityData<$components>, _: &$components, _: &mut $services)
                {

                }

                fn __reactivated(&mut self, _: $crate::EntityData<$components>, _: &$components, _: &mut $services)
                {

                }

                fn __deactivated(&mut self, _: $crate::EntityData<$components>, _: &$components, _: &mut $services)
                {

                }

                fn __update(&mut self, _: &mut $crate::DataHelper<$components, $services>)
                {

                }
            }
        };
        {
            $(#[$attr:meta])*
            struct $Name:ident<$components:ty, $services:ty> {
                active: {
                    $($field_name:ident : $field_ty:ty = $field_init:expr,)*
                },
                passive: {
                    $($p_field_name:ident : $p_field_ty:ty = $p_field_init:expr,)*
                }
            }
        } => {
            $(#[$attr])*
            pub struct $Name {
                $(pub $field_name : $field_ty,)*
                $(pub $p_field_name : $p_field_ty,)*
            }

            impl $crate::SystemManager for $Name
            {
                type Components = $components;
                type Services = $services;
                fn __new() -> $Name
                {
                    $Name {
                        $(
                            $field_name : $field_init,
                        )*
                        $(
                            $p_field_name : $p_field_init,
                        )*
                    }
                }

                fn __activated(&mut self, en: $crate::EntityData<$components>, co: &$components, se: &mut $services)
                {
                    $(
                        $crate::System::activated(&mut self.$field_name, &en, co, se);
                    )*
                    $(
                        $crate::System::activated(&mut self.$p_field_name, &en, co, se);
                    )*
                }

                fn __reactivated(&mut self, en: $crate::EntityData<$components>, co: &$components, se: &mut $services)
                {
                    $(
                        $crate::System::reactivated(&mut self.$field_name, &en, co, se);
                    )*
                    $(
                        $crate::System::reactivated(&mut self.$p_field_name, &en, co, se);
                    )*
                }

                fn __deactivated(&mut self, en: $crate::EntityData<$components>, co: &$components, se: &mut $services)
                {
                    $(
                        $crate::System::deactivated(&mut self.$field_name, &en, co, se);
                    )*
                    $(
                        $crate::System::deactivated(&mut self.$p_field_name, &en, co, se);
                    )*
                }

                fn __update(&mut self, _co: &mut $crate::DataHelper<$components, $services>)
                {
                    $(
                        $crate::Process::process(&mut self.$field_name, _co);
                    )*
                }
            }
        };
    }

    #[macro_export]
    macro_rules! aspect {
        {
            <$components:ty>
            all: [$($all_field:ident),*]
            none: [$($none_field:ident),*]
        } => {
            $crate::Aspect::__new(Box::new(|_en: &$crate::EntityData<$components>, _co: &$components| {
                ($(_co.$all_field.has(_en) &&)* true) &&
                !($(_co.$none_field.has(_en) ||)* false)
            }))
        };
        {
            <$components:ty>
            all: [$($field:ident),*]
        } => {
            aspect!(
                <$components>
                all: [$($field),*]
                none: []
            )
        };
        {
            <$components:ty>
            none: [$($field:ident),*]
        } => {
            aspect!(
                <$components>
                all: []
                none: [$($field),*]
            )
        };
    }
}
