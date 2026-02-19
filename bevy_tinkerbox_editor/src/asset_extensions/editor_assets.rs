/// Plumbing to help frontload initializing our UI assets.

use bevy::image::ImageFilterMode;

use bevy::image::ImageLoaderSettings;
use bevy::image::ImageSamplerDescriptor;

use bevy::image::ImageSampler;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AssortedIcons>();
}

/// Assets required for our editor UI.
#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AssortedIcons {
    #[dependency]
    pub trash: Handle<Image>,
    #[dependency]
    pub package_plus: Handle<Image>,
    #[dependency]
    pub list_plus: Handle<Image>,
    //TODO - we do not actually need a ducky here.
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub move_icon: Handle<Image>,
    #[dependency]
    pub rotate_ccw: Handle<Image>,
    #[dependency]
    pub maximize_2: Handle<Image>,
    #[dependency]
    pub eye: Handle<Image>,
    #[dependency]
    pub move_3d: Handle<Image>,
    #[dependency]
    pub folder: Handle<Image>,
    #[dependency]
    pub square_pen: Handle<Image>,
    #[dependency]
    pub checker_board: Handle<Image>,
}

impl FromWorld for AssortedIcons {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            trash: assets.load("lucide/trash-2-white.png"),
            package_plus: assets.load("lucide/package-plus-white.png"),
            list_plus: assets.load("lucide/list-plus-white.png"),
            ducky: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            move_icon: assets.load("lucide/move-white.png"),
            rotate_ccw: assets.load("lucide/rotate-ccw-white.png"),
            maximize_2: assets.load("lucide/maximize-2-white.png"),
            eye: assets.load("lucide/eye-white.png"),
            move_3d: assets.load("lucide/move-3d-white.png"),
            folder: assets.load("lucide/folder-white.png"),
            square_pen: assets.load("lucide/square-pen-white.png"),
            checker_board: assets.load_with_settings(
                "images/Kenny/Checkerboard/checkerboard.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: bevy::image::ImageAddressMode::Repeat,
                        address_mode_v: bevy::image::ImageAddressMode::Repeat,
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    })
                },
            ),
        }
    }
}
