use bevy::{
    app::App,
    color::Luminance,
    ecs::system::ResMut,
    feathers::{self, theme::UiTheme},
    state::state::OnEnter,
};

use crate::LoadingStatus;

pub mod local_tokens {
    use bevy::feathers::theme::ThemeToken;

    pub const WARNING_BG: ThemeToken = ThemeToken::new_static("local.warning_bg");
    pub const WARNING_PRIMARY: ThemeToken = ThemeToken::new_static("local.warning_primary");
    pub const ITEM_BG: ThemeToken = ThemeToken::new_static("local.item_background");
    pub const ITEM_ACTIVE: ThemeToken = ThemeToken::new_static("local.item_active");
    pub const PANE_BG: ThemeToken = ThemeToken::new_static("local.pane_background");
    pub const PANE_BORDER: ThemeToken = ThemeToken::new_static("local.pane_border");
    pub const SLIDER_ACTIVE: ThemeToken = ThemeToken::new_static("local.slider_inactive");
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(LoadingStatus::Complete), theme_init);
}

fn theme_init(mut theme: ResMut<UiTheme>) {
    theme.set_color(
        &local_tokens::WARNING_BG.to_string(),
        feathers::palette::X_AXIS.darker(0.3),
    );
    theme.set_color(
        &local_tokens::WARNING_PRIMARY.to_string(),
        feathers::palette::X_AXIS,
    );
    theme.set_color(
        &local_tokens::ITEM_BG.to_string(),
        feathers::palette::GRAY_2,
    );
    theme.set_color(
        &local_tokens::ITEM_ACTIVE.to_string(),
        feathers::palette::GRAY_3,
    );
    theme.set_color(
        &local_tokens::PANE_BG.to_string(),
        feathers::palette::GRAY_1,
    );
    theme.set_color(
        &local_tokens::PANE_BORDER.to_string(),
        feathers::palette::WARM_GRAY_1,
    );
    theme.set_color(
        &local_tokens::SLIDER_ACTIVE.to_string(),
        feathers::palette::ACCENT.lighter(0.2),
    );
}
