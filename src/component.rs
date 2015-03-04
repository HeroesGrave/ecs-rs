
//! Store data in parts to allow defining different entities through composition.

use std::any::TypeId;
use std::collections::BitVec;
use std::mem;

use buffer::Buffer;
use error;
use Entity;

pub trait Component: Copy+'static {}

impl<T:Copy+'static> Component for T {}

pub type ComponentId = TypeId;

#[doc(hidden)]
pub struct ComponentList
{
    buffer: Buffer,
    enabled: BitVec,
    id: ComponentId,
}

impl ComponentList
{
    pub fn new<T:Component>() -> ComponentList
    {
        ComponentList
        {
            buffer: Buffer::new(mem::size_of::<T>()),
            enabled: BitVec::new(),
            id: TypeId::of::<T>(),
        }
    }

    pub fn clear(&mut self)
    {
        self.buffer.clear();
        self.enabled = BitVec::new();
    }

    pub fn add<T:Component>(&mut self, entity: &Entity, component: &T)
    {
        if TypeId::of::<T>() != self.id
        {
            error("Invalid Component for ComponentList")
        }

        if !self.has(entity)
        {
            unsafe { self.buffer.set(**entity, component); }
            if **entity >= self.enabled.len()
            {
                let diff = **entity - self.enabled.len();
                self.enabled.grow(diff+1, false);
            }
            self.enabled.set(**entity, true);
        }
        else
        {
            error("Cannot add component: Component already exists")
        }
    }

    pub fn set<T:Component>(&mut self, entity: &Entity, component: &T)
    {
        if TypeId::of::<T>() != self.id
        {
            error("Invalid Component for ComponentList")
        }

        if self.has(entity)
        {
            unsafe { self.buffer.set(**entity, component); }
        }
        else
        {
            error("Cannot set component: Component does not exist")
        }
    }

    pub fn has(&self, entity: &Entity) -> bool
    {
        self.enabled.get(**entity).unwrap_or(false)
    }

    pub fn get<T:Component>(&self, entity: &Entity) -> T
    {
        if TypeId::of::<T>() != self.id
        {
            error("Invalid Component for ComponentList")
        }

        if self.has(entity)
        {
            unsafe { self.buffer.get::<T>(**entity) }
        }
        else
        {
            error("Cannot get component: Component does not exist")
        }
    }

    pub fn try_get<T:Component>(&self, entity: &Entity) -> Option<T>
    {
        if TypeId::of::<T>() != self.id
        {
            error("Invalid Component for ComponentList")
        }

        if self.has(entity)
        {
            Some(unsafe { self.buffer.get::<T>(**entity) })
        }
        else
        {
            None
        }
    }

    pub fn borrow<T:Component>(&mut self, entity: &Entity) -> &mut T
    {
        if TypeId::of::<T>() != self.id
        {
            error("Invalid Component for ComponentList")
        }

        if self.has(entity)
        {
            unsafe { self.buffer.borrow::<T>(**entity) }
        }
        else
        {
            error("Cannot get component: Component does not exist")
        }
    }

    pub fn try_borrow<T:Component>(&mut self, entity: &Entity) -> Option<&mut T>
    {
        if TypeId::of::<T>() != self.id
        {
            error("Invalid Component for ComponentList")
        }

        if self.has(entity)
        {
            Some(unsafe { self.buffer.borrow::<T>(**entity) })
        }
        else
        {
            None
        }
    }

    pub fn remove(&mut self, entity: &Entity)
    {
        if self.has(entity)
        {
            self.enabled.set(**entity, false);
        }
    }

    pub fn get_cid(&self) -> ComponentId
    {
        self.id
    }
}
