use bevy::{
    input::ButtonInput, prelude::*, reflect::Map
};
use std::collections::HashMap;

#[derive(Resource)]
struct SnakeEM(HashMap<Entity, HashMap<i32, Entity>>);

#[derive(Resource)]
struct SnakeIE(HashMap<i32, Entity>);

#[derive(Resource)]
struct CursorPos(Vec2);

#[derive(Component)]
struct CameraEntityInfos {
    visionview: f32,
}

#[derive(Component)]
struct SnakeEntityInfos {
    piece: i32,
    color: Srgba,
    health: f32,
    vision: f32,
    speed: f32,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Piece;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(SnakeEM(HashMap::new()))
    .insert_resource(SnakeIE(HashMap::new()))
    .insert_resource(CursorPos(Vec2::ZERO))
    .add_systems(Startup, setup)
    .add_systems(Update, (get_cursor, plmvmnt, spawn_tail, tail_mvmnt))
    .run();
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sn_em: ResMut<SnakeEM>,
    mut sn_ie: ResMut<SnakeIE>,
) {
    cmd.spawn(Camera2d)
    .insert(CameraEntityInfos {
        visionview: 10.0,
    });

    let pl_en = cmd.spawn(Transform::from_xyz(0.0, 0.0, 0.0))
    .insert(Mesh2d(meshes.add(Capsule2d::new(20.0, 50.0))))
    .insert(MeshMaterial2d(materials.add(Color::srgba(0.0, 8.0, 0.0, 1.0))))
    .insert(SnakeEntityInfos {
        piece: 0,
        color: Srgba::new(0.0, 8.0, 0.0, 1.0),
            health: 100.0,
            vision: 10.0,
            speed: 20.0,
    })
    .insert(Player)
    .id();

    let mut inner_map = HashMap::new();
    inner_map.insert(0, pl_en);

    sn_em.0.insert(pl_en, inner_map);
}

fn tail_mvmnt(
    mut sn_em: ResMut<SnakeEM>,
    mut query: Query<&Player>,
    mut tquery: Query<&mut Transform>,
    mut squery: Query<&mut SnakeEntityInfos>,
    time: Res<Time>,
) {
    let mut pl_en = Vec::new();

    for (entity, nested_map) in sn_em.0.iter_mut() {
        for (_, nested_entity) in nested_map.iter() {
            if query.get(*nested_entity).is_ok() {
                pl_en.push(*nested_entity);
            }
        }
    }

    for (outer_key, inner_map) in &sn_em.0 {
        for (inner_key, inner_value) in inner_map {
            if *inner_key < 0 {
                continue;
            }

            let mut prev_pos = None;
            let mut snake_speed = 0.0;

            if let Some(prev_entity) = inner_map.get(&(inner_key - 1)) {
                if let Ok(prev_transform) = tquery.get(*prev_entity) {
                    prev_pos = Some(prev_transform.translation);
                }
            }

            let this_pos = match tquery.get(*inner_value) {
                Ok(transform) => transform.translation,
                Err(_) => continue,
            };

            if let Ok(snakeinfo) = squery.get(*inner_value) {
                snake_speed = snakeinfo.speed;
            }

            if let Some(prev_pos) = prev_pos {
                let distance = this_pos.distance(prev_pos);
                let base_speed = 50.0;
                let speed_factor = distance * 0.1;
                let speed = f32::clamp(base_speed + speed_factor, 10.0, 100.0);

                if let Ok(mut transform) = tquery.get_mut(*inner_value) {
                    let rotation_speed: f32 = 90.0;
                    let rotation_radians = rotation_speed.to_radians() * time.delta_secs();
                    transform.rotation = Quat::from_rotation_z(rotation_radians) * transform.rotation;

                    let movement_direction = transform.rotation * Vec3::Y;
                    let movement_distance = snake_speed * time.delta_secs();
                    let translation_delta = movement_direction * movement_distance;
                    transform.translation += translation_delta;
                }
            }
        }
    }
}


fn spawn_tail(
    mut cmd: Commands,
    kb: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sn_em: ResMut<SnakeEM>,
    mut query: Query<&Player>
) {
    if kb.just_pressed(KeyCode::Space) {
        let mut pl_en = Vec::new();

        for (entity, nested_map) in sn_em.0.iter_mut() {
            for (_, nested_entity) in nested_map.iter() {
                if query.get(*nested_entity).is_ok() {
                    pl_en.push(*nested_entity);
                }
            }
        }

        let bd_en = cmd.spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(Mesh2d(meshes.add(Capsule2d::new(20.0, 50.0))))
        .insert(MeshMaterial2d(materials.add(Color::srgba(0.0, 8.0, 0.0, 1.0))))
        .insert(SnakeEntityInfos {
            piece: 0,
            color: Srgba::new(8.0, 0.0, 0.0, 1.0),
                health: 100.0,
                vision: 10.0,
                speed: 20.0,
        })
        .insert(Piece)
        .id();

        let len = sn_em.0.len().try_into().unwrap();

        if let Some(inner_map) = sn_em.0.get_mut(&pl_en[0]) {
            inner_map.insert(len, bd_en);
        }
    }
}

fn plmvmnt(
    kb: Res<ButtonInput<KeyCode>>,
    mb: Res<ButtonInput<MouseButton>>,
    mut pl_q: Query<(&mut SnakeEntityInfos, &mut Transform), With<Player>>,
           curpos: Res<CursorPos>,
           time: Res<Time>,
) {
    let threshold = 50.0;

    for (mut player, mut transform) in pl_q.iter_mut() {
        let pos: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);
        let target: Vec2 = Vec2::new(curpos.0.x, curpos.0.y);
        let direction = target - pos;

        if direction.length_squared() > 0.0 {
            let pi = std::f32::consts::PI;

            let true_angle = direction.y.atan2(direction.x);
            let current_angle = transform.rotation.to_euler(EulerRot::ZXY).0;

            let mut diff_angle = true_angle - pi / 2.0 - current_angle;

            if diff_angle > pi {
                diff_angle -= 2.0 * pi;
            } else if diff_angle < -pi {
                diff_angle += 2.0 * pi;
            }

            let change_angle = diff_angle * player.speed * time.delta_secs();

            transform.rotation = Quat::from_rotation_z(current_angle + change_angle);
        }

        if kb.pressed(KeyCode::ShiftLeft) {
            player.speed = 25.0;
        } else {
            player.speed = 10.0;
        }

        if mb.pressed(MouseButton::Left) {
            let cursor_pos = Vec3::new(curpos.0.x, curpos.0.y, 0.0);
            let distance_to_cursor = transform.translation.distance(cursor_pos);

            if distance_to_cursor > threshold {
                let direction = (cursor_pos - transform.translation).normalize();
                let move_distance = threshold * time.delta_secs() * player.speed;
                transform.translation += direction * move_distance;
            }
        }
    }
}

fn get_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
              windows: Query<&Window>,
              mut curpos: ResMut<CursorPos>,
) {
    let (camera, camera_transform) = *camera_query;
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else { return; };
    curpos.0 = point;
    println!("{:?}", curpos.0);
}
