// If these tests fail, make sure to update the corresponding code in doc/tutorial.md

#![forbid(warnings)]

#[macro_use]
extern crate ecs;

pub mod chapter2 {
    use ecs::World;

    components! {
        struct MyComponents;
    }

    systems! {
        struct MySystems<MyComponents, ()>;
    }

    #[test]
    fn test() {
        let _ = World::<MySystems>::new();
    }
}

pub mod chapter3 {
    use ecs::World;

    use chapter2::{MySystems};

    #[test]
    fn test() {
        let mut world = World::<MySystems>::new();

        let entity = world.create_entity(());
        println!("{:?}", entity);
        world.remove_entity(entity);
        let entity2 = world.create_entity(());
        assert!(entity != entity2);
        println!("{:?}", entity2);
    }
}

pub mod chapter4 {
    use ecs::{BuildData, ModifyData};
    use ecs::World;

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Position {
        pub x: f32,
        pub y: f32,
    }

    components! {
        struct MyComponents {
            #[hot] position: Position,
            #[cold] respawn: Position
        }
    }

    systems! {
        struct MySystems<MyComponents, ()>;
    }

    #[test]
    fn test() {
        let mut world = World::<MySystems>::new();

        let entity = world.create_entity(
            |entity: BuildData<MyComponents>, data: &mut MyComponents| {
                data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                data.respawn.add(&entity, Position { x: 0.0, y: 0.0 });
            }
        );

        world.with_entity_data(&entity, |entity, data| {
            data.position[entity].x += 5.0;
            data.position[entity].y += 8.0;
        });

        world.modify_entity(entity,
            |entity: ModifyData<MyComponents>, data: &mut MyComponents| {
                data.respawn[entity].x -= 4.0;
                data.position[entity] = data.respawn[entity];
                data.respawn.remove(&entity);
                assert_eq!(data.respawn.get(&entity), None);
                data.respawn.insert(&entity, Position { x: 1.0, y: 2.0});
            }
        );
    }
}

pub mod chapter5 {
    use ecs::system::{Process, System};
    use ecs::{DataHelper, World};

    use chapter2::MyComponents;

    pub struct PrintMessage(pub String);
    impl System for PrintMessage {
        type Components = MyComponents;
        type Services = ();
    }
    impl Process for PrintMessage {
        fn process(&mut self, _: &mut DataHelper<MyComponents, ()>) {
            println!("{}", &self.0);
        }
    }

    systems! {
        struct MySystems<MyComponents, ()> {
            active: {
            },
            passive: {
                print_msg: PrintMessage = PrintMessage("Hello World".to_string()),
            }
        }
    }

    #[test]
    fn test() {
        let mut world = World::<MySystems>::new();

        world.update(); // Doesn't print anything because we the system is passive.
        world.systems.print_msg.0 = "Goodbye World".to_string();
        world.systems.print_msg.process(&mut world.data); // "Goodbye World"
        process!(world, print_msg); // "Goodbye World"
    }
}

pub mod chapter6 {
    use ecs::system::{EntityProcess, EntitySystem, System};
    use ecs::{DataHelper, EntityIter, World};
    use ecs::{BuildData};

    use chapter4::Position;

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Velocity {
        pub dx: f32,
        pub dy: f32,
    }

    components! {
        struct MyComponents {
            #[hot] position: Position,
            #[hot] velocity: Velocity
        }
    }

    pub struct MotionProcess;
    impl System for MotionProcess { type Components = MyComponents; type Services = (); }
    impl EntityProcess for MotionProcess {
        fn process(&mut self, entities: EntityIter<MyComponents>, data: &mut DataHelper<MyComponents, ()>) {
            for e in entities {
                let mut position = data.position[e];
                let velocity = data.velocity[e];
                position.x += velocity.dx;
                position.y += velocity.dy;
                data.position[e] = position;
            }
        }
    }

    systems! {
        struct MySystems<MyComponents, ()> {
            active: {
                motion: EntitySystem<MotionProcess> = EntitySystem::new(MotionProcess, aspect!(<MyComponents> all: [position, velocity])),
            },
            passive: {
            }
        }
    }

    #[test]
    fn test() {
        let mut world = World::<MySystems>::new();

        let entity = world.create_entity(
            |entity: BuildData<MyComponents>, data: &mut MyComponents| {
                data.position.add(&entity, Position { x: 0.0, y: 0.0 });
                data.velocity.add(&entity, Velocity { dx: 1.0, dy: 0.0 });
            }
        );

        world.create_entity(
            |entity: BuildData<MyComponents>, data: &mut MyComponents| {
                data.position.add(&entity, Position { x: 0.0, y: 0.0 });
            }
        );

        world.with_entity_data(&entity, |en, data|
            assert_eq!(Position { x: 0.0, y: 0.0 }, data.position[en])
        );

        world.update();

        world.with_entity_data(&entity, |en, data|
            assert_eq!(Position { x: 1.0, y: 0.0 }, data.position[en])
        );
    }
}
