
//! Systems to specifically deal with entities.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use Aspect;
use DataHelper;
use {Entity, IndexedEntity};
use EntityData;
use EntityIter;
use {System, Process};

pub trait EntityProcess: System
{
    fn process<'a>(&mut self, EntityIter<'a, Self::Components>, &mut DataHelper<Self::Components, Self::Services>);
}

pub struct EntitySystem<T: EntityProcess>
{
    pub inner: T,
    interested: HashMap<Entity, IndexedEntity<T::Components>>,
    aspect: Aspect<T::Components>,
}

impl<T: EntityProcess> EntitySystem<T>
{
    pub fn new(inner: T, aspect: Aspect<T::Components>) -> EntitySystem<T>
    {
        EntitySystem
        {
            interested: HashMap::new(),
            aspect: aspect,
            inner: inner,
        }
    }
}

impl<T: EntityProcess> Deref for EntitySystem<T>
{
    type Target = T;
    fn deref(&self) -> &T
    {
        &self.inner
    }
}

impl<T: EntityProcess> DerefMut for EntitySystem<T>
{
    fn deref_mut(&mut self) -> &mut T
    {
        &mut self.inner
    }
}

impl<T: EntityProcess> System for EntitySystem<T>
{
    type Components = T::Components;
    type Services = T::Services;
    fn activated(&mut self, entity: &EntityData<T::Components>, components: &T::Components, services: &mut T::Services)
    {
        if self.aspect.check(entity, components)
        {
            self.interested.insert(***entity, (**entity).__clone());
            self.inner.activated(entity, components, services);
        }
    }

    fn reactivated(&mut self, entity: &EntityData<T::Components>, components: &T::Components, services: &mut T::Services)
    {
        if self.interested.contains_key(entity)
        {
            if self.aspect.check(entity, components)
            {
                self.inner.reactivated(entity, components, services);
            }
            else
            {
                self.interested.remove(entity);
                self.inner.deactivated(entity, components, services);
            }
        }
        else if self.aspect.check(entity, components)
        {
            self.interested.insert(***entity, (**entity).__clone());
            self.inner.activated(entity, components, services);
        }
    }

    fn deactivated(&mut self, entity: &EntityData<T::Components>, components: &T::Components, services: &mut T::Services)
    {
        if self.interested.remove(entity).is_some()
        {
            self.inner.deactivated(entity, components, services);
        }
    }

    fn is_active(&self) -> bool
    {
        self.inner.is_active()
    }
}

impl<T: EntityProcess> Process for EntitySystem<T>
{
    fn process(&mut self, c: &mut DataHelper<T::Components, T::Services>)
    {
        self.inner.process(EntityIter::Map(self.interested.values()), c);
    }
}
