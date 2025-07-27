use std::{any::TypeId, collections::hash_map::Entry};

use downcast_rs::{DowncastSync, impl_downcast};

use crate::utils::Map;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

pub trait Component: Send + Sync + 'static {
    type Storage: Storage<Component = Self> + 'static;
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
}

#[derive(Default)]
pub struct System {
    storages: Map<TypeId, Box<dyn StorageBase + Send + Sync>>,
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

    pub fn entity(&mut self) -> EntityBuilder {
        let entity = Entity(self.entity_counter);
        self.entity_counter += 1;
        EntityBuilder {
            system: self,
            entity,
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        for storage in self.storages.values_mut() {
            storage.delete(entity);
        }
    }

    pub fn get<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.storage::<C>()?.get(entity)
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
