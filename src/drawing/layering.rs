//! Original Code + Inspiration: https://github.com/goto64/bevy_2d_screen_space_lightmaps/blob/master/src/lightmap_plugin/lightmap_plugin.rs
//! Tweaked it to be my own for more control + understanding

use crate::camera::CameraMarker;
use crate::camera::DynamicCameraBundle;
use crate::meta::consts::MENU_HEIGHT;
use crate::meta::consts::MENU_WIDTH;
use crate::meta::consts::SCREEN_HEIGHT;
use crate::meta::consts::SCREEN_WIDTH;
use crate::physics::dyno::IntMoveable;
use bevy::core_pipeline::bloom::BloomCompositeMode;
use bevy::core_pipeline::bloom::BloomPrefilterSettings;
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::prelude::*;
use bevy::render::camera::ClearColorConfig;
use bevy::render::camera::RenderTarget;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d,
    RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle};

use super::animation_mat::AnimationMaterial;

pub struct LayeringPlugin;

impl Plugin for LayeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlendTexturesMaterial>::default());
        app.add_plugins(Material2dPlugin::<ReducedMaterial>::default());
        app.add_systems(
            Startup,
            (setup_post_processing_camera, setup_sprite_camera).chain(),
        );
        app.init_resource::<LayeringPluginSettings>();
        app.init_resource::<CameraTargets>();
    }
}

#[derive(Component)]
pub struct BgSpriteCameraMarker;
pub fn bg_sprite_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_BG_SPRITE)
}
pub fn bg_sprite_layer_u8() -> u8 {
    CAMERA_LAYER_BG_SPRITE[0]
}

#[derive(Component)]
pub struct SpriteCameraMarker;
pub fn sprite_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_SPRITE)
}
pub fn sprite_layer_u8() -> u8 {
    CAMERA_LAYER_SPRITE[0]
}

#[derive(Component)]
pub struct BgLightCameraMarker;
pub fn bg_light_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_BG_LIGHT)
}
pub fn bg_light_layer_u8() -> u8 {
    CAMERA_LAYER_BG_LIGHT[0]
}

#[derive(Component)]
pub struct LightCameraMarker;
pub fn light_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_LIGHT)
}
pub fn light_layer_u8() -> u8 {
    CAMERA_LAYER_LIGHT[0]
}

#[derive(Component)]
struct ReducedCameraMarker;

#[derive(Component)]
pub struct MenuCameraMarker;
pub fn menu_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_MENU)
}
pub fn menu_layer_u8() -> u8 {
    CAMERA_LAYER_MENU[0]
}

#[derive(Resource)]
pub struct LayeringPluginSettings {
    bg_clear_color: ClearColorConfig,
    bg_ambient_light: Color,
    clear_color: ClearColorConfig,
    ambient_light: Color,
}

impl Default for LayeringPluginSettings {
    fn default() -> Self {
        Self {
            bg_clear_color: ClearColorConfig::Default,
            bg_ambient_light: Color::rgb(0.2, 0.2, 0.2),
            clear_color: ClearColorConfig::Custom(Color::Rgba {
                red: 0.1,
                green: 0.1,
                blue: 0.1,
                alpha: 0.05,
            }),
            ambient_light: Color::rgb(0.7, 0.7, 0.7),
        }
    }
}

/// All background sprites must be added to this camera layer
pub const CAMERA_LAYER_BG_SPRITE: &[u8] = &[1];

/// All background sprites must be added to this camera layer
pub const CAMERA_LAYER_BG_LIGHT: &[u8] = &[2];

/// All normal sprites must be added to this camera layer
pub const CAMERA_LAYER_SPRITE: &[u8] = &[3];

/// All light sprites must be added to this camera layer
pub const CAMERA_LAYER_LIGHT: &[u8] = &[4];

/// Reduced layer
const CAMERA_LAYER_REDUCED: &[u8] = &[5];

/// All menu components (non-rounding pixelated, rendered last) must be added to this layer
pub const CAMERA_LAYER_MENU: &[u8] = &[6];

