//! Original Code + Inspiration: https://github.com/goto64/bevy_2d_screen_space_lightmaps/blob/master/src/lightmap_plugin/lightmap_plugin.rs
//! Tweaked it to be my own for more control + understanding

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
use bevy::window::PrimaryWindow;

use crate::camera::CameraMarker;
use crate::meta::consts::NUM_PIXELS;

use super::post_pixel::PostPixelSettings;

pub struct LightmapPlugin;

impl Plugin for LightmapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlendTexturesMaterial>::default());
        app.add_systems(
            Startup,
            (setup_post_processing_camera, setup_sprite_camera).chain(),
        );
        app.add_systems(Update, on_resize_window);
        app.init_resource::<LightmapPluginSettings>();
        app.init_resource::<CameraTargets>();
    }
}

#[derive(Component)]
pub struct BgSpriteCameraMarker;
pub fn bg_sprite_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_BG_SPRITE)
}

#[derive(Component)]
pub struct SpriteCameraMarker;
pub fn sprite_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_SPRITE)
}

#[derive(Component)]
pub struct BgLightCameraMarker;
pub fn bg_light_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_BG_LIGHT)
}

#[derive(Component)]
pub struct LightCameraMarker;
pub fn light_layer() -> RenderLayers {
    RenderLayers::from_layers(CAMERA_LAYER_LIGHT)
}

#[derive(Resource)]
pub struct LightmapPluginSettings {
    bg_clear_color: ClearColorConfig,
    bg_ambient_light: Color,
    clear_color: ClearColorConfig,
    ambient_light: Color,
    bloom: Option<BloomSettings>,
}

impl Default for LightmapPluginSettings {
    fn default() -> Self {
        Self {
            bg_clear_color: ClearColorConfig::Default,
            bg_ambient_light: Color::rgb(0.2, 0.2, 0.2),
            clear_color: ClearColorConfig::Custom(Color::Rgba {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.0,
            }),
            ambient_light: Color::rgb(0.5, 0.5, 0.5),
            bloom: None,
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
struct BlendTexturesMaterial {
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

#[derive(Resource, Default)]
struct CameraTargets {
    pub bg_sprite_target: Handle<Image>,
    pub bg_light_target: Handle<Image>,
    pub sprite_target: Handle<Image>,
    pub light_target: Handle<Image>,
}

impl CameraTargets {
    pub fn create(images: &mut Assets<Image>, sizes: &Vec2) -> Self {
        let target_size = Extent3d {
            width: sizes.x as u32,
            height: sizes.y as u32,
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

        // Fill images data with zeroes.
        bg_sprite_image.resize(target_size);
        bg_light_image.resize(target_size);
        sprite_image.resize(target_size);
        light_image.resize(target_size);

        let bg_sprite_image_handle: Handle<Image> = Handle::weak_from_u128(84562364042238462870);
        let bg_light_image_handle: Handle<Image> = Handle::weak_from_u128(81297563682952991276);
        let sprite_image_handle: Handle<Image> = Handle::weak_from_u128(84562364042238462871);
        let light_image_handle: Handle<Image> = Handle::weak_from_u128(81297563682952991277);

        images.insert(bg_sprite_image_handle.clone(), bg_sprite_image);
        images.insert(bg_light_image_handle.clone(), bg_light_image);
        images.insert(sprite_image_handle.clone(), sprite_image);
        images.insert(light_image_handle.clone(), light_image);

        Self {
            bg_sprite_target: bg_sprite_image_handle,
            bg_light_target: bg_light_image_handle,
            sprite_target: sprite_image_handle,
            light_target: light_image_handle,
        }
    }
}

fn setup_sprite_camera(
    mut commands: Commands,
    camera_targets: Res<CameraTargets>,
    lightmap_plugin_settings: Res<LightmapPluginSettings>,
) {
    let bg_bloom = BloomSettings {
        intensity: 0.46,
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
}

const BG_PP_QUAD: Handle<Mesh> = Handle::weak_from_u128(23467206864860343677);
const BG_PP_MATERIAL: Handle<BlendTexturesMaterial> = Handle::weak_from_u128(52374148673736462870);

const PP_QUAD: Handle<Mesh> = Handle::weak_from_u128(23467206864860343678);
const PP_MATERIAL: Handle<BlendTexturesMaterial> = Handle::weak_from_u128(52374148673736462871);

fn setup_post_processing_camera(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    lightmap_plugin_settings: Res<LightmapPluginSettings>,
    mut camera_targets: ResMut<CameraTargets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlendTexturesMaterial>>,
) {
    let Ok(window) = window.get_single() else {
        panic!("No window")
    };
    let primary_size = Vec2::new(
        (window.physical_width() as f32 / window.scale_factor()) as f32,
        (window.physical_height() as f32 / window.scale_factor()) as f32,
    );

    let quad = Mesh::from(Rectangle::new(primary_size.x, primary_size.y));
    meshes.insert(BG_PP_QUAD.clone(), quad.clone());
    meshes.insert(PP_QUAD.clone(), quad.clone());

    *camera_targets = CameraTargets::create(&mut images, &primary_size);

    let bg_material = BlendTexturesMaterial {
        texture1: camera_targets.bg_sprite_target.clone(),
        texture2: camera_targets.bg_light_target.clone(),
    };

    let material = BlendTexturesMaterial {
        texture1: camera_targets.sprite_target.clone(),
        texture2: camera_targets.light_target.clone(),
    };

    materials.insert(BG_PP_MATERIAL.clone(), bg_material);
    materials.insert(PP_MATERIAL.clone(), material);

    let layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

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
        layer,
    ));

    commands.spawn((
        PostProcessingQuad,
        MaterialMesh2dBundle {
            mesh: PP_QUAD.clone().into(),
            material: PP_MATERIAL.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        layer,
    ));

    // Camera that renders the final image for the screen
    let camera_id = commands
        .spawn((
            Name::new("post_processing_camera"),
            Camera2dBundle {
                camera: Camera {
                    order: 1,
                    hdr: true,
                    ..default()
                },
                ..Camera2dBundle::default()
            },
            CameraMarker::new(),
            PostPixelSettings {
                num_pixels: NUM_PIXELS as f32,
            },
            layer,
        ))
        .id();

    if lightmap_plugin_settings.bloom.is_some() {
        commands
            .entity(camera_id)
            .insert(lightmap_plugin_settings.bloom.clone().unwrap());
    }
}

fn on_resize_window(// mut resize_reader: EventReader<WindowResized>,
    // window: Query<&Window, With<PrimaryWindow>>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut camera_targets: ResMut<CameraTargets>,
    // mut images: ResMut<Assets<Image>>,
    // mut materials: ResMut<Assets<BlendTexturesMaterial>>,
) {
    // TODO: Re-enable this, or disable changing size
    // for ev in resize_reader.read() {
    //     let Ok(window) = window.get_single() else {
    //         panic!("No window")
    //     };
    //     let primary_size = Vec2::new(
    //         (ev.width / window.scale_factor()) as f32,
    //         (ev.height / window.scale_factor()) as f32,
    //     );

    //     let quad = Mesh::from(Rectangle::new(primary_size.x, primary_size.y));
    //     meshes.insert(POST_PROCESSING_QUAD.clone(), quad);

    //     *camera_targets = CameraTargets::create(&mut images, &primary_size);

    //     let material = BlendTexturesMaterial {
    //         texture1: camera_targets.sprite_target.clone(),
    //         texture2: camera_targets.light_target.clone(),
    //     };

    //     materials.insert(POST_PROCESSING_MATERIAL.clone(), material);
    // }
}
