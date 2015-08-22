
use std::ops::{Deref, DerefMut};

use DataHelper;
use EntityData;
use {Process, System};

/// System which operates every certain number of updates.
pub struct IntervalSystem<T: Process>
{
    pub inner: T,
    interval: u8,
    ticker: u8,
}

impl<T: Process> IntervalSystem<T>
{
    /// Create a new interval system with the specified number of updates between processes.
    pub fn new(system: T, interval: u8) -> IntervalSystem<T>
    {
        IntervalSystem
        {
            interval: interval,
            ticker: 0,
            inner: system,
        }
    }
}

impl<T: Process> Deref for IntervalSystem<T>
{
    type Target = T;
    fn deref(&self) -> &T
    {
        &self.inner
    }
}

impl<T: Process> DerefMut for IntervalSystem<T>
{
    fn deref_mut(&mut self) -> &mut T
    {
        &mut self.inner
    }
}

impl<T: Process> Process for IntervalSystem<T>
{
    fn process(&mut self, c: &mut DataHelper<T::Components, T::Services>)
    {
        self.ticker += 1;
        if self.ticker == self.interval
        {
            self.ticker = 0;
            self.inner.process(c);
        }
    }
}

impl<T: Process> System for IntervalSystem<T>
{
    type Components = T::Components;
    type Services = T::Services;
    fn activated(&mut self, e: &EntityData<T::Components>, c: &T::Components, s: &mut T::Services)
    {
        self.inner.activated(e, c, s);
    }

    fn reactivated(&mut self, e: &EntityData<T::Components>, c: &T::Components, s: &mut T::Services)
    {
        self.inner.reactivated(e, c, s);
    }

    fn deactivated(&mut self, e: &EntityData<T::Components>, c: &T::Components, s: &mut T::Services)
    {
        self.inner.deactivated(e, c, s);
    }

    fn is_active(&self) -> bool
    {
        self.inner.is_active()
    }
}
