use clap::Parser;
use macroquad::prelude::*;
use macroquad::rand::RandomRange;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum BoidType {
    Predator,
    Prey,
    Environment,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Boid {
    boid_type: BoidType,
    health: u16,
    colour: Color,
}

struct ArgsDefaults {
    prey_health: u16,
    predator_health: u16,
    environment_health: u16,
}

impl Boid {
    fn change(&mut self, bt: BoidType, defaults: &ArgsDefaults) {
        match bt {
            BoidType::Prey => {
                self.boid_type = bt;
                self.health = defaults.prey_health;
                self.colour = GREEN
            }
            BoidType::Predator => {
                self.boid_type = bt;
                self.health = defaults.predator_health;
                self.colour = RED
            }
            BoidType::Environment => {
                self.boid_type = bt;
                self.health = defaults.environment_health;
                self.colour = BLACK;
            }
        };
    }

    fn new_boid(bt: BoidType, defaults: &ArgsDefaults) -> Boid {
        match bt {
            BoidType::Prey => Boid {
                boid_type: bt,
                health: defaults.prey_health,
                colour: GREEN,
            },
            BoidType::Predator => Boid {
                boid_type: bt,
                health: defaults.predator_health,
                colour: RED,
            },
            BoidType::Environment => Boid {
                boid_type: bt,
                health: defaults.environment_health,
                colour: BLACK,
            },
        }
    }
}

static mut SEED: usize = 42;

unsafe fn custom_rand() -> usize {
    SEED = SEED.wrapping_mul(1103515245) + 12345;
    return (SEED / 65536) % 32768;
}

fn rand() -> usize {
    unsafe { custom_rand() }
}

fn populate_map(
    num: u64,
    map: &mut Vec<Vec<Boid>>,
    bt: BoidType,
    width: usize,
    height: usize,
    defs: &ArgsDefaults,
) -> () {
    for _ in 0..num {
        let cx = RandomRange::gen_range(0, width - 1);
        let cy = RandomRange::gen_range(0, height - 1);
        map[cy][cx].change(bt, defs);
    }
}

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    #[clap(long, short = 'p', default_value_t = 1)]
    prey_health: u16,

    #[clap(long, short = 's', default_value_t = 3)]
    prey_split: u16,

    #[clap(long, short = 'h', default_value_t = 5)]
    predator_health: u16,

    #[clap(long, short = 'r', default_value_t = 0.001)]
    predators: f64,

    #[clap(long, short = 'y', default_value_t = 0.01)]
    prey: f64,
}

