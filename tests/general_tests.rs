
#[macro_use]
extern crate ecs;

use ecs::{ModifyData};
use ecs::{World};
use ecs::{Process, System};
use ecs::system::{EntityProcess, EntitySystem};
use ecs::EntityIter;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position
{
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Team(u8);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SomeFeature;

components! {
    #[builder(EntityInit)]
    struct TestComponents {
        #[hot] blank_data: (),
        #[hot] position: Position,
        #[cold] team: Team,
        #[hot] feature: SomeFeature,
    }
}

systems! {
    struct TestSystems<TestComponents, ()> {
        active: {
            hello_world: HelloWorld = HelloWorld("Hello, World!"),
        },
        passive: {
            print_position: EntitySystem<PrintPosition> = EntitySystem::new(PrintPosition,
                                                                            aspect!(<TestComponents>
                                                                                    all: [position, feature]
                                                                            )
            ),
        }
    }
}

pub type DataHelper = ecs::DataHelper<TestComponents, ()>;

pub struct HelloWorld(&'static str);
impl Process for HelloWorld
{
    fn process(&mut self, _: &mut DataHelper)
    {
        println!("{}", self.0);
    }
}
impl System for HelloWorld { type Components = TestComponents; type Services = (); }

pub struct PrintPosition;
impl EntityProcess for PrintPosition
{
    fn process(&mut self, en: EntityIter<TestComponents>, co: &mut DataHelper)
    {
        for e in en
        {
            println!("{:?}", co.position.borrow(&e));
        }
    }
}
impl System for PrintPosition {
    type Components = TestComponents;
    type Services = ();
}

#[test]
fn test_general_1()
{
    let mut world = World::<TestSystems>::new();

    // Test entity builders
    let entity = world.create_entity(EntityInit {
        position: Some(Position { x: 0.5, y: 0.7 }),
        team: Some(Team(4)),
        ..Default::default()
    });
    world.create_entity(EntityInit {
        position: Some(Position { x: 0.6, y: 0.8 }),
        team: Some(Team(3)),
        feature: Some(SomeFeature),
        ..Default::default()
    });

    // Finish adding new entities
    world.flush_queue();

    // Test passive systems
    process!(world, print_position);

    // Test entity modifiers
    world.modify_entity(entity, |e: ModifyData<TestComponents>, c: &mut TestComponents| {
        assert_eq!(Some(Position { x: 0.5, y: 0.7 }), c.position.insert(&e, Position { x: -2.5, y: 7.6 }));
        assert_eq!(Some(Team(4)), c.team.remove(&e));
        assert!(!c.feature.has(&e));
        assert!(c.feature.insert(&e, SomeFeature).is_none());
    });

    process!(world, print_position);

    world.modify_entity(entity, |e: ModifyData<TestComponents>, c: &mut TestComponents| {
        assert_eq!(Position { x: -2.5, y: 7.6 }, c.position[e]);
        assert_eq!(None, c.team.remove(&e));
        assert!(c.feature.insert(&e, SomeFeature).is_some());
    });

    // world.modify_entity(entity, EntityChange {
    //     position: Some(Some(Position { x: 0.5, y: 0.7 })), // Set
    //     team: Some(None), // Remove
    //     ..Default::default() // Ignore the rest (None)
    // });

    process!(world, print_position);

    // Test external entity iterator
    for e in world.entities()
    {
        assert!(world.position.has(&e));
    }

    // Test external entity iterator with aspect filtering
    for e in world.entities().filter(aspect!(<TestComponents> all: [team]), &world)
    {
        assert!(world.team.has(&e));
    }

    // Test active systems
    world.update();

    // Test system modification
    world.systems.hello_world.0 = "Goodbye, World!";
    world.update();
}
