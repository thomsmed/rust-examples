use bevy::math::FloatOrd;
use bevy::prelude::*;

pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_texture_atlas_index, update_flip, update_transform).chain(),
        );
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
    pub index: TextureAtlasIndex,
    pub flip_x: bool,
    pub flip_y: bool,
    /// A relative translation to whatever is specified in the [Sprite]'s current [Transform].
    pub translation: Vec3,
    /// A relative rotation to whatever is specified in the [Sprite]'s current [Transform].
    pub rotation: Quat,
    /// A relative scale to whatever is specified in the [Sprite]'s current [Transform].
    pub scale: Vec3,
    previous_transform: Transform,
}

impl AnimatedSprite {
    pub fn from_index(index: usize) -> Self {
        let transform = Transform::IDENTITY;
        Self {
            index: TextureAtlasIndex::new(index),
            flip_x: false,
            flip_y: false,
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
            previous_transform: transform,
        }
    }
}

impl Default for AnimatedSprite {
    fn default() -> Self {
        let transform = Transform::IDENTITY;
        Self {
            index: TextureAtlasIndex::new(0),
            flip_x: false,
            flip_y: false,
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
            previous_transform: transform,
        }
    }
}

fn update_texture_atlas_index(
    sprites: Query<(&mut Sprite, &AnimatedSprite)>,
) {
    for (mut sprite, animated_sprite) in sprites {
        let inner_sprite = sprite.bypass_change_detection();

        let Some(texture_atlas) = &mut inner_sprite.texture_atlas else {
            continue;
        };

        let new_index = *animated_sprite.index;

        if new_index != texture_atlas.index {
            texture_atlas.index = new_index;
            sprite.set_changed();
        }
    }
}

fn update_flip(
    sprites: Query<(&mut Sprite, &AnimatedSprite)>,
) {
    for (mut sprite, animated_sprite) in sprites {
        let inner_sprite = sprite.bypass_change_detection();

        let flip_x = inner_sprite.flip_x;
        let flip_y = inner_sprite.flip_y;

        if flip_x != animated_sprite.flip_x || flip_y != animated_sprite.flip_y {
            sprite.flip_x = flip_x;
            sprite.flip_y = flip_y;
        }
    }
}

fn update_transform(
    sprites: Query<(&mut Transform, &mut AnimatedSprite), With<Sprite>>,
) {
    for (mut transform,mut animated_sprite) in sprites {
        let inner_animated_sprite = animated_sprite.bypass_change_detection();

        let animated_transform = Transform {
            translation: inner_animated_sprite.translation,
            rotation: inner_animated_sprite.rotation,
            scale: inner_animated_sprite.scale,
        };

        if animated_transform != inner_animated_sprite.previous_transform {
            let delta_transform = Transform {
                translation: animated_transform.translation - inner_animated_sprite.previous_transform.translation,
                rotation: animated_transform.rotation - inner_animated_sprite.previous_transform.rotation,
                scale: animated_transform.scale - inner_animated_sprite.previous_transform.scale,
            };

            transform.translation += delta_transform.translation;
            transform.rotation += delta_transform.rotation;
            transform.scale += delta_transform.scale;

            animated_sprite.previous_transform = animated_transform;
        }
    }
}
