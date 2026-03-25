use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

// ── Entity ────────────────────────────────────────────────────────────────────

/// A lightweight handle for a game object. Just an integer under the hood.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity(pub u64);

// ── Component storage ─────────────────────────────────────────────────────────

type Storage = HashMap<Entity, Box<dyn Any>>;

/// The central ECS world. Stores all entities and their components.
pub struct World {
    next_id:    u64,
    alive:      HashSet<Entity>,
    components: HashMap<TypeId, Storage>,
    /// Entities queued for removal at end of frame.
    dead_queue: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id:    1,
            alive:      HashSet::new(),
            components: HashMap::new(),
            dead_queue: Vec::new(),
        }
    }

    // ── Spawning ──────────────────────────────────────────────────────────────

    /// Begin building a new entity.
    pub fn spawn(&mut self) -> EntityBuilder {
        let id = Entity(self.next_id);
        self.next_id += 1;
        self.alive.insert(id);
        EntityBuilder { entity: id, world: self }
    }

    /// Schedule an entity for removal at the end of the current frame.
    pub fn despawn(&mut self, entity: Entity) {
        self.dead_queue.push(entity);
    }

    /// Actually remove all queued-up entities. Call once per frame.
    pub fn flush_dead(&mut self) {
        let dead: Vec<Entity> = self.dead_queue.drain(..).collect();
        for e in dead {
            self.alive.remove(&e);
            for storage in self.components.values_mut() {
                storage.remove(&e);
            }
        }
    }

    #[inline] pub fn is_alive(&self, entity: Entity) -> bool { self.alive.contains(&entity) }
    #[inline] pub fn entity_count(&self) -> usize { self.alive.len() }

    // ── Component CRUD ────────────────────────────────────────────────────────

    pub fn add<T: 'static>(&mut self, entity: Entity, component: T) {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(HashMap::new)
            .insert(entity, Box::new(component));
    }

    pub fn remove<T: 'static>(&mut self, entity: Entity) {
        if let Some(storage) = self.components.get_mut(&TypeId::of::<T>()) {
            storage.remove(&entity);
        }
    }

    pub fn has<T: 'static>(&self, entity: Entity) -> bool {
        self.components
            .get(&TypeId::of::<T>())
            .map_or(false, |s| s.contains_key(&entity))
    }

    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())?
            .get(&entity)?
            .downcast_ref()
    }

    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())?
            .get_mut(&entity)?
            .downcast_mut()
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Iterate all entities that have component `T`.
    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.components
            .get(&TypeId::of::<T>())
            .into_iter()
            .flat_map(|storage| {
                storage.iter().filter_map(|(e, c)| {
                    c.downcast_ref::<T>().map(|t| (*e, t))
                })
            })
    }

    /// Collect entities that have both `A` and `B`.
    pub fn query2<A: 'static, B: 'static>(&self) -> Vec<(Entity, &A, &B)> {
        let sa = match self.components.get(&TypeId::of::<A>()) { Some(s) => s, None => return vec![] };
        let sb = match self.components.get(&TypeId::of::<B>()) { Some(s) => s, None => return vec![] };
        sa.iter()
            .filter_map(|(e, ca)| {
                let a = ca.downcast_ref::<A>()?;
                let b = sb.get(e)?.downcast_ref::<B>()?;
                Some((*e, a, b))
            })
            .collect()
    }

    /// Collect entities that have `A`, `B`, and `C`.
    pub fn query3<A: 'static, B: 'static, C: 'static>(&self) -> Vec<(Entity, &A, &B, &C)> {
        let sa = match self.components.get(&TypeId::of::<A>()) { Some(s) => s, None => return vec![] };
        let sb = match self.components.get(&TypeId::of::<B>()) { Some(s) => s, None => return vec![] };
        let sc = match self.components.get(&TypeId::of::<C>()) { Some(s) => s, None => return vec![] };
        sa.iter()
            .filter_map(|(e, ca)| {
                let a = ca.downcast_ref::<A>()?;
                let b = sb.get(e)?.downcast_ref::<B>()?;
                let c = sc.get(e)?.downcast_ref::<C>()?;
                Some((*e, a, b, c))
            })
            .collect()
    }

    /// Return all entity IDs that have component `T`.
    pub fn entities_with<T: 'static>(&self) -> Vec<Entity> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|s| s.keys().copied().collect())
            .unwrap_or_default()
    }

    /// All live entities.
    pub fn all_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.alive.iter().copied()
    }
}

impl Default for World {
    fn default() -> Self { Self::new() }
}

// ── EntityBuilder ─────────────────────────────────────────────────────────────

/// Fluent builder for attaching components to a newly spawned entity.
pub struct EntityBuilder<'w> {
    pub entity: Entity,
    world: &'w mut World,
}

impl<'w> EntityBuilder<'w> {
    /// Attach a component and continue building.
    pub fn with<T: 'static>(self, component: T) -> Self {
        self.world.add(self.entity, component);
        self
    }

    /// Finish building and return the entity handle.
    pub fn build(self) -> Entity { self.entity }
}
