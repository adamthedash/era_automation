use std::marker::PhantomData;
use std::{hash::Hash, ops::Deref};

use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        system::SystemParam,
    },
    platform::collections::HashMap,
    prelude::*,
};

/// A convenience parameter for the following pattern:
/// ```rust
/// fn system(
///     stockpile_lut: Res<StockpileLut>,
///     mut stockpiles: Query<&mut ResourceStockpile>,
/// ) {
///     stockpile_lut
///         .get(&ResourceType::Wood)
///         .and_then(|entity| stockpiles.get(*entity).ok())
/// }
/// ```
/// becomes:
/// ```rust
/// fn system(
///     mut stockpiles: LUTParam<StockpileLut, &mut ResourceStockpile>,
/// ){
///     stockpiles.get(&ResourceType:::Wood)
/// }
/// ```
#[derive(SystemParam)]
pub struct LUTParam<'w, 's, L, K, Q, F = ()>
where
    L: Resource + Deref<Target = HashMap<K, Entity>>,
    K: Eq + Hash + 'static,
    Q: QueryData + 'static,
    F: QueryFilter + 'static,
{
    lut: Res<'w, L>,
    query: Query<'w, 's, Q, F>,
    _phantom: PhantomData<K>,
}

impl<'w, 's, L, K, Q, F> LUTParam<'w, 's, L, K, Q, F>
where
    L: Resource + Deref<Target = HashMap<K, Entity>>,
    K: Eq + Hash,
    Q: QueryData + 'static,
    F: QueryFilter,
{
    pub fn get(&self, key: &K) -> Option<<Q::ReadOnly as QueryData>::Item<'_, '_>> {
        self.lut
            .get(key)
            .and_then(|entity| self.query.get(*entity).ok())
    }

    pub fn get_mut(&mut self, key: &K) -> Option<Q::Item<'_, 's>> {
        self.lut
            .get(key)
            .and_then(|entity| self.query.get_mut(*entity).ok())
    }
}
