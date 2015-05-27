
//! Entity identifier and manager types.

use std::collections::hash_map::{HashMap, Values};
use std::default::Default;
use std::marker::PhantomData;
use std::ops::Deref;

use Aspect;
use ComponentManager;
use EntityData;

pub type Id = u64;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature="serialisation", derive(CerealData))]
pub struct Entity(Id);

#[derive(Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature="serialisation", derive(CerealData))]
pub struct IndexedEntity<T: ComponentManager>(usize, Entity, PhantomData<T>);

impl Entity
{
    pub fn nil() -> Entity
    {
        Entity(0)
    }

    /// Returns the entity's unique identifier.
    #[inline]
    pub fn id(&self) -> Id
    {
        self.0
    }
}

impl<T: ComponentManager> IndexedEntity<T>
{
    pub fn index(&self) -> usize
    {
        self.0
    }

    pub unsafe fn clone(&self) -> IndexedEntity<T>
    {
        IndexedEntity(self.0, self.1, self.2)
    }
}

impl<T: ComponentManager> Deref for IndexedEntity<T>
{
    type Target = Entity;
    fn deref(&self) -> &Entity
    {
        &self.1
    }
}

impl Default for Entity
{
    fn default() -> Entity
    {
        Entity::nil()
    }
}

pub struct FilteredEntityIter<'a, T: ComponentManager>
{
    inner: EntityIter<'a, T>,
    aspect: Aspect<T>,
    components: &'a T,
}

// Inner Entity Iterator
pub enum EntityIter<'a, T: ComponentManager>
{
    Map(Values<'a, Entity, IndexedEntity<T>>),
}

impl<'a, T: ComponentManager> EntityIter<'a, T>
{
    pub fn filter(self, aspect: Aspect<T>, components: &'a T) -> FilteredEntityIter<'a, T>
    {
        FilteredEntityIter
        {
            inner: self,
            aspect: aspect,
            components: components,
        }
    }
}

impl<'a, T: ComponentManager> Iterator for EntityIter<'a, T>
{
    type Item = EntityData<'a, T>;
    fn next(&mut self) -> Option<EntityData<'a, T>>
    {
        match *self
        {
            EntityIter::Map(ref mut values) => values.next().map(|x| EntityData(x))
        }
    }
}

impl<'a, T: ComponentManager> Iterator for FilteredEntityIter<'a, T>
{
    type Item = EntityData<'a, T>;
    fn next(&mut self) -> Option<EntityData<'a, T>>
    {
        for x in self.inner.by_ref()
        {
            if self.aspect.check(&x, self.components)
            {
                return Some(x);
            }
            else
            {
                continue
            }
        }
        None
    }
}

/// Handles creation, activation, and validating of entities.
#[doc(hidden)]
#[cfg_attr(feature="serialisation", derive(CerealData))]
pub struct EntityManager<T: ComponentManager>
{
    indices: IndexPool,
    entities: HashMap<Entity, IndexedEntity<T>>,
    next_id: Id,
}

impl<T: ComponentManager> EntityManager<T>
{
    /// Returns a new `EntityManager`
    pub fn new() -> EntityManager<T>
    {
        EntityManager
        {
            indices: IndexPool::new(),
            entities: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn iter(&self) -> EntityIter<T>
    {
        EntityIter::Map(self.entities.values())
    }

    pub fn count(&self) -> usize
    {
        self.indices.count()
    }

    pub fn indexed(&self, entity: &Entity) -> &IndexedEntity<T>
    {
        &self.entities[entity]
    }

    /// Creates a new `Entity`, assigning it the first available index.
    pub fn create(&mut self) -> Entity
    {
        self.next_id += 1;
        let ret = Entity(self.next_id);
        self.entities.insert(ret, IndexedEntity(self.indices.get_index(), ret, PhantomData));
        ret
    }

    /// Returns true if an entity is valid (not removed from the manager).
    #[inline]
    pub fn is_valid(&self, entity: &Entity) -> bool
    {
        self.entities.contains_key(entity)
    }

    /// Deletes an entity from the manager.
    pub fn remove(&mut self, entity: &Entity)
    {
        self.entities.remove(entity).map(|e| self.indices.return_id(e.index()));
    }
}


#[cfg_attr(feature="serialisation", derive(CerealData))]
struct IndexPool
{
    recycled: Vec<usize>,
    next_index: usize,
}

impl IndexPool
{
    pub fn new() -> IndexPool
    {
        IndexPool
        {
            recycled: Vec::new(),
            next_index: 0,
        }
    }

    pub fn count(&self) -> usize
    {
        self.next_index - self.recycled.len()
    }

    pub fn get_index(&mut self) -> usize
    {
        match self.recycled.pop()
        {
            Some(id) => id,
            None => {
                self.next_index += 1;
                self.next_index - 1
            }
        }
    }

    pub fn return_id(&mut self, id: usize)
    {
        self.recycled.push(id);
    }
}
