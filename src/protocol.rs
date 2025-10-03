use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_egui::egui::Color32;
use lightyear::{input::native::plugin::InputPlugin, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Fill(pub Color32);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Velocity(pub Vec3);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Radius(pub f32);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[require(Mass, Crafts)]
pub struct Body;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Mass(pub f32);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Crafts(pub u32);

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Fill>();
        app.register_component::<Velocity>();
        app.register_component::<Radius>();
        app.register_component::<Body>();
        app.register_component::<Mass>();
        app.register_component::<Crafts>();

        app.add_plugins(InputPlugin::<Inputs>::default());

        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Reflect, Default)]
pub struct CraftDelta(i32);

#[derive(Serialize, Deserialize, Debug, PartialEq, Reflect, Eq, Clone)]
pub enum Inputs {
    CraftDelta(CraftDelta),
}

impl Default for Inputs {
    fn default() -> Self {
        Self::CraftDelta(CraftDelta(0))
    }
}

// All inputs need to implement the `MapEntities` trait
impl MapEntities for Inputs {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {}
}

pub struct Channel1;
