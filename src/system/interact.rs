
//! System to specifically deal with interactions between two types of entity.

use std::collections::HashMap;

use Aspect;
use DataHelper;
use {Entity, IndexedEntity};
use EntityData;
use EntityIter;
use {Process, System};

pub trait InteractProcess: System
{
    fn process<'a>(&self, EntityIter<'a, Self::Components>, EntityIter<'a, Self::Components>, &mut DataHelper<Self::Components, Self::Services>);
}

pub struct InteractSystem<T: InteractProcess>
{
    pub inner: T,
    interested_a: HashMap<Entity, IndexedEntity<T::Components>>,
    interested_b: HashMap<Entity, IndexedEntity<T::Components>>,
    aspect_a: Aspect<T::Components>,
    aspect_b: Aspect<T::Components>,
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
    fn activated(&mut self, entity: &EntityData<T::Components>, world: &T::Components)
    {
        if self.aspect_a.check(entity, world)
        {
            self.interested_a.insert(***entity, unsafe { (**entity).clone() });
            self.inner.activated(entity, world);
        }
        if self.aspect_b.check(entity, world)
        {
            self.interested_b.insert(***entity, unsafe { (**entity).clone() });
            self.inner.activated(entity, world);
        }
    }

    fn reactivated(&mut self, entity: &EntityData<T::Components>, world: &T::Components)
    {
        if self.interested_a.contains_key(entity)
        {
            if self.aspect_a.check(entity, world)
            {
                self.inner.reactivated(entity, world);
            }
            else
            {
                self.interested_a.remove(entity);
                self.inner.deactivated(entity, world);
            }
        }
        else if self.aspect_a.check(entity, world)
        {
            self.interested_a.insert(***entity, unsafe { (**entity).clone() });
            self.inner.activated(entity, world);
        }
        if self.interested_b.contains_key(entity)
        {
            if self.aspect_b.check(entity, world)
            {
                self.inner.reactivated(entity, world);
            }
            else
            {
                self.interested_b.remove(entity);
                self.inner.deactivated(entity, world);
            }
        }
        else if self.aspect_b.check(entity, world)
        {
            self.interested_b.insert(***entity, unsafe { (**entity).clone() });
            self.inner.activated(entity, world);
        }
    }

    fn deactivated(&mut self, entity: &EntityData<T::Components>, world: &T::Components)
    {
        if self.interested_a.remove(entity).is_some()
        {
            self.inner.deactivated(entity, world);
        }
        if self.interested_b.remove(entity).is_some()
        {
            self.inner.deactivated(entity, world);
        }
    }

    fn is_active(&self) -> bool
    {
        self.inner.is_active()
    }
}

impl<T: InteractProcess> Process for InteractSystem<T>
{
    fn process(&mut self, c: &mut DataHelper<T::Components, T::Services>)
    {
        self.inner.process(EntityIter::Map(self.interested_a.values()), EntityIter::Map(self.interested_b.values()), c);
    }
}
