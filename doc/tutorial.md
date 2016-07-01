ecs-rs Tutorial
===============
**_This tutorial is a WIP and will probably change a bit as Rust and/or ecs-rs go through more changes.  
I'll do my best to keep it up to date but if something doesn't work, please report it on the github issue tracker._**

## 1. Introduction
This tutorial is aimed at people who already have a general idea of what an entity component system is and how it should be used.

There are a few other links in the [README](https://github.com/HeroesGrave/ecs-rs/blob/refactor/README.md), but this one should cover lots of what you need to know:
- http://gameprogrammingpatterns.com/component.html

However, there is a key difference from the design explained there. ecs-rs has taken a bit of influence from the [Artemis](http://gamadu.com/artemis/) framework. Here's a quote from that page that summarises it nicely:
> The framework is based on the concept that entities in a game world exist as pure identifiers, their components contain only data, and systems process entities based on their aspects. This promotes separation of concern and simplifies game design tremendously.

First of all, let's look at all these terms:

### Entities
An entity is a 'thing' that interacts with the game world and/or other entities.

In more commonly used Object-Oriented entity systems, the entity stores data and if you need more data or new behaviour you extend the most similar class and add what you need. This is easy to get your head around when you have a simple hierarchy of entity types (eg: Player extends Human extends Biped extends Creature extends Entity), but with more complex systems, you can come across some problems (eg: FlyingEntity and SwimmingEntity extend Creature, but what if you want a flying and swimming entity?)

In this entity-component system, an entity is nothing more than an index to a collection of components. No data, no behaviour.

### Components
A component is a small piece of raw data. (eg: Position, Velocity, Colour, Speed, Ammo, Damage, any sort of data you can think of). Defining behaviour on a component is best avoided (unless you're absolutely certain you know what you're doing), but helper methods are fine (eg: convert the stored spritesheet index to a set of texture-coords, normalising a velocity component).

### Systems
A system is where we define behaviour and manipulate components. Because components don't define behaviour themselves (because they may need information from another component), it instead has to be done externally, which fortunately provides us with more flexibility. A system has full power to manipulate the components of any available entities, but it should go without saying that you should practice common sense (eg: a physics system should not touch a Colour component)

### Aspects
Aspects are a clever way of allowing a system to operate on all the entities it wants to and only the entities it wants to. An aspect filters through entities based on what components they contain, and pulls out the ones that a system is interested in for processing.

They aren't really a key part of an entity-component system, more of a sub-feature of systems.

### So what does this somewhat complex design mean?
It means that an entity can be defined purely by its components, and all relevant behaviour will be automatically applied without any additional code required. If a behaviour is defined once, it will be used for all applicable entities.

It encourages separation of each piece of logic, allowing the programmer to concentrate on a specific system when trying to modify or debug behaviour, and usually allowing new mechanics to have no unintended effects on existing behaviours.

## 2. Creating The World
In ecs-rs, we use a `World` object to manage all entities, components, and systems. Of course, the `World` has no definition of your component types or systems, and so these are handled through generics and associated types.
```rust
World<S> where S: SystemManager<S>
```
`SystemManager` in turn, has the associated types `Components: ComponentManager` and `Services: ServiceManager`.

`ServiceManager` has a default implementation for `()`, but we'll have to specify our own `ComponentManager` and `SystemManager` types. If you look into the source, you'll see that they are `unsafe` traits. There's nothing particularly unsafe about them in the way that `unsafe` is usually used, but if you define the methods wrongly you can get _unexpected_ (but not unsafe/undefined) behaviour.

For the sake of simplicity we'll not include any components or systems. We'll have a proper look at them later. This makes the macro definitions rather simple.
```rust
#[macro_use]
extern crate ecs;

use ecs::World;

components! {
    struct MyComponents;
}

systems! {
    struct MySystems<MyComponents, ()>;
}

fn main() {
    let mut world = World::<MySystems>::new();
}
```
That's all it takes to create a world object. Admittedly, what we have here is a rather useless World, but we'll make it more complex later.

Let's move on to putting entities into our world.

## 3. Adding Entities
Entities are added to the world by using `World.create_entity(EntityBuilder)`. EntityBuilder is implemented for `FnMut(BuildData<T>, &mut T) where T: ComponentManager`, as well as `()` for entities that don't need any data. You can also create custom implementations, but usually closures should be plenty.

(We'll have a closer look at `BuildData` later)

Since we don't have any components available, we'll just use `()`:
```rust
let entity = world.create_entity(());
```
Yay! Now we have an entity. Remember that an entity is just an identifier. Let's have a closer look:
```rust
println!("{:?}", entity);
```
As you can see Entity is made up of two numbers, more specifically a `usize` and a `u64`. The `usize`, or index, is used internally to index components. This value is recycled when an entity is deleted to save memory. However, this means that you could end up with two different entities with identical indices. One of them is a valid entity, and one is not. We solve this by using the `u64`, or identifier (or id because short names are fun).  
This value is unique, as can be seen here:
```rust
world.remove_entity(entity);
let entity2 = world.create_entity(());
assert!(entity != entity2);
println!("{:?}", entity2);
```
(If that `assert!` fails then something has gone horribly wrong internally)

Now that we have entities, we should probably add some components and do something with them.

## 4a. Adding Components to the World
The `Component` trait is automatically implemented for all `'static'` types. All you need to do is create a type and add it to your `ComponentManager`.

Let's try a simple `Position` type:
```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
```
And then we change our `ComponentManager` to this:
```rust
components! {
    struct MyComponents {
        #[hot] position: Position
    }
}
```
You don't need to do anything else to allow usage of the `position` component in the world. All the code for that is generated by the macro. The only thing we need to look at here is the #[hot] 'attribute'.

First of all, it's not actually an attribute. It's just a pattern in the macro. What it does is signal how you want the components to be stored. At the time or writing there are two options: **hot** and **cold**.

- If you use `#[hot]`, the components are stored contiguously (currently `VecMap`) for fast access and cache-friendliness. However, this comes at the cost of taking up memory for every entity, regardless of whether the entity uses the component or not.
- If you use `#[cold]` the components are stored more efficiently in a map (currently `HashMap`). While the storage is not slow, it will take up more CPU time than if the component was marked `#[hot]`.

Generally, you should use `#[cold]` by default, and `#[hot]` for the most important components that are accessed a lot and used by all, if not most entities. Because the position of an entity is commonly required and is used a lot by performance-critical parts of a game as well as most other minor systems, `#[hot]` is probably the best option.

For the sake of demonstration, let's add another `Position` component that holds the respawn location of an entity.
```rust
components! {
    struct MyComponents {
        #[hot] position: Position,
        #[cold] respawn: Position
    }
}
```
Because the respawn data is used very rarely (only when an entity respawns), it's better stored as `#[cold]`.

## 4b. Adding Components to an Entity
To add components to entities we're going to have to use a proper `EntityBuilder`.
```rust
let entity = world.create_entity(
    |entity: BuildData<MyComponents>, data: &mut MyComponents| {
        data.position.add(&entity, Position { x: 0.0, y: 0.0 });
        data.respawn.add(&entity, Position { x: 0.0, y: 0.0 });
    }
);
```

## 4c. Modifying an Entity's Components
This term can mean two things. Modifying the components that an entity has, or adding new components and removing existing ones. We'll start off with the former:

### Modifying existing components
To modify an entity's components from outside the world object, you need to use a closure. You request access to the data of an entity using its identifier (the `Entity` struct), and if the entity is valid the closure is passed an `EntityData` struct which can be used to modify components in a similar fashion to `BuildData` in the previous section.
```rust
world.with_entity_data(&entity, |entity, data| {
    data.position[entity].x += 5.0;
    data.position[entity].y += 8.0;
});
```

### Changing components
To modify an entity's 'aspect' (it's set of active components), you have to use an `EntityModifier`, which is practically the same as an `EntityBuilder`, except you can modify existing data as well as add new components.
```rust
world.modify_entity(entity,
    |entity: ModifyData<MyComponents>, data: &mut MyComponents| {
        data.respawn[entity].x -= 4.0;
        data.position[entity] = data.respawn[entity];
        data.respawn.remove(&entity);
        assert_eq!(data.respawn.get(&entity), None);
        data.respawn.insert(&entity, Position { x: 1.0, y: 2.0});
    }
);
```

Now that we have entities and components, it's time to look at systems.

## 5. Processing the World-state (Systems)
Although lots of behaviour can be implemented with the various component modification functions shown so far, it's better to use `System`s which have a few restrictions that remove the need for some checks (checks which stop you doing silly things).

### Basic system: Printing a message

First off, let's define the type:
```rust
pub struct PrintMessage(pub String);
```
No explanation really needed here.

Next, we need to implement the `System` trait. It has a few functions, but they all have defaults, so you just need to define the `Components` and `Services` associated types:
```rust
impl System for PrintMessage { type Components = MyComponents; type Services = (); }
```
Finally, we implement the `Process` trait. The reason this isn't part of the `System` trait is because special helper systems need to pass in extra data to the `process` function, but still need all the behaviour from the `System` implementation.
```rust
impl Process for PrintMessage {
    fn process(&mut self, _: &mut DataHelper<MyComponents, ()>) {
        println!("{}", &self.0);
    }
}
```

### Adding the system to the world
To add the system, we modify our call to the `systems!` macro to look like this:
```rust
systems! {
    struct MySystems<MyComponents, ()> {
        active: {
            print_msg: PrintMessage = PrintMessage("Hello World".to_string()),
        },
        passive: {}
    }
}
```
The definition is basically `name: type = expr`. Here we are creating a field `print_msg` of the type `PrintMessage`, and setting it to be initialised to `PrintMessage("Hello World".to_string())`.

For the message to be printed out, we need to process a cycle on the world.
```rust
world.update(); // Should print out "Hello World"
```

### Accessing and modifying systems
Systems can be accessed and modified through the `World.systems` field.

For example, we could change the message printed:
```rust
world.systems.print_msg.0 = "Goodbye World".to_string();
world.update(); // Should print out "Goodbye World"
```

### Making a system passive
If we want to manually tell a system when to process, and not when `world.update()` is called, we can make it passive by overriding the `is_active` method.
```rust
impl System for PrintMessage {
    type Components = MyComponents;
    type Services = ();
    fn is_active(&self) -> bool { false }
}
```
Now when we call `world.update()`, nothing will be printed out.

Manually calling process is a little bit awkward:
```rust
world.systems.print_msg.process(&mut world.data); // Should print out "Goodbye World"
```
But I added a macro to make it nicer:
```rust
process!(world, print_msg); // Should print out "Goodbye World"
```
The latter is expanded into the former. Additionally, you can manually call process on active systems, but I don't know why you'd do that.

## 6. EntitySystems and Aspects
Most of the time, your systems should be processing entities. To make this easier there is an `EntitySystem` wrapper type, that sorts out entities based on their components and passes them in to a special type of process (called `EntityProcess`).

To demonstrate this, we'll create a simple motion system by applying a velocity to the entity's position.

First off, we create a `Velocity` component:
```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}
```
And then add it to the component definition: (removing respawn because it's irrelevant)
```rust
components! {
    struct MyComponents {
        #[hot] position: Position,
        #[hot] velocity: Velocity
    }
}
```
Next up we create a struct for our process,
```rust
pub struct MotionProcess;
```
Implement `System`,
```rust
impl System for MotionProcess { type Components = MyComponents; type Services = (); }
```
And now we implement `EntityProcess` instead of `Process`:
```rust
impl EntityProcess for MotionProcess {
    fn process(&mut self, entities: EntityIter<MyComponents>, data: &mut DataHelper<MyComponents, ()>) {

    }
}
```
The only difference to `Process` is that we are passed in an `EntityIter`. An `EntityIter` yields `EntityData` structs (which were described in part 4c) which can be used to modify data from the `DataHelper`. This works pretty much like the `World.with_entity_data()` method, but with less overhead.

If we were trying to write less code, we'd probably want to implement `Add<Velocity>` for `Position`, but for the sake of simplicity we'll just add them together manually.
```rust
fn process(&mut self, entities: EntityIter<MyComponents>, data: &mut DataHelper<MyComponents, ()>) {
    for e in entities {
        let mut position = data.position[e];
        let velocity = data.velocity[e];
        position.x += velocity.dx;
        position.y += velocity.dy;
        data.position[e] = position;
    }
}
```
Next, we have to add this to the systems definition:
```rust
systems! {
    struct MySystems<MyComponents, ()> {
        active: {
            motion: EntitySystem<MotionProcess> = EntitySystem::new(
                MotionProcess,
                aspect!(<MyComponents> all: [position, velocity])
            )
        }
        passive: {}
    }
}
```
This might require a bit of explaining.

The `EntitySystem` type is generic over the trait `EntityProcess`, and to create it, you need to pass an `EntityProcess` and an `Aspect`.

`EntityProcess` has already been explained.
`Aspect`s, as mentioned earlier, are filters used to separate out the entities that have the components to fulfill certain requirements.

### Aspects
Aspects are usually defined by the `aspect!` macro:
```rust
aspect!(<MyComponents> all: [position, velocity])
```
The first section is the type defined by the `components!` macro. After that, you can have an `all` section and/or a `none` section. To be accepted by the aspect, an entity must have all the components listed under "all", and none of the components listed under "none".

For example, if we had another component that disabled an entity from moving, we'd define the aspect like this:
```rust
aspect!(<MyComponents> all: [position, velocity] none: [disable_movement])
```
And if we wanted an aspect that applied only to motionless entities, we'd define it like this:
```rust
aspect!(<MyComponents> none: [velocity])
```
If we wanted to accept all entities, we can just use a plain function on `Aspect`:
```rust
Aspect::all()
```
And the same for an aspect that rejects all entities (not sure what this would be used for, but the functionality is there)
```rust
Aspect::none()
```

More complicated functionality for aspects may be available in the future, but for now, this should be enough for most use cases.

### Testing the system
Just to check the systems works, let's create an entity:
```rust
let entity = world.create_entity(
    |entity: BuildData<MyComponents>, data: &mut MyComponents| {
        data.position.add(&entity, Position { x: 0.0, y: 0.0 });
        data.velocity.add(&entity, Velocity { dx: 1.0, dy: 0.0 });
    }
);
```
And call `world.update()`
```rust
world.update();
```
If we check the value of the entity's position we should see it has moved 1 unit in the positive-x direction.
```rust
world.with_entity_data(&entity, |en, data|
    assert_eq!(Position { x: 1.0, y: 0.0 }, data.position[en])
);
```

## More coming soon
That's more or less the basics of using **ecs-rs**. There are a few more advanced features available that I haven't got into yet, and also some advice on common patterns that work well. There's also a few more features that may be added to the library (custom managers, for things like sorting teams, players, etc.).

If you need any help, the most reliable way is the general questions 'issue' on github. See [here](https://github.com/HeroesGrave/ecs-rs/issues/13). I also sometimes hang around on the #rust-gamedev IRC channel. Ping me (username is HeroesGrave) with your question.
