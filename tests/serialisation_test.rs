
#![cfg(feature="serialisation")]

#![feature(plugin)]
#![feature(custom_derive)]

#![plugin(cereal_macros)]

#[macro_use]
extern crate ecs;
extern crate cereal;

use ecs::{EntityIter, System, World};
use ecs::system::{EntityProcess, EntitySystem};
use std::io::Cursor;

#[derive(Copy, Clone, Debug, PartialEq, CerealData)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, CerealData)]
pub struct Team(u8);

#[derive(Copy, Clone, Debug, PartialEq, CerealData)]
pub struct SomeFeature;

components!(
    #[builder(EntityInit)]
    #[modifier(EntityChange)]
    #[derive(CerealData)]
    struct TestComponents {
        #[hot] position: Position,
        #[cold] team: Team,
        #[hot] feature: SomeFeature,
    }
);

systems!(
    struct TestSystems<TestComponents, TestServices> {
        print_position: EntitySystem<PrintPosition> = EntitySystem::new(PrintPosition,
            aspect!(<TestComponents>
                all: [position, feature]
            )
        ),
    }
);

services!(
    #[derive(CerealData)]
    struct TestServices {
        check: u32 = 0,
    }
);

pub type DataHelper = ecs::DataHelper<TestComponents, TestServices>;

pub struct PrintPosition;
impl EntityProcess for PrintPosition
{
    fn process(&mut self, en: EntityIter<TestComponents>, co: &mut DataHelper)
    {
        for e in en
        {
            println!("{:?}: {:?}", **e, co.position.borrow(&e));
        }
    }
}
impl System for PrintPosition {
    type Components = TestComponents;
    type Services = TestServices;
    fn is_active(&self) -> bool { false }
}

#[test]
pub fn test_serialisation() {
    let mut world = World::<TestSystems>::new();

    // Test entity builders
    world.create_entity(EntityInit {
        position: Some(Position { x: 0.5, y: 0.7 }),
        team: Some(Team(4)),
        ..Default::default()
    });
    for i in 0..10 {
        world.create_entity(EntityInit {
            position: Some(Position { x: 0.5, y: 0.8 * (i as f32) }),
            team: Some(Team(i as u8)),
            feature: Some(SomeFeature),
            ..Default::default()
        });
    }

    world.flush_queue();

    process!(world, print_position);

    world.services.check = 2;

    let mut store = Cursor::new(Vec::new());
    world.save(&mut store).unwrap();
    println!("Finished Writing. Bytes written: {:?}", store.position());
    store.set_position(0);

    let mut world_2: World<TestSystems> = World::load(&mut store).unwrap();

    assert_eq!(world.services.check, 2);

    process!(world_2, print_position);
}