#[derive(Component)]
struct PostProcessingQuad;
const BLEND_ADD: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
};

#[derive(AsBindGroup, TypePath, Asset, Debug, Clone)]
pub(super) struct BlendTexturesMaterial {
    #[texture(1)]
    #[sampler(2)]
    pub texture1: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub texture2: Handle<Image>,
}

impl Material2d for BlendTexturesMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blend_light.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BLEND_ADD);
            }
        }

        Ok(())
    }
}

#[derive(AsBindGroup, TypePath, Asset, Debug, Clone)]
pub(super) struct ReducedMaterial {
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
    #[uniform(3)]
    pub num_pixels_w: f32,
    #[uniform(4)]
    pub num_pixels_h: f32,
}

impl Material2d for ReducedMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/reduce.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BLEND_ADD);
            }
        }

        Ok(())
    }
}

#[derive(Resource, Default)]
pub(super) struct CameraTargets {
    pub bg_sprite_target: Handle<Image>,
    pub bg_light_target: Handle<Image>,
    pub sprite_target: Handle<Image>,
    pub light_target: Handle<Image>,
    pub reduced_target: Handle<Image>,
    pub menu_target: Handle<Image>,
}

impl CameraTargets {
    pub fn create(images: &mut Assets<Image>, sizes: &Vec2) -> Self {
        let target_size = Extent3d {
            width: sizes.x as u32,
            height: sizes.y as u32,
            ..default()
        };
        let menu_size = Extent3d {
            width: MENU_WIDTH as u32,
            height: MENU_HEIGHT as u32,
            ..default()
        };

        let mut bg_sprite_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("target_bg_sprite"),
                size: target_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        let mut bg_light_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("target_bg_light"),
                size: target_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        let mut sprite_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("target_sprite"),
                size: target_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        let mut light_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("target_light"),
                size: target_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        let mut reduced_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("target_reduced"),
                size: target_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        let mut menu_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("target_menu"),
                size: menu_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        // Fill images data with zeroes.
        bg_sprite_image.resize(target_size);
        bg_light_image.resize(target_size);
        sprite_image.resize(target_size);
        light_image.resize(target_size);
        reduced_image.resize(target_size);
        menu_image.resize(menu_size);

        let bg_sprite_image_handle: Handle<Image> = Handle::weak_from_u128(84562364042238462870);
        let bg_light_image_handle: Handle<Image> = Handle::weak_from_u128(81297563682952991276);
        let sprite_image_handle: Handle<Image> = Handle::weak_from_u128(84562364042238462871);
        let light_image_handle: Handle<Image> = Handle::weak_from_u128(81297563682952991277);
        let reduced_image_handle: Handle<Image> = Handle::weak_from_u128(81297563682952991278);
        let menu_image_handle: Handle<Image> = Handle::weak_from_u128(51267563632952991278);

        images.insert(bg_sprite_image_handle.clone(), bg_sprite_image);
        images.insert(bg_light_image_handle.clone(), bg_light_image);
        images.insert(sprite_image_handle.clone(), sprite_image);
        images.insert(light_image_handle.clone(), light_image);
        images.insert(reduced_image_handle.clone(), reduced_image);
        images.insert(menu_image_handle.clone(), menu_image);

        Self {
            bg_sprite_target: bg_sprite_image_handle,
            bg_light_target: bg_light_image_handle,
            sprite_target: sprite_image_handle,
            light_target: light_image_handle,
            reduced_target: reduced_image_handle,
            menu_target: menu_image_handle,
        }
    }
}

