use bevy::prelude::*;

use crate::{
    drawing::{
        animation::{AnimationManager, SpriteInfo},
        layering::menu_layer_u8,
    },
    environment::convo::{
        operation::{
            MaterializedPortrait, MaterializedPortraitBundle, MaterializedSound,
            MaterializedSoundBundle,
        },
        ConvoBoxSpeaker, SpeakerEmotion,
    },
    meta::consts::MENU_GROWTH,
    sound::effect::SoundEffect,
};

impl ConvoBoxSpeaker {
    pub fn materialize(
        &self,
        portrait_offset: Option<Vec2>,
    ) -> Option<(MaterializedPortraitBundle, MaterializedSoundBundle)> {
        let common_data: Option<((SpriteInfo, u32), (&str, f32))> = match self {
            Self::None => None,
            Self::Ship { emotion } => match emotion {
                SpeakerEmotion::Default => {
                    let anim_data = (
                        SpriteInfo {
                            path: "sprites/convo/speakers/narf/default.png".into(),
                            size: UVec2::new(22, 22),
                            ..default()
                        },
                        1,
                    );
                    let sound_path = "sound_effects/speakers/narf/default.ogg";
                    let sound_intensity = 0.3;
                    Some((anim_data, (sound_path, sound_intensity)))
                }
            },
        };
        let Some(((sprite, anim_len), (sound_path, sound_intensity))) = common_data else {
            return None;
        };
        let portrait_offset = portrait_offset.unwrap_or_default();
        let portrait_bund = MaterializedPortraitBundle {
            name: Name::new("portrait"),
            marker: MaterializedPortrait,
            anim: AnimationManager::single_repeating(sprite, anim_len)
                .force_render_layer(menu_layer_u8()),
            spatial: SpatialBundle::from_transform(Transform {
                scale: (Vec2::ONE * MENU_GROWTH as f32).extend(1.0),
                translation: portrait_offset.extend(0.0),
                ..default()
            }),
        };
        let sound_bund = MaterializedSoundBundle {
            name: Name::new("sound"),
            marker: MaterializedSound,
            sound: SoundEffect::universal(sound_path, sound_intensity, true),
        };
        Some((portrait_bund, sound_bund))
    }
}
