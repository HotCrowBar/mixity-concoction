use crate::prelude::*;
use bevy::{gltf::Gltf, render::{once_cell::sync::OnceCell, view::RenderLayers}, scene::SceneInstance, reflect::TypeUuid};
pub use bevy_asset_loader::prelude::*;
use bevy::ecs::query::ReadOnlyWorldQuery;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset::<Spawnable>()
			.add_loading_state(
                LoadingState::new(GameState::LoadingAssets)
                    .continue_to_state(GameState::InGame)
            )
			.add_collection_to_loading_state::<_, GameAssets>(GameState::LoadingAssets)
            .add_system(setup.in_schedule(OnExit(GameState::LoadingAssets)))
            .add_systems((
                check_scene_init,
                update_scene_children::<RenderLayers, With<Handle<Mesh>>>,
                update_scene_children::<CollisionGroups, With<Collider>>
            ));
    }
} 

pub static SHADOW_BUNDLE: OnceCell<(Handle<StandardMaterial>,Handle<Mesh>)> = OnceCell::new();

fn setup(
    //commands: Commands,
    mut spawnable_assets: ResMut<Assets<Spawnable>>,
	mut mesh_assets: ResMut<Assets<Mesh>>,
	mut material_assets: ResMut<Assets<StandardMaterial>>,
	assets: Res<GameAssets>,
    gltfs: Res<Assets<Gltf>>,
) {
	let plane_mesh = mesh_assets.add(Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 }));

	let shadow_material = material_assets.add(StandardMaterial {
		base_color_texture: Some(assets.circle_texture.clone()),
		alpha_mode: AlphaMode::Mask(0.5),
        base_color: Color::rgb(0.6 * 0.9, 0.8 * 0.9, 0.2 * 0.9),
        unlit: true,
		..default()
	});

	SHADOW_BUNDLE.set((shadow_material,plane_mesh)).unwrap();
    
    let tree_scenes = &gltfs.get(&assets.tree_gltf).unwrap().scenes;
    for (_i,tree) in tree_scenes.iter().enumerate() {
        let spawnable = Spawnable {
            archetype: SpawnableArchetype::Tree,
            scene: tree.clone(),
            ingridient: None,
            spawn_weight: 1.5 / tree_scenes.len() as f32,
            size: 2.8
        };
        spawnable_assets.add(spawnable);
    }

    let bush_scenes = &gltfs.get(&assets.bush_gltf).unwrap().scenes;
    for (i,scene) in bush_scenes.iter().enumerate() {
        let spawnable = Spawnable {
            archetype: SpawnableArchetype::Bush,
            scene: scene.clone(),
            ingridient: None,
            spawn_weight: 1.0 / tree_scenes.len() as f32,
            size: match i {
                1 => 2.0,
                _ => 1.5
            }
        };
        spawnable_assets.add(spawnable);
    }

    let ingridient_scenes = &gltfs.get(&assets.ingridients_gltf).unwrap().scenes;
    for (_i,scene) in ingridient_scenes.iter().enumerate() {
        let spawnable = Spawnable {
            archetype: SpawnableArchetype::Mushroom,
            scene: scene.clone(),
            ingridient: Some(SpawnableIngridient { 
                pick_event: PickUpEvent::Destroy,
                inventory_scene: scene.clone(),
            }),
            spawn_weight: 0.5 / ingridient_scenes.len() as f32,
            size: 0.6
        };
        spawnable_assets.add(spawnable);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnableIngridient {
    pick_event: PickUpEvent,
    inventory_scene: Handle<Scene>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickUpEvent {
    Destroy,
    Replace(Handle<Scene>),
}

#[derive(Clone, Debug, TypeUuid, PartialEq)]
#[uuid = "2e680e06-a271-4804-8f5a-73927db8dec4"]
pub struct Spawnable {
    pub scene: Handle<Scene>,
    pub archetype: SpawnableArchetype,
    pub ingridient: Option<SpawnableIngridient>,
    pub spawn_weight: f32,
    pub size: f32,
}

// Use hash from Scene handle. Each scene handle SHOULD have only one spawnable.
impl std::hash::Hash for Spawnable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.scene.hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnableArchetype {
    Tree,
    Bush,
    Mushroom,
}


#[derive(AssetCollection, Resource)]
pub struct GameAssets {
	#[asset(path = "textures/dither.png")]
    pub dither_texture: Handle<Image>,
    #[asset(path = "textures/circle.png")]
    pub circle_texture: Handle<Image>,
    #[asset(path = "models/tree.gltf")]
    pub tree_gltf: Handle<Gltf>,
    #[asset(path = "models/bush.gltf")]
    pub bush_gltf: Handle<Gltf>,
    #[asset(path = "models/ingridients.gltf")]
    pub ingridients_gltf: Handle<Gltf>,
}

#[derive(Component)]
pub struct SceneInstanceReady;

/// Checks if scene instance is ready and adds a `SceneInstanceReady` component to it
pub fn check_scene_init(
    mut commands: Commands,
    scene_manager: Res<SceneSpawner>,
    scene_query: Query<(Entity, &SceneInstance), Without<SceneInstanceReady>>,
    //changed_scene:  Query<(Entity, &SceneInstance), (With<SceneInstanceReady>, Changed<SceneInstance>)>,
) {
    for (entity,scene_id) in &scene_query {
        if scene_manager.instance_is_ready(**scene_id) {
            commands.entity(entity).insert(SceneInstanceReady);
        }
    }
}

/// Gtlf importer is absolute shite. <br>
/// To metigate this, I created this little system, that will apply some of the components from the original scene entity to all of it's descendants (With filters) <br>
/// It janky. It work. It stay.
#[allow(clippy::type_complexity)]
pub fn update_scene_children<T: Component + Clone, F: ReadOnlyWorldQuery>(
    mut commands: Commands,
	children_query: Query<&Children>,
    scene_query: Query<(Entity, &T), (With<SceneInstanceReady>, Or<(Added<SceneInstanceReady>, Changed<T>)>)>,
    mut t_query: Query<Option<&mut T>, (F, Without<SceneInstanceReady>)>,
) {
    for (parent,parent_t) in &scene_query {
        for child in children_query.iter_descendants(parent) {
            // Check filters
            let Ok(maybe_t) = t_query.get_mut(child) else {
                continue;
            };

            // Check if T already exists
            if let Some(mut t) = maybe_t {
                *t = parent_t.clone();
            } else {
                commands.entity(child).insert(parent_t.clone());
            }
        }
    }
}