fn setup_sprite_camera(
    mut commands: Commands,
    camera_targets: Res<CameraTargets>,
    lightmap_plugin_settings: Res<LayeringPluginSettings>,
) {
    let bg_bloom = BloomSettings {
        intensity: 0.56,
        low_frequency_boost: 0.7,
        low_frequency_boost_curvature: 0.95,
        high_pass_frequency: 1.0,
        prefilter_settings: BloomPrefilterSettings {
            threshold: 0.2,
            threshold_softness: 0.2,
        },
        composite_mode: BloomCompositeMode::Additive,
    };
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    order: 0,
                    target: RenderTarget::Image(camera_targets.bg_sprite_target.clone()),
                    clear_color: lightmap_plugin_settings.bg_clear_color.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("bg_sprite_camera"),
            BgSpriteCameraMarker,
            bg_bloom.clone(),
        ))
        .insert(RenderLayers::from_layers(CAMERA_LAYER_BG_SPRITE));

    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    order: 1,
                    target: RenderTarget::Image(camera_targets.bg_light_target.clone()),
                    clear_color: ClearColorConfig::Custom(
                        lightmap_plugin_settings.bg_ambient_light,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("bg_light_camera"),
            BgLightCameraMarker,
            bg_bloom.clone(),
        ))
        .insert(RenderLayers::from_layers(CAMERA_LAYER_BG_LIGHT));

    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    order: 2,
                    target: RenderTarget::Image(camera_targets.sprite_target.clone()),
                    clear_color: lightmap_plugin_settings.clear_color.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("sprite_camera"),
            SpriteCameraMarker,
        ))
        .insert(RenderLayers::from_layers(CAMERA_LAYER_SPRITE));

    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    order: 3,
                    target: RenderTarget::Image(camera_targets.light_target.clone()),
                    clear_color: ClearColorConfig::Custom(lightmap_plugin_settings.ambient_light),
                    ..Default::default()
                },
                ..Default::default()
            },
            Name::new("light_camera"),
            LightCameraMarker,
        ))
        .insert(RenderLayers::from_layers(CAMERA_LAYER_LIGHT));

    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    order: 4,
                    target: RenderTarget::Image(camera_targets.reduced_target.clone()),
                    clear_color: ClearColorConfig::Default,
                    ..Default::default()
                },
                ..Default::default()
            },
            ReducedCameraMarker,
            Name::new("reduced_camera"),
        ))
        .insert(RenderLayers::from_layers(CAMERA_LAYER_REDUCED));

    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    order: 5,
                    target: RenderTarget::Image(camera_targets.menu_target.clone()),
                    clear_color: ClearColorConfig::from(Color::Hsla {
                        hue: 0.0,
                        saturation: 0.0,
                        lightness: 0.0,
                        alpha: 0.0,
                    }),
                    ..Default::default()
                },
                projection: OrthographicProjection {
                    scale: 2.0,
                    near: -1000.0,
                    far: 1000.0,
                    ..default()
                },
                ..Default::default()
            },
            MenuCameraMarker,
            Name::new("menu_camera"),
        ))
        .insert(RenderLayers::from_layers(CAMERA_LAYER_MENU));
}

const BG_PP_QUAD: Handle<Mesh> = Handle::weak_from_u128(23467206864860343677);
const BG_PP_MATERIAL: Handle<BlendTexturesMaterial> = Handle::weak_from_u128(52374148673736462870);

const PP_QUAD: Handle<Mesh> = Handle::weak_from_u128(23467206864860343678);
const PP_MATERIAL: Handle<BlendTexturesMaterial> = Handle::weak_from_u128(52374148673736462871);

const REDUCED_QUAD: Handle<Mesh> = Handle::weak_from_u128(23467206864860383170);
const REDUCED_MATERIAL: Handle<ReducedMaterial> = Handle::weak_from_u128(52374148673136432070);

const MENU_QUAD: Handle<Mesh> = Handle::weak_from_u128(36467206864860383170);
const MENU_MATERIAL: Handle<AnimationMaterial> = Handle::weak_from_u128(29374148673136432070);

#[derive(Component)]
pub(super) struct ScaledOutputQuad;

#[derive(Component)]
pub(super) struct ScaledMenuQuad;

