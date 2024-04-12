use bevy::prelude::*;

use crate::AppState;

const NORMAL_BUTTON_COLOUR: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON_COLOUR: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON_COLOUR: Color = Color::rgb(0.35, 0.75, 0.35);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans/FiraSans-Bold.ttf"),
        font_size: 20.0,
        color: Color::rgb(0.9, 0.9, 0.9),
    };

    let button_bundle = ButtonBundle {
        style: Style {
            width: Val::Px(200.),
            height: Val::Px(65.),
            border: UiRect::all(Val::Px(5.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: NORMAL_BUTTON_COLOUR.into(),
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(button_bundle.clone()).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Ball Simulation",
                    text_style.clone(),
                ));
            });
            parent.spawn(button_bundle.clone()).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "SPH Simulation",
                    text_style.clone(),
                ));
            });
        });
}

pub fn destroy(mut commands: Commands, query: Query<Entity, With<Node>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn handle_input(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    text_query: Query<&Text>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let text = text_query.get(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON_COLOUR.into();
                border_color.0 = Color::RED;

                match text.sections[0].value.as_str() {
                    "Ball Simulation" => {
                        next_state.set(AppState::BallSimulation);
                    }
                    "SPH Simulation" => {
                        next_state.set(AppState::SPHSimulation);
                    }
                    _ => {}
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON_COLOUR.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON_COLOUR.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
