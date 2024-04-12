#![allow(clippy::needless_pass_by_value)]

use bevy::prelude::*;

mod balls;
mod common;
mod main_menu;
mod sph;

const BACKGROUND_COLOUR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    MainMenu,
    BallSimulation,
    SPHSimulation,
}

pub struct AlephPlugin;

fn global_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

impl Plugin for AlephPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOUR))
            .init_state::<AppState>()
            .add_systems(Startup, global_setup)
            // Main Menu
            .add_systems(OnEnter(AppState::MainMenu), main_menu::setup)
            .add_systems(OnExit(AppState::MainMenu), main_menu::destroy)
            .add_systems(
                Update,
                main_menu::handle_input.run_if(in_state(AppState::MainMenu)),
            )
            // Ball Simulation
            .add_systems(OnEnter(AppState::BallSimulation), balls::setup)
            .add_systems(OnExit(AppState::BallSimulation), balls::destroy)
            .add_systems(
                Update,
                (balls::handle_mouse_events, balls::handle_interaction)
                    .run_if(in_state(AppState::BallSimulation)),
            )
            .add_systems(
                FixedUpdate,
                (
                    common::apply_velocity,
                    common::apply_gravity,
                    common::check_for_boundary_collisions,
                    balls::check_for_ball_collisions,
                )
                    .chain()
                    .run_if(in_state(AppState::BallSimulation)),
            ); // TODO: SPH Simulation
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins, AlephPlugin)).run();
}