#[macroquad::main("PredatorVsPrey")]
async fn main() {
    let args = Cli::parse();

    let prey_health: u16 = args.prey_health;
    let prey_split: u16 = args.prey_split;
    let predator_health: u16 = args.predator_health;
    let environment_health: u16 = 0;

    let defaults = ArgsDefaults {
        prey_health,
        predator_health,
        environment_health,
    };

    // N NE E SE S SW W NW
    // N 1, 0
    // NE 1, 1
    // E 0, 1
    // SE -1, 1
    // S -1, 0
    // SW -1, -1
    // W 0, -1
    // Position (y, x)
    let directions: Vec<(isize, isize)> =
        vec![(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1)];

    let w = screen_width() as usize;
    let h = screen_height() as usize;

    let map_size = w * h;

    // Set set the number of values to add
    let prey_percent = args.prey;
    let predator_percent = args.predators;

    // Populate map
    let mut map1: Vec<Vec<Boid>> = Vec::with_capacity(h);
    for _ in 0..h {
        map1.push(vec![Boid::new_boid(BoidType::Environment, &defaults); w]);
    }
    let mut map2 = map1.clone(); // Copy the map for updating alternate

    // Create establish number of predators and prey
    let number_of_prey: u64 = (map_size as f64 * prey_percent) as u64;
    let number_of_predator: u64 = (map_size as f64 * predator_percent) as u64;

    let mut image = Image::gen_image_color(w as u16, h as u16, BLACK);
    let texture = Texture2D::from_image(&image);
    populate_map(number_of_prey, &mut map1, BoidType::Prey, w, h, &defaults);
    populate_map(
        number_of_predator,
        &mut map1,
        BoidType::Predator,
        w,
        h,
        &defaults,
    );

    let mut map_use: bool = false;
    let mut safe_directions: Vec<(isize, isize)> = Vec::with_capacity(directions.len());

    let mut map = &mut map1;
    let mut map_u = &mut map2;
    let mut iterations: u64 = 0;
    loop {
        // Add game logic here
        //-For prey:
        //    -Tries to move in a random direction.
        //    -Health increases.
        //    -When health reaches a threshold:
        //        -They will reproduce, creating a new "Prey"
        //        -Their health resets to 1
        //
        //-For predator:
        //    -Tries to move in a random direction.
        //    -Health decreases.
        //    -When health reaches 0, they die and turn into "Nothing".
        //    -If the adjacent square is a prey:
        //        -They will eat it, turning it into a "predator" (reproducing)
        //        -Their health will increase by the amount of health the eaten prey had
        //

        if map_use {
            map = &mut map2;
            map_u = &mut map1;
        } else {
            map = &mut map1;
            map_u = &mut map2;
        }
        // Predators first
        for y in 0..map.len() {
            for x in 0..map[y].len() {
                match map[y][x].boid_type {
                    BoidType::Predator => {
                        if map[y][x].health == 1 {
                            map_u[y][x].change(BoidType::Environment, &defaults);
                            map[y][x].change(BoidType::Environment, &defaults);
                            continue;
                        }
                        map[y][x].health -= 1;
                        let mut fed: bool = false;
                        // Check if any prey are nearby
                        for dir in directions.iter() {
                            let new_y = dir.0 + y as isize;
                            let new_x = dir.1 + x as isize;
                            if 0 <= new_y && new_y < h as isize && 0 <= new_x && new_x < w as isize
                            {
                                match map[new_y as usize][new_x as usize].boid_type {
                                    BoidType::Prey => {
                                        if fed {
                                            // Predators can only eat once per a turn
                                            continue;
                                        }
                                        fed = true;
                                        map[y][x].health +=
                                            map[new_y as usize][new_x as usize].health;
                                        map_u[new_y as usize][new_x as usize]
                                            .change(BoidType::Predator, &defaults);
                                        map[new_y as usize][new_x as usize]
                                            .change(BoidType::Environment, &defaults);
                                    }
                                    BoidType::Environment => {
                                        safe_directions.push((dir.0, dir.1));
                                    }
                                    BoidType::Predator => {
                                        continue;
                                    }
                                }
                            }
                        }

                        let rand_idx = rand() as usize % directions.len();
                        let new_dir = directions[rand_idx];
                        if safe_directions.contains(&new_dir) {
                            let newy = (y as isize + new_dir.0) as usize;
                            let newx = (x as isize + new_dir.1) as usize;
                            map_u[newy][newx].boid_type = BoidType::Predator;
                            map_u[newy][newx].health = map[y][x].health;
                            map_u[newy][newx].colour = RED;
                        } else {
                            map_u[y][x].boid_type = BoidType::Predator;
                            map_u[y][x].health = map[y][x].health;
                            map_u[y][x].colour = RED;
                        }
                        map[y][x].change(BoidType::Environment, &defaults);
                    }
                    BoidType::Prey => {
                        map[y][x].health += 1;
                        let mut safe_directions: Vec<(isize, isize)> =
                            Vec::with_capacity(directions.len());
                        // Check if any prey are nearby
                        for dir in directions.iter() {
                            let new_y = dir.0 + y as isize;
                            let new_x = dir.1 + x as isize;
                            if 0 <= new_y && new_y < h as isize && 0 <= new_x && new_x < w as isize
                            {
                                let update_val = map_u[new_y as usize][new_x as usize];
                                match update_val.boid_type {
                                    BoidType::Prey => {
                                        continue;
                                    }
                                    BoidType::Environment => {
                                        safe_directions.push((dir.0, dir.1));
                                    }
                                    BoidType::Predator => {
                                        continue;
                                    }
                                }
                            }
                        }

                        if safe_directions.len() == 0 {
                            map_u[y][x].boid_type = BoidType::Prey;
                            map_u[y][x].health = map[y][x].health;
                            map_u[y][x].colour = GREEN;
                            continue;
                        }

                        let rand_idx = rand() % directions.len();
                        let new_dir = directions[rand_idx];
                        if !safe_directions.contains(&new_dir) {
                            map_u[y][x].boid_type = BoidType::Prey;
                            map_u[y][x].health = map[y][x].health;
                            map_u[y][x].colour = GREEN;
                            continue;
                        }

                        let newy = (y as isize + new_dir.0) as usize;
                        let newx = (x as isize + new_dir.1) as usize;
                        // Create another prey boid if ther are 2, and safe directions are greater than 2
                        if map[y][x].health >= prey_split {
                            map_u[newy][newx].change(BoidType::Prey, &defaults);
                            map_u[y][x].change(BoidType::Prey, &defaults);
                            map[y][x].change(BoidType::Environment, &defaults);
                        } else {
                            map_u[newy][newx].boid_type = BoidType::Prey;
                            map_u[newy][newx].health = map[y][x].health;
                            map_u[newy][newx].colour = GREEN;
                            map[y][x].change(BoidType::Environment, &defaults);
                        }
                    }
                    BoidType::Environment => continue,
                }
                safe_directions.clear();
            }
        }
        let mut predators = 0;
        let mut prey = 0;
        for y in 0..map_u.len() {
            for x in 0..map_u[y].len() {
                image.set_pixel(x as u32, y as u32, map_u[y][x].colour);
                match map_u[y][x].boid_type {
                    BoidType::Prey => prey += 1,
                    BoidType::Predator => predators += 1,
                    _ => continue,
                }
            }
        }
        if predators == 0 || prey == 0 {
            println!("Species went extinct after {} iterations.", iterations);
            break;
        }
        texture.update(&image);
        draw_texture(&texture, 0., 0., WHITE);
        draw_text_ex(
            &format!("Predators: {}", predators),
            30.0,
            30.0,
            TextParams {
                color: WHITE,
                ..TextParams::default()
            },
        );
        draw_text_ex(
            &format!("Prey: {}", prey),
            30.0,
            50.0,
            TextParams {
                color: WHITE,
                ..TextParams::default()
            },
        );
        draw_text_ex(
            &format!("Iterations: {}", iterations),
            30.0,
            70.0,
            TextParams {
                color: WHITE,
                ..TextParams::default()
            },
        );

        map_use = !map_use;
        iterations += 1;
        next_frame().await;
    }
}
