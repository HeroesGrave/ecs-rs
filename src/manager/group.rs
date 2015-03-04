
use std::borrow::Borrow;
use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::ops::{Index, IndexMut};

use Entity;
use Manager;
use World;

pub trait GroupKey: Hash+Eq+'static {}
impl<T: Hash+Eq+'static> GroupKey for T {}

pub struct GroupManager<Key: GroupKey>
{
    groups: HashMap<Key, Vec<Entity>>,
}

impl<Key: GroupKey> GroupManager<Key>
{
    pub fn new() -> GroupManager<Key>
    {
        GroupManager
        {
            groups: HashMap::new(),
        }
    }

    pub fn create(&mut self, key: Key)
    {
        match self.groups.entry(key)
        {
            Entry::Vacant(entry) => {
                entry.insert(Vec::new());
            },
            _ => (),
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&Vec<Entity>>
        where Q: GroupKey+Borrow<Key>
    {
        self.groups.get(key.borrow())
    }

    pub fn delete<Q: ?Sized>(&mut self, key: &Q) -> Option<Vec<Entity>>
        where Q: GroupKey+Borrow<Key>
    {
        self.groups.remove(key.borrow())
    }
}

impl<Key: GroupKey> Index<Key> for GroupManager<Key>
{
    type Output = Vec<Entity>;
    fn index(&self, i: &Key) -> &Vec<Entity>
    {
        &self.groups[*i]
    }
}

impl<Key: GroupKey> IndexMut<Key> for GroupManager<Key>
{
    fn index_mut(&mut self, i: &Key) -> &mut Vec<Entity>
    {
        &mut self.groups[*i]
    }
}

impl<Key: GroupKey> Manager for GroupManager<Key>
{
    fn deactivated(&mut self, entity: &Entity, _: &World)
    {
        for (_, ref mut vec) in self.groups.iter_mut()
        {
            vec.retain(|e| *e != *entity);
        }
    }
}
