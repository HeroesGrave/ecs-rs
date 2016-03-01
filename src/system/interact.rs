
//! System to specifically deal with interactions between two types of entity.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use Aspect;
use DataHelper;
use {Entity, IndexedEntity};
use EntityData;
use EntityIter;
use {Process, System};

pub trait InteractProcess: System
{
    fn process<'a>(&mut self, EntityIter<'a, Self::Components>, EntityIter<'a, Self::Components>, &mut DataHelper<Self::Components, Self::Services>);
}

pub struct InteractSystem<T: InteractProcess>
{
    pub inner: T,
    interested_a: HashMap<Entity, IndexedEntity<T::Components>>,
    interested_b: HashMap<Entity, IndexedEntity<T::Components>>,
    aspect_a: Aspect<T::Components>,
    aspect_b: Aspect<T::Components>,
}

impl<T: InteractProcess> Deref for InteractSystem<T>
{
    type Target = T;
    fn deref(&self) -> &T
    {
        &self.inner
    }
}

impl<T: InteractProcess> DerefMut for InteractSystem<T>
{
    fn deref_mut(&mut self) -> &mut T
    {
        &mut self.inner
    }
}

impl<T: InteractProcess> InteractSystem<T>
{
    pub fn new(inner: T, aspect_a: Aspect<T::Components>, aspect_b: Aspect<T::Components>) -> InteractSystem<T>
    {
        InteractSystem
        {
            interested_a: HashMap::new(),
            interested_b: HashMap::new(),
            aspect_a: aspect_a,
            aspect_b: aspect_b,
            inner: inner,
        }
    }
}

impl<T: InteractProcess> System for InteractSystem<T>
{
    type Components = T::Components;
    type Services = T::Services;
    fn activated(&mut self, entity: &EntityData<T::Components>, components: &T::Components, services: &mut T::Services)
    {
        if self.aspect_a.check(entity, components)
        {
            self.interested_a.insert(***entity, (**entity).__clone());
            self.inner.activated(entity, components, services);
        }
        if self.aspect_b.check(entity, components)
        {
            self.interested_b.insert(***entity, (**entity).__clone());
            self.inner.activated(entity, components, services);
        }
    }

    fn reactivated(&mut self, entity: &EntityData<T::Components>, components: &T::Components, services: &mut T::Services)
    {
        if self.interested_a.contains_key(entity)
        {
            if self.aspect_a.check(entity, components)
            {
                self.inner.reactivated(entity, components, services);
            }
            else
            {
                self.interested_a.remove(entity);
                self.inner.deactivated(entity, components, services);
            }
        }
        else if self.aspect_a.check(entity, components)
        {
            self.interested_a.insert(***entity, (**entity).__clone());
            self.inner.activated(entity, components, services);
        }
        if self.interested_b.contains_key(entity)
        {
            if self.aspect_b.check(entity, components)
            {
                self.inner.reactivated(entity, components, services);
            }
            else
            {
                self.interested_b.remove(entity);
                self.inner.deactivated(entity, components, services);
            }
        }
        else if self.aspect_b.check(entity, components)
        {
            self.interested_b.insert(***entity, (**entity).__clone());
            self.inner.activated(entity, components, services);
        }
    }

    fn deactivated(&mut self, entity: &EntityData<T::Components>, components: &T::Components, services: &mut T::Services)
    {
        if self.interested_a.remove(entity).is_some()
        {
            self.inner.deactivated(entity, components, services);
        }
        if self.interested_b.remove(entity).is_some()
        {
            self.inner.deactivated(entity, components, services);
        }
    }
}

impl<T: InteractProcess> Process for InteractSystem<T>
{
    fn process(&mut self, c: &mut DataHelper<T::Components, T::Services>)
    {
        self.inner.process(EntityIter::Map(self.interested_a.values()), EntityIter::Map(self.interested_b.values()), c);
    }
}
