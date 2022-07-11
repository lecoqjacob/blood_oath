use super::*;

pub fn build_entrance(ecs: &mut World) -> Layer {
    let mut layer = Layer::new(std::usize::MAX, ecs); // Gets a default layer

    all_space(&mut layer);
    add_landscape(&mut layer);
    add_docking_capsule(&mut layer, ecs);

    layer
}

fn add_docking_capsule(map: &mut Layer, ecs: &mut World) {
    const MIDDLE: usize = HEIGHT / 2;
    const TOP: usize = MIDDLE - 3;
    const BOTTOM: usize = MIDDLE + 3;
    const LEFT: usize = 1;
    const RIGHT: usize = 8;

    // Floor
    for y in TOP..=BOTTOM {
        for x in LEFT..=RIGHT {
            let idx = map.point2d_to_index(Point::new(x, y));
            map.tiles[idx] = Tile::capsule_floor();
        }
    }

    // Encasing Walls
    for x in LEFT - 1..=RIGHT + 1 {
        let idx = map.point2d_to_index(Point::new(x, TOP - 1));
        map.tiles[idx] = Tile::capsule_wall();
        let idx = map.point2d_to_index(Point::new(x, BOTTOM + 1));
        map.tiles[idx] = Tile::capsule_wall();
    }
    for y in TOP - 1..=BOTTOM + 1 {
        let idx = map.point2d_to_index(Point::new(LEFT - 1, y));
        map.tiles[idx] = Tile::capsule_wall();
        let idx = map.point2d_to_index(Point::new(RIGHT + 1, y));
        map.tiles[idx] = Tile::capsule_wall();
    }

    // Add some windows
    let x_middle = (LEFT + RIGHT) / 2;
    let idx = map.point2d_to_index(Point::new(x_middle - 2, TOP - 1));
    map.tiles[idx] = Tile::capsule_window();
    let idx = map.point2d_to_index(Point::new(x_middle - 2, BOTTOM + 1));
    map.tiles[idx] = Tile::capsule_window();
    let idx = map.point2d_to_index(Point::new(x_middle + 2, TOP - 1));
    map.tiles[idx] = Tile::capsule_window();
    let idx = map.point2d_to_index(Point::new(x_middle + 2, BOTTOM + 1));
    map.tiles[idx] = Tile::capsule_window();

    // Spawn the game exit
    add_game_exit(map, ecs, Point::new(LEFT - 1, MIDDLE));

    // Start adding in building complex features
    add_door(map, ecs, Point::new(RIGHT + 1, MIDDLE));

    map.starting_point = Point::new(LEFT + 1, MIDDLE);
}

fn add_game_exit(map: &mut Layer, ecs: &mut World, pt: Point) {
    let exit_idx = map.point2d_to_index(pt);
    map.tiles[exit_idx] = Tile::game_over();

    ecs.push((
        Position::with_pt(pt, 0),
        Description(
            "Exit to SecBot's Ship. Leave through here when you are ready to call it game over."
                .to_string(),
        ),
        TileTrigger(crate::components::TriggerType::EndGame),
    ));
}

fn add_landscape(map: &mut Layer) {
    let mut rng = crate::RNG.lock();
    let mut noise = FastNoise::seeded(rng.next_u64());

    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(10);
    noise.set_fractal_gain(0.5);
    noise.set_fractal_lacunarity(3.5);
    noise.set_frequency(0.02);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let h = noise.get_noise(x as f32, y as f32);
            let idx = map.point2d_to_index(Point::new(x, y));
            map.tiles[idx] = Tile::alien_landscape(h);
        }
    }
}

fn add_door(map: &mut Layer, ecs: &mut World, pt: Point) {
    let idx = map.point2d_to_index(pt);
    ecs.push((
        Position::with_pt(pt, 0),
        Description("A heavy, steel door.".to_string()),
        Glyph {
            glyph: to_cp437('+'),
            color: ColorPair::new(CYAN, BLACK),
        },
        Door {},
    ));
    map.tiles[idx] = Tile::wall();
    map.is_door[idx] = true;
}