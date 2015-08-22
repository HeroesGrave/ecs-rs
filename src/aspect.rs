
use {ComponentManager, EntityData};

pub struct Aspect<T: ComponentManager>(Box<Fn(&EntityData<T>, &T) -> bool + 'static>);

impl<T: ComponentManager> Aspect<T>
{
    pub fn all() -> Aspect<T>
    {
        Aspect(Box::new(|_, _| true))
    }

    pub fn none() -> Aspect<T>
    {
        Aspect(Box::new(|_, _| false))
    }

    #[doc(hidden)]
    pub fn __new(inner: Box<Fn(&EntityData<T>, &T) -> bool + 'static>) -> Aspect<T>
    {
        Aspect(inner)
    }

    pub fn check<'a>(&self, entity: &EntityData<'a, T>, components: &T) -> bool
    {
        (self.0)(entity, components)
    }
}