pub(super) fn remake_layering_materials(
    camera_targets: &CameraTargets,
    materials: &mut ResMut<Assets<BlendTexturesMaterial>>,
    dum_materials: &mut ResMut<Assets<ReducedMaterial>>,
    anim_materials: &mut ResMut<Assets<AnimationMaterial>>,
) {
    let bg_material = BlendTexturesMaterial {
        texture1: camera_targets.bg_sprite_target.clone(),
        texture2: camera_targets.bg_light_target.clone(),
    };
    let material = BlendTexturesMaterial {
        texture1: camera_targets.sprite_target.clone(),
        texture2: camera_targets.light_target.clone(),
    };
    let reduced_material = ReducedMaterial {
        texture: camera_targets.reduced_target.clone(),
        num_pixels_w: SCREEN_WIDTH as f32,
        num_pixels_h: SCREEN_HEIGHT as f32,
    };
    let menu_material =
        AnimationMaterial::from_handle(camera_targets.menu_target.clone(), 1, Vec2::ONE);
    materials.insert(BG_PP_MATERIAL.clone(), bg_material);
    materials.insert(PP_MATERIAL.clone(), material);
    dum_materials.insert(REDUCED_MATERIAL.clone(), reduced_material);
    anim_materials.insert(MENU_MATERIAL.clone(), menu_material);
}

fn setup_post_processing_camera(
    mut commands: Commands,
    mut camera_targets: ResMut<CameraTargets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlendTexturesMaterial>>,
    mut dum_materials: ResMut<Assets<ReducedMaterial>>,
    mut anim_materials: ResMut<Assets<AnimationMaterial>>,
) {
    let primary_size = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
    let quad = Mesh::from(Rectangle::new(primary_size.x, primary_size.y));
    meshes.insert(BG_PP_QUAD.clone(), quad.clone());
    meshes.insert(PP_QUAD.clone(), quad.clone());
    meshes.insert(REDUCED_QUAD.clone(), quad.clone());
    let quad = Mesh::from(Rectangle::new(primary_size.x * 2.0, primary_size.y * 2.0));
    meshes.insert(MENU_QUAD.clone(), quad.clone());

    *camera_targets = CameraTargets::create(&mut images, &primary_size);

    remake_layering_materials(
        &camera_targets,
        &mut materials,
        &mut dum_materials,
        &mut anim_materials,
    );

    let reduced_layer = RenderLayers::from_layers(CAMERA_LAYER_REDUCED);
    let output_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    commands.spawn((
        PostProcessingQuad,
        MaterialMesh2dBundle {
            mesh: BG_PP_QUAD.clone().into(),
            material: BG_PP_MATERIAL.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        Name::new("bg_reduced_layer"),
        reduced_layer,
    ));

    commands.spawn((
        PostProcessingQuad,
        MaterialMesh2dBundle {
            mesh: PP_QUAD.clone().into(),
            material: PP_MATERIAL.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..default()
            },
            ..default()
        },
        Name::new("reduced_layer"),
        reduced_layer,
    ));

    commands.spawn((
        ScaledOutputQuad,
        PostProcessingQuad,
        MaterialMesh2dBundle {
            mesh: REDUCED_QUAD.clone().into(),
            material: REDUCED_MATERIAL.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
        Name::new("reduced_output_layer"),
        output_layer,
    ));

    commands.spawn((
        ScaledMenuQuad,
        PostProcessingQuad,
        MaterialMesh2dBundle {
            mesh: MENU_QUAD.clone().into(),
            material: MENU_MATERIAL.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
        Name::new("menu_output_layer"),
        output_layer,
    ));

    // Camera that renders the final image for the screen
    commands
        .spawn((
            Name::new("post_processing_camera"),
            Camera2dBundle {
                camera: Camera {
                    order: 6,
                    hdr: true,
                    ..default()
                },
                ..default()
            },
            InheritedVisibility::VISIBLE,
            output_layer,
        ))
        .with_children(|parent| {
            parent.spawn(DynamicCameraBundle {
                marker: CameraMarker::new(),
                moveable: IntMoveable::default(),
                spatial: SpatialBundle::default(),
            });
        });
}
