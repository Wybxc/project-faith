use std::{
    any::{Any, TypeId},
    collections::hash_map::Entry,
};

use downcast_rs::{DowncastSync, impl_downcast};

use crate::utils::Map;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

impl Entity {
    pub fn id(self) -> u32 {
        self.0
    }
}

impl From<u32> for Entity {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

pub trait Component: Send + Sync + 'static {
    type Storage: Storage<Component = Self> + 'static;
}

#[macro_export]
macro_rules! impl_component {
    ($name: ident) => {
        impl $crate::system::Component for $name {
            type Storage = $crate::utils::Map<$crate::system::Entity, $name>;
        }
    };
}

pub trait StorageBase: DowncastSync {
    fn delete(&mut self, entity: Entity) -> bool;
    fn has(&self, entity: Entity) -> bool;
}
impl_downcast!(StorageBase);

pub trait Storage: StorageBase + Default {
    type Component;

    fn add(&mut self, entity: Entity, component: Self::Component) -> Result<(), Self::Component>;
    fn remove(&mut self, entity: Entity) -> Option<Self::Component>;
    fn get(&self, entity: Entity) -> Option<&Self::Component>;
    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component>;
    fn iter(&self) -> impl Iterator<Item = (Entity, &Self::Component)>;
}

impl<C> StorageBase for Map<Entity, C>
where
    C: Send + Sync + 'static,
{
    fn delete(&mut self, entity: Entity) -> bool {
        self.remove(&entity).is_some()
    }

    fn has(&self, entity: Entity) -> bool {
        self.contains_key(&entity)
    }
}

impl<C> Storage for Map<Entity, C>
where
    C: Component,
{
    type Component = C;

    fn add(&mut self, entity: Entity, component: Self::Component) -> Result<(), Self::Component> {
        if let Entry::Vacant(e) = self.entry(entity) {
            e.insert(component);
            Ok(())
        } else {
            Err(component)
        }
    }

    fn remove(&mut self, entity: Entity) -> Option<Self::Component> {
        self.remove(&entity)
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        self.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component> {
        self.get_mut(&entity)
    }

    fn iter(&self) -> impl Iterator<Item = (Entity, &Self::Component)> {
        std::collections::HashMap::iter(self).map(|(e, c)| (*e, c))
    }
}

#[derive(Default)]
pub struct System {
    storages: Map<TypeId, Box<dyn StorageBase + Send + Sync>>,
    resources: Map<TypeId, Box<dyn Any + Send + Sync>>,
    entity_counter: u32,
}

impl System {
    fn storage<C: Component>(&self) -> Option<&C::Storage> {
        self.storages
            .get(&TypeId::of::<C::Storage>())
            .map(|storage| storage.as_any().downcast_ref::<C::Storage>().unwrap())
    }

    fn storage_mut<C: Component>(&mut self) -> &mut C::Storage {
        self.storages
            .entry(TypeId::of::<C::Storage>())
            .or_insert_with(|| Box::new(C::Storage::default()))
            .as_any_mut()
            .downcast_mut::<C::Storage>()
            .unwrap()
    }

    pub fn resource<R: Any + Send + Sync>(&self) -> Option<&R> {
        self.resources
            .get(&TypeId::of::<R>())
            .and_then(|r| r.downcast_ref::<R>())
    }

    pub fn resource_mut<R: Any + Send + Sync>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .and_then(|r| r.downcast_mut::<R>())
    }

    pub fn resource_or_insert<R: Any + Send + Sync, F: FnOnce() -> R>(
        &mut self,
        default: F,
    ) -> &mut R {
        self.resources
            .entry(TypeId::of::<R>())
            .or_insert_with(|| Box::new(default()))
            .downcast_mut::<R>()
            .unwrap()
    }

    pub fn resource_or_default<R: Any + Default + Send + Sync>(&mut self) -> &mut R {
        self.resource_or_insert(|| R::default())
    }

    pub fn add_resource<R: Any + Send + Sync>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    pub fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R> {
        self.resources
            .remove(&TypeId::of::<R>())
            .and_then(|r| r.downcast::<R>().ok())
            .map(|r| *r)
    }

    pub fn entity(&mut self) -> EntityBuilder {
        let entity = Entity(self.entity_counter);
        self.entity_counter += 1;
        EntityBuilder {
            system: self,
            entity,
        }
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        self.storage_mut::<C>().add(entity, component)
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        self.storage_mut::<C>().remove(entity)
    }

    pub fn despawn(&mut self, entity: Entity) {
        for storage in self.storages.values_mut() {
            storage.delete(entity);
        }
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.storage::<C>()?.get(entity)
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        self.storage_mut::<C>().get_mut(entity)
    }

    pub fn query<'a, Q: Query + 'a>(
        &'a self,
        query: Q,
    ) -> impl Iterator<Item = (Entity, Q::Result<'a>)> + 'a {
        query.execute_query(self)
    }
}

impl Entity {
    pub fn add<C: Component>(self, system: &mut System, component: C) -> Result<(), C> {
        system.add_component(self, component)
    }

    pub fn remove<C: Component>(self, system: &mut System) -> Option<C> {
        system.remove_component::<C>(self)
    }

    pub fn get<C: Component>(self, system: &System) -> Option<&C> {
        system.get_component(self)
    }

    pub fn get_mut<C: Component>(self, system: &mut System) -> Option<&mut C> {
        system.get_component_mut(self)
    }
}

pub struct EntityBuilder<'a> {
    system: &'a mut System,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    pub fn component<C: Component>(self, component: C) -> Self {
        let storage = self.system.storage_mut::<C>();
        if storage.add(self.entity, component).is_err() {
            panic!("Component already exists for this entity");
        }
        self
    }

    pub fn spawn(self) -> Entity {
        self.entity
    }
}

