use avian2d::dynamics::solver::SolverConfig;
use avian2d::prelude::*;
use bevy::prelude::*;
use rand::RngExt;

const TILE_DIMENSION: u32 = 640;
const ATLAS_ROWS: u32 = 9;
const ATLAS_COLUMNS: u32 = 8;
const USE_COLUMN: usize = 4;
const START_RADIUS: f32 = 25.0;
const RESTITUTION: f32 = 0.3;
const SPAWN_LIMIT: f32 = 200.0;
const OUT_OF_BOUNDS: f32 = -400.0;

#[derive(Component)]
struct Ball {
    index: usize,
}

#[derive(Component)]
struct Staged;

#[derive(Resource)]
struct EmojiAtlas {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    count: usize,
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Line;

#[derive(Component)]
struct UIText;

#[derive(Resource)]
struct Score(i32);

#[derive(Message)]
struct IncreaseScore(i32);

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(600.0, 10.0),
        Transform::from_xyz(0.0, -350.0, 0.0),
        Mesh2d(meshes.add(Rectangle::new(600.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.9, 0.95))),
        Wall,
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(600.0, 10.0),
        Transform {
            translation: Vec3::new(300.0, -55.0, 0.0),
            rotation: Quat::from_rotation_z(90.0_f32.to_radians()),
            ..default()
        },
        Mesh2d(meshes.add(Rectangle::new(600.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.9, 0.95))),
        Wall,
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(600.0, 10.0),
        Transform {
            translation: Vec3::new(-300.0, -55.0, 0.0),
            rotation: Quat::from_rotation_z(90.0_f32.to_radians()),
            ..default()
        },
        Mesh2d(meshes.add(Rectangle::new(600.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.9, 0.95))),
        Wall,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(590.0, 2.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 200.0, 0.0),
        Line,
    ));
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        UIText,
    ));
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("emoji-atlas.png");

    let layout = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::new(TILE_DIMENSION, TILE_DIMENSION),
        ATLAS_COLUMNS,
        ATLAS_ROWS,
        None,
        None,
    ));

    commands.insert_resource(EmojiAtlas {
        texture,
        layout,
        count: (ATLAS_ROWS * ATLAS_COLUMNS) as usize,
    });
}

fn stage_ball(mut commands: Commands, emoji_atlas: Res<EmojiAtlas>) {
    let mut rng = rand::rng();

    let index = rng.random_range(1..=3); // Some variance when spawning balls

    let atlas_index = USE_COLUMN + ((index - 1) * ATLAS_COLUMNS as usize);
    let radius = (START_RADIUS * index as f32) * 0.66;

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..Sprite::from_atlas_image(
                emoji_atlas.texture.clone(),
                TextureAtlas {
                    layout: emoji_atlas.layout.clone(),
                    index: atlas_index,
                },
            )
        },
        Collider::circle(radius),
        Restitution::new(RESTITUTION),
        RigidBody::Static,
        Staged,
        Transform::from_xyz(-450.0, 200.0, 0.0),
        Ball { index },
    ));
}

fn drop_ball(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    ball_query: Single<Entity, With<Staged>>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    if world_pos.y < SPAWN_LIMIT {
        return;
    }

    let entity = ball_query.into_inner();

    commands.entity(entity).insert((
        RigidBody::Dynamic,
        Transform::from_translation(world_pos.extend(0.0)),
        CollisionEventsEnabled,
    ));

    commands.entity(entity).remove::<Staged>();

    commands.run_system_cached(stage_ball);
}

fn despawn_ball(
    mut commands: Commands,
    balls: Query<(&Transform, Entity), With<Ball>>,
    mut writer: MessageWriter<IncreaseScore>,
) {
    for (transform, entity) in balls {
        if transform.translation.y < OUT_OF_BOUNDS {
            commands.entity(entity).despawn();
            writer.write(IncreaseScore(-50));
        }
    }
}

fn merge(
    mut commands: Commands,
    mut collisions: MessageReader<CollisionStart>,
    balls: Query<(&Ball, &Transform, Entity)>,
    emoji_atlas: Res<EmojiAtlas>,
    mut writer: MessageWriter<IncreaseScore>,
) {
    for collision in collisions.read() {
        if let Ok(ball1) = balls.get(collision.collider1)
            && let Ok(ball2) = balls.get(collision.collider2)
        {
            let (ball1, t1, entity1) = ball1;
            let (ball2, t2, entity2) = ball2;

            if ball1.index != ball2.index {
                return;
            }

            let index = ball1.index + 1;
            let atlas_index = USE_COLUMN + ((index - 1) * ATLAS_COLUMNS as usize);
            let radius = (START_RADIUS * index as f32) * 0.66;

            let x_pos = (t1.translation.x + t2.translation.x) / 2.0;
            let y_pos = (t1.translation.y + t2.translation.y) / 2.0;

            if index < emoji_atlas.count {
                commands.spawn((
                    RigidBody::Dynamic,
                    Collider::circle(radius),
                    Transform::from_translation(Vec3::new(x_pos, y_pos, 0.0)),
                    Restitution::new(RESTITUTION),
                    Sprite {
                        custom_size: Some(Vec2::splat(radius * 2.0)),
                        ..Sprite::from_atlas_image(
                            emoji_atlas.texture.clone(),
                            TextureAtlas {
                                layout: emoji_atlas.layout.clone(),
                                index: atlas_index,
                            },
                        )
                    },
                    CollisionEventsEnabled,
                    Ball { index },
                ));
            }

            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();

            writer.write(IncreaseScore(index as i32 * 10));
        }
    }
}

fn update_score(mut reader: MessageReader<IncreaseScore>, mut score: ResMut<Score>) {
    for message in reader.read() {
        score.0 += message.0;
    }
}

fn update_ui(mut span: Single<&mut Text, With<UIText>>, score: Res<Score>) {
    **span = format!("Score: {}", score.0).into();
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin,
        ))
        .insert_resource(Gravity(Vec2::new(0.0, -600.0)))
        .insert_resource(SolverConfig {
            max_overlap_solve_speed: 50.0,
            ..default()
        })
        .insert_resource(Score(0))
        .add_message::<IncreaseScore>()
        .add_systems(
            Startup,
            (load_assets, setup_scene, stage_ball, setup_ui, setup_camera).chain(),
        )
        .add_systems(
            Update,
            (despawn_ball, merge, drop_ball, update_score, update_ui),
        )
        .run();
}
