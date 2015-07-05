
use DataHelper;
use EntityData;
use {Process, System};

/// System which operates every certain number of updates.
pub struct LazySystem<T: Process>
{
    pub inner: Option<T>,
}

impl<T: Process> LazySystem<T>
{
    /// Create a new lazy system
    pub fn new() -> LazySystem<T>
    {
        LazySystem {
            inner: None,
        }
    }

    /// Initialise the lazy system.
    ///
    /// Returns whether the system was already initialised.
    pub fn init(&mut self, sys: T) -> bool
    {
        match self.inner {
            Some(_) => true,
            None => {
                self.inner = Some(sys);
                false
            },
        }
    }

    /// Initialise the lazy system, overriding an already existing initialisation
    ///
    /// Returns whether the system was already initialised.
    pub fn init_override(&mut self, sys: T) -> bool
    {
        let ret = self.is_initialised();
        self.inner = Some(sys);
        ret
    }

    #[inline]
    pub fn is_initialised(&self) -> bool
    {
        self.inner.is_some()
    }
}

impl<T: Process> Process for LazySystem<T>
{
    fn process(&mut self, c: &mut DataHelper<T::Components, T::Services>)
    {
        if let Some(ref mut sys) = self.inner {
            sys.process(c);
        }
    }
}

impl<T: Process> System for LazySystem<T>
{
    type Components = T::Components;
    type Services = T::Services;
    fn activated(&mut self, e: &EntityData<T::Components>, w: &T::Components)
    {
        self.inner.as_mut().map(|sys| sys.activated(e, w));
    }

    fn reactivated(&mut self, e: &EntityData<T::Components>, w: &T::Components)
    {
        self.inner.as_mut().map(|sys| sys.reactivated(e, w));
    }

    fn deactivated(&mut self, e: &EntityData<T::Components>, w: &T::Components)
    {
        self.inner.as_mut().map(|sys| sys.deactivated(e, w));
    }

    fn is_active(&self) -> bool
    {
        self.inner.as_ref().map(|sys| sys.is_active()).unwrap_or(false)
    }
}
