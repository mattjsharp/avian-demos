use avian2d::prelude::*;
use bevy::prelude::*;
use rand::RngExt;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct UIText;

#[derive(Resource)]
struct BallRadius(f32);

#[derive(Resource)]
struct BallRestitution(f32);

#[derive(Resource)]
struct EmojiAtlas {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    count: usize,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(200.0, 10.0),
        Transform::from_xyz(0.0, -300.0, 0.0),
        Mesh2d(meshes.add(Rectangle::new(200.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.15, 0.25, 0.25))),
        Floor,
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(400.0, 10.0),
        Transform {
            translation: Vec3::new(150.0, -110.0, 0.0),
            rotation: Quat::from_rotation_z(70.0_f32.to_radians()),
            ..default()
        },
        Mesh2d(meshes.add(Rectangle::new(400.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.15, 0.25, 0.25))),
        Wall,
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(400.0, 10.0),
        Transform {
            translation: Vec3::new(-150.0, -110.0, 0.0),
            rotation: Quat::from_rotation_z(-70.0_f32.to_radians()),
            ..default()
        },
        Mesh2d(meshes.add(Rectangle::new(400.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.15, 0.25, 0.25))),
        Wall,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(600.0, 2.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 100.0, 0.0),
    ));

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
        UVec2::new(640, 640), // size of one emoji
        8,                    // columns
        9,                    // rows
        None,
        None,
    ));

    commands.insert_resource(EmojiAtlas {
        texture,
        layout,
        count: 8 * 9,
    });
}

fn spawn_ball(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    emoji_atlas: Res<EmojiAtlas>,
    restitution: Res<BallRestitution>,
    radius: Res<BallRadius>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    if world_pos.y < 100.0 {
        return;
    }

    let mut rng = rand::rng();

    let emoji_index = rng.random_range(0..emoji_atlas.count);

    commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(radius.0),
        Restitution::new(restitution.0),
        Transform::from_translation(world_pos.extend(0.0)),
        Sprite {
            custom_size: Some(Vec2::splat(radius.0 * 2.0)),
            ..Sprite::from_atlas_image(
                emoji_atlas.texture.clone(),
                TextureAtlas {
                    layout: emoji_atlas.layout.clone(),
                    index: emoji_index,
                },
            )
        },
        Ball,
    ));
}

fn despawn_ball(mut commands: Commands, balls: Query<(&Transform, Entity), With<Ball>>) {
    for (transform, entity) in balls {
        if transform.translation.y < -400.0 {
            commands.entity(entity).despawn();
        }

        if transform.translation.y > 400.0 {
            commands.entity(entity).despawn();
        }

        if transform.translation.x > 700.0 {
            commands.entity(entity).despawn();
        }

        if transform.translation.x < -700.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_ui(
    balls: Query<&Ball>,
    mut span: Single<&mut Text, With<UIText>>,
    gravity: Res<Gravity>,
    restitution: Res<BallRestitution>,
    radius: Res<BallRadius>,
) {
    **span = format!(
        "Balls: {}\nGravity (up/down/left/right): {:.2}\nRestitution (Q/A): {:.2}\nRadius (W/S): {:.2}",
        balls.count().to_string(),
        gravity.0,
        restitution.0,
        radius.0
    )
    .into();
}

fn update_gravity(
    mut gravity: ResMut<Gravity>,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let mut displacement = Vec2::default();

    if keycode.pressed(KeyCode::ArrowUp) {
        displacement.y -= 50.0 * dt;
    }
    if keycode.pressed(KeyCode::ArrowDown) {
        displacement.y += 50.0 * dt;
    }
    if keycode.pressed(KeyCode::ArrowLeft) {
        displacement.x += 50.0 * dt;
    }
    if keycode.pressed(KeyCode::ArrowRight) {
        displacement.x -= 50.0 * dt;
    }

    gravity.0 += displacement;
}

fn update_radius(
    mut radius: ResMut<BallRadius>,
    time: Res<Time>,
    keycode: Res<ButtonInput<KeyCode>>,
) {
    let dt = time.delta_secs();

    if keycode.pressed(KeyCode::KeyW) {
        radius.0 += 10.0 * dt;
    }

    if keycode.pressed(KeyCode::KeyS) {
        radius.0 -= 10.0 * dt;
    }

    if radius.0 < 10.0 {
        radius.0 = 10.0;
    }
}

fn update_restitution(
    mut restitution: ResMut<BallRestitution>,
    time: Res<Time>,
    keycode: Res<ButtonInput<KeyCode>>,
) {
    let dt = time.delta_secs();

    if keycode.pressed(KeyCode::KeyQ) {
        restitution.0 += 1.0 * dt;
    }

    if keycode.pressed(KeyCode::KeyA) {
        restitution.0 -= 1.0 * dt;
    }
}

fn toggle_floor_collision(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Single<(Entity, Option<&ColliderDisabled>), With<Floor>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let (entity, disabled) = query.into_inner();

    if disabled.is_some() {
        commands.entity(entity).remove::<ColliderDisabled>();
        commands
            .entity(entity)
            .insert(MeshMaterial2d(materials.add(Color::srgb(0.15, 0.25, 0.25))));
    } else {
        commands.entity(entity).insert(ColliderDisabled);
        commands
            .entity(entity)
            .insert(MeshMaterial2d(materials.add(Color::srgb(0.05, 0.12, 0.15))));
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin,
        ))
        .insert_resource(Gravity(Vec2::new(0.0, -600.0)))
        .insert_resource(BallRadius(10.0))
        .insert_resource(BallRestitution(0.0))
        .add_systems(Startup, (setup, load_assets))
        .add_systems(
            Update,
            (
                spawn_ball,
                despawn_ball,
                update_ui,
                toggle_floor_collision,
                update_gravity,
                update_restitution,
                update_radius,
            ),
        )
        .run();
}
