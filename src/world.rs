
#[cfg(feature="serialisation")] use cereal::{CerealData, CerealError, CerealResult};
#[cfg(feature="serialisation")] use std::io::{Read, Write};

use std::ops::{Deref, DerefMut};

use {BuildData, EntityData, ModifyData};
use {Entity, IndexedEntity, EntityIter};
use {EntityBuilder, EntityModifier};
use entity::EntityManager;

enum Event
{
    BuildEntity(Entity),
    RemoveEntity(Entity),
}

pub struct World<S> where S: SystemManager
{
    pub systems: S,
    pub data: DataHelper<S::Components, S::Services>,
}

pub struct DataHelper<C, M> where C: ComponentManager, M: ServiceManager
{
    pub components: C,
    pub services: M,
    entities: EntityManager<C>,
    event_queue: Vec<Event>,
}

pub trait ComponentManager: 'static+Sized
{
    #[doc(hidden)]
    fn __new() -> Self;
    #[doc(hidden)]
    fn __remove_all(&mut self, &IndexedEntity<Self>);
}

pub trait ServiceManager: 'static {}

impl ServiceManager for () {}

pub trait SystemManager
{
    type Components: ComponentManager;
    type Services: ServiceManager;
    #[doc(hidden)]
    fn __new() -> Self;
    #[doc(hidden)]
    fn __activated(&mut self, EntityData<Self::Components>, &Self::Components, &mut Self::Services);
    #[doc(hidden)]
    fn __reactivated(&mut self, EntityData<Self::Components>, &Self::Components, &mut Self::Services);
    #[doc(hidden)]
    fn __deactivated(&mut self, EntityData<Self::Components>, &Self::Components, &mut Self::Services);
    #[doc(hidden)]
    fn __update(&mut self, &mut DataHelper<Self::Components, Self::Services>);
}

impl<S: SystemManager> Deref for World<S>
{
    type Target = DataHelper<S::Components, S::Services>;
    fn deref(&self) -> &DataHelper<S::Components, S::Services>
    {
        &self.data
    }
}

impl<S: SystemManager> DerefMut for World<S>
{
    fn deref_mut(&mut self) -> &mut DataHelper<S::Components, S::Services>
    {
        &mut self.data
    }
}

impl<C: ComponentManager, M: ServiceManager> Deref for DataHelper<C, M>
{
    type Target = C;
    fn deref(&self) -> &C
    {
        &self.components
    }
}

impl<C: ComponentManager, M: ServiceManager> DerefMut for DataHelper<C, M>
{
    fn deref_mut(&mut self) -> &mut C
    {
        &mut self.components
    }
}

impl<C: ComponentManager, M: ServiceManager> DataHelper<C, M>
{
    pub fn with_entity_data<F, R>(&mut self, entity: &Entity, mut call: F) -> Option<R>
        where F: FnMut(EntityData<C>, &mut C) -> R
    {
        if self.entities.is_valid(entity) {
            Some(call(EntityData(&self.entities.indexed(&entity).__clone()), self))
        } else {
            None
        }
    }

    pub fn create_entity<B>(&mut self, builder: B) -> Entity where B: EntityBuilder<C>
    {
        let entity = self.entities.create();
        builder.build(BuildData(self.entities.indexed(&entity)), &mut self.components);
        self.event_queue.push(Event::BuildEntity(entity));
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity)
    {
        self.event_queue.push(Event::RemoveEntity(entity));
    }
}

#[cfg(feature="serialisation")]
impl<C: ComponentManager, M: ServiceManager> CerealData for DataHelper<C, M> where C: CerealData, M: CerealData {
    fn write(&self, w: &mut Write) -> CerealResult<()> {
        if self.event_queue.len() != 0 {
            Err(CerealError::Msg("Please flush events before serialising the world".to_string()))
        } else {
            try!(self.services.write(w));
            try!(self.entities.write(w));
            self.components.write(w)
        }
    }

    fn read(r: &mut Read) -> CerealResult<Self> {
        let services = try!(CerealData::read(r));
        let entities = try!(CerealData::read(r));
        let components = try!(CerealData::read(r));
        Ok(DataHelper {
            components: components,
            services: services,
            entities: entities,
            event_queue: Vec::new(),
        })
    }
}

#[cfg(feature="serialisation")]
impl<S: SystemManager> World<S> where DataHelper<S::Components, S::Services>: CerealData {
    pub fn load(reader: &mut Read) -> CerealResult<World<S>> {
        let mut world = World {
            systems: S::__new(),
            data: try!(CerealData::read(reader)),
        };
        world.refresh();
        Ok(world)
    }

    pub fn save(&mut self, writer: &mut Write) -> CerealResult<()> {
        self.flush_queue();
        self.data.write(writer)
    }
}

impl<S: SystemManager> World<S>
{
    pub fn new() -> World<S> where S::Services: Default
    {
        World {
            systems: S::__new(),
            data: DataHelper {
                components: S::Components::__new(),
                services: S::Services::default(),
                entities: EntityManager::new(),
                event_queue: Vec::new(),
            },
        }
    }

    pub fn with_services(services: S::Services) -> World<S>
    {
        World {
            systems: S::__new(),
            data: DataHelper {
                components: S::Components::__new(),
                services: services,
                entities: EntityManager::new(),
                event_queue: Vec::new(),
            },
        }
    }

    pub fn entities(&self) -> EntityIter<S::Components>
    {
        self.data.entities.iter()
    }

    pub fn modify_entity<M>(&mut self, entity: Entity, modifier: M) where M: EntityModifier<S::Components>
    {
        let indexed = self.data.entities.indexed(&entity);
        modifier.modify(ModifyData(indexed), &mut self.data.components);
        self.systems.__reactivated(
            EntityData(indexed), &self.data.components, &mut self.data.services
        );
    }

    pub fn refresh(&mut self)
    {
        self.flush_queue();
        for entity in self.data.entities.iter() {
            self.systems.__reactivated(entity, &self.data.components, &mut self.data.services);
        }
    }

    #[cfg(feature="nightly")]
    pub fn flush_queue(&mut self)
    {
        for e in self.data.event_queue.drain(..) {
            match e {
                Event::BuildEntity(entity) => self.systems.__activated(
                    EntityData(self.data.entities.indexed(&entity)),
                    &self.data.components,
                    &mut self.data.services
                ),
                Event::RemoveEntity(entity) => {
                    {
                        let indexed = self.data.entities.indexed(&entity);
                        self.systems.__deactivated(
                            EntityData(indexed), &self.data.components, &mut self.data.services
                        );
                        self.data.components.__remove_all(indexed);
                    }
                    self.data.entities.remove(&entity);
                }
            }
        }
    }

    #[cfg(not(feature="nightly"))]
    pub fn flush_queue(&mut self)
    {
        let mut events = Vec::new();
        ::std::mem::swap(&mut self.event_queue, &mut events);
        for e in events {
            match e {
                Event::BuildEntity(entity) => self.systems.__activated(
                    EntityData(self.data.entities.indexed(&entity)),
                    &self.data.components,
                    &mut self.data.services
                ),
                Event::RemoveEntity(entity) => {
                    {
                        let indexed = self.data.entities.indexed(&entity);
                        self.systems.__deactivated(
                            EntityData(indexed), &self.data.components, &mut self.data.services
                        );
                        self.data.components.__remove_all(indexed);
                    }
                    self.data.entities.remove(&entity);
                }
            }
        }
    }

    pub fn update(&mut self)
    {
        self.flush_queue();
        self.systems.__update(&mut self.data);
        self.flush_queue();
    }
}
