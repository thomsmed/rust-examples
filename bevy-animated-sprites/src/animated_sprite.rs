use bevy::math::FloatOrd;
use bevy::prelude::*;

pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_current_texture_atlas_index);
    }
}

#[derive(Reflect, Deref, Clone, Default, Debug)]
pub struct TextureAtlasIndex(pub usize);

impl TextureAtlasIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

impl Animatable for TextureAtlasIndex {
    fn interpolate(a: &Self, b: &Self, time: f32) -> Self {
        if time < 1.0 { a.clone() } else { b.clone() }
    }

    fn blend(inputs: impl Iterator<Item = BlendInput<Self>>) -> Self {
        inputs
            .max_by_key(|x| FloatOrd(x.weight))
            .map_or(Self::default(), |x| x.value)
    }
}

#[derive(Component, Reflect, Clone, Debug)]
#[require(Sprite)]
pub struct AnimatedSprite {
    pub current_index: TextureAtlasIndex,
}

impl AnimatedSprite {
    pub fn from_index(index: usize) -> Self {
        Self {
            current_index: TextureAtlasIndex::new(index),
        }
    }
}

impl Default for AnimatedSprite {
    fn default() -> Self {
        Self {
            current_index: TextureAtlasIndex::new(0),
        }
    }
}

fn update_current_texture_atlas_index(sprites: Query<(&mut Sprite, &AnimatedSprite)>) {
    for (mut sprite, index_as_float) in sprites {
        let inner_sprite = sprite.bypass_change_detection();
        let Some(texture_atlas) = &mut inner_sprite.texture_atlas else {
            continue;
        };
        let current_index = *index_as_float.current_index;
        if current_index != texture_atlas.index {
            texture_atlas.index = current_index;
            sprite.set_changed();
        }
    }
}
