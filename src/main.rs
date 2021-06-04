extern crate yaml_rust;

use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::time;

use yaml_rust::{YamlLoader, Yaml};

use bevy::prelude::*;
use bevy_asset::*;

fn read_config() -> Vec<Yaml> {
    let mut file = File::open("config.yml").expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    let yaml = YamlLoader::load_from_str(&contents).unwrap();
    
    return yaml;
}

struct HitObject {
    col: i64,
    time: i64,
    init_time: i64,
}

struct OsuFile {
    keys: i64,
    data: Vec<HitObject>,
    music_filename: String, 
}

fn read_osu(filename: &str, height: i64, scroll_speed: i64) -> OsuFile {
    let mut file = File::open(filename).expect("Unable to open .osu file");
    let mut contents = String::new();
    
    file.read_to_string(&mut contents); // contents works here
    
    let mut lines = contents.lines();
    
    let mut data: Vec<HitObject> = vec![];
    let mut audio_filename: String = "".to_string();
    let mut keys: i64 = 4;
    let mut hitobject_section: bool = false;
    
    loop {
        let ln = lines.next();
        let mut lin: &str = "";
        match ln {
            Some(l) => lin = l,
            None => println!("blank line!"),
        }
        let line: String = lin.to_string();
        if line.contains("AudioFilename:") {
            println!("found audio filename: {}", line);
            audio_filename = line[15..line.len()].to_string();
        }
        if line.contains("CircleSize:") {
            println!("found circle size: {}", line);
            keys = line[11..12].parse().unwrap();
        }
        if line.contains("[HitObjects]") {
            println!("found hitobject section: {}", line);
            hitobject_section = true;
            continue;
        }
        if hitobject_section {
            match ln {
                Some(_) => (),
                None => break,
            }
            
            let split = line.split(",");
            let vec: Vec<&str> = split.collect();
            let x: i64 = vec[0].parse().unwrap();
            let column: i64 = (x * keys / 512);
            
            let time: i64 = vec[2].parse().unwrap();
            let init_time: i64 = (time as f64 - ((height as f64 - 40.0) / scroll_speed as f64 * 1000.0)) as i64;
            let hitobject: HitObject = HitObject { col: column, time: time, init_time: init_time};
            
            println!("hitobject on column {}: init_time={}, hit_time={}", column, init_time, time);
            data.push(hitobject);
        }
    }
    let osufile: OsuFile = OsuFile {
        keys: keys,
        data: data,
        music_filename: audio_filename,
    };
    return osufile
}

pub struct GamePlugin;

// define all resources
struct GameData {
    height: i64,
    width: i64,
    offset: i64,
    data: OsuFile,
    scroll: i64,
}

fn game_loop(mut game_data: ResMut<GameData>, mut char_input_events: EventReader<ReceivedCharacter>) {

    for event in char_input_events.iter() {
        println!("{:?}: '{}'", event, event.char);
    }
}

fn start_audio(game_data: Res<GameData>, mut asset_server: ResMut<AssetServer>, audio: Res<Audio>) {
    //let mut asset_loader = AssetLoader();
    //asset_server.add_loader(asset_loader);
    let delay_time = time::Duration::from_millis(game_data.offset as u64);
    let music_filename: &str = &game_data.data.music_filename[..];
    println!("music_filename: {}", music_filename);
    let music = asset_server.load(music_filename);
    thread::sleep(delay_time);
    audio.play(music);
}

fn setup_field(game_data: Res<GameData>, mut commands: Commands, asset_server: Res<AssetServer>, mut materials: ResMut<Assets<ColorMaterial>>){
    let base_constant = (-(game_data.data.keys * 80) as f32 * 0.5 + 40.0) as f32;
    for i in 0..game_data.data.keys {
    let col_bottom_img = asset_server.load("column_bottom.png");
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());
        commands.spawn_bundle(SpriteBundle {
            material: materials.add(col_bottom_img.into()),
            transform: Transform::from_translation(Vec3::new(base_constant + 80.0 * i as f32, -(0.5 * game_data.height as f64 - 40.0) as f32, 0.0)),
            ..Default::default()
        });
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(start_audio.system());
        app.add_system(setup_field.system());
        app.add_system(game_loop.system());
    }
}

fn main() {
    let config = &read_config()[0];
    
    let height = config["window"][0].as_i64().unwrap();
    let width = config["window"][1].as_i64().unwrap();
    let offset = config["offset"][0].as_i64().unwrap();
    let filename = config["filename"][0].as_str().unwrap();
    let scroll = config["scroll"][0].as_i64().unwrap();
    
    println!("h, w: {}, {}; offset={}, scroll={}", height, width, offset, scroll);
    let data = read_osu(filename, height, scroll);
    
    let game_data: GameData = GameData {height: height, width: width, offset: offset, data: data, scroll: scroll};
    
    App::build()
        .insert_resource(WindowDescriptor{
            title: "rust-4k".to_string(),
            width: width as f32,
            height: height as f32,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(game_data)
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
    
    println!("window dimensions: ({}, {})", height, width);
}