pub trait Query {
    type Result<'s>
    where
        Self: 's;

    fn execute_query<'s>(
        self,
        system: &'s System,
    ) -> impl Iterator<Item = (Entity, Self::Result<'s>)> + 's
    where
        Self: 's;

    fn execute_filter<'s>(&self, system: &'s System, entity: Entity) -> Option<Self::Result<'s>>;

    fn and<Q2: Query>(self, other: Q2) -> And<Self, Q2>
    where
        Self: Sized,
    {
        And(self, other)
    }
}

pub struct Has<C>(std::marker::PhantomData<C>);

pub fn has<C>() -> Has<C> {
    Has(std::marker::PhantomData)
}

impl<C: Component> Query for Has<C> {
    type Result<'s> = &'s C;

    fn execute_query<'s>(
        self,
        system: &'s System,
    ) -> impl Iterator<Item = (Entity, Self::Result<'s>)> + 's
    where
        Self: 's,
    {
        system
            .storage::<C>()
            .map(|storage| storage.iter())
            .into_iter()
            .flatten()
    }

    fn execute_filter<'s>(&self, system: &'s System, entity: Entity) -> Option<Self::Result<'s>> {
        system
            .storage::<C>()
            .and_then(|storage| storage.get(entity))
    }
}

pub struct Exact<C>(C);

pub fn exact<C>(component: C) -> Exact<C> {
    Exact(component)
}

impl<C: Component + PartialEq> Query for Exact<C> {
    type Result<'s> = &'s C;

    fn execute_query<'s>(
        self,
        system: &'s System,
    ) -> impl Iterator<Item = (Entity, Self::Result<'s>)> + 's
    where
        Self: 's,
    {
        system
            .storage::<C>()
            .map(move |storage| storage.iter().filter(move |&(_, c)| *c == self.0))
            .into_iter()
            .flatten()
    }

    fn execute_filter<'s>(&self, system: &'s System, entity: Entity) -> Option<Self::Result<'s>> {
        system
            .storage::<C>()
            .and_then(|storage| storage.get(entity))
            .filter(|&c| *c == self.0)
    }
}

pub struct And<Q1, Q2>(Q1, Q2);

impl<Q1: Query, Q2: Query> Query for And<Q1, Q2> {
    type Result<'s>
        = (Q1::Result<'s>, Q2::Result<'s>)
    where
        Q1: 's,
        Q2: 's;

    fn execute_query<'s>(
        self,
        system: &'s System,
    ) -> impl Iterator<Item = (Entity, Self::Result<'s>)> + 's
    where
        Self: 's,
    {
        self.0.execute_query(system).flat_map(move |(e, r1)| {
            self.1
                .execute_filter(system, e)
                .map(move |r2| (e, (r1, r2)))
        })
    }

    fn execute_filter<'s>(&self, system: &'s System, entity: Entity) -> Option<Self::Result<'s>> {
        let r1 = self.0.execute_filter(system, entity)?;
        let r2 = self.1.execute_filter(system, entity)?;
        Some((r1, r2))
    }
}
