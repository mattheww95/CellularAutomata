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

const PREY_HEALTH: u16 = 1;
//const PREY_SPLIT: u16 = 5;
const PREY_SPLIT: u16 = 15;
//const PREDATOR_HEALTH: u16 = 10;
const PREDATOR_HEALTH: u16 = 5;
const ENVIRONMENT_HEALTH: u16 = 0;

impl Boid {
    fn change(&mut self, bt: BoidType) {
        match bt {
            BoidType::Prey => {
                self.boid_type = bt;
                self.health = PREY_HEALTH;
                self.colour = GREEN
            }
            BoidType::Predator => {
                self.boid_type = bt;
                self.health = PREDATOR_HEALTH;
                self.colour = RED
            }
            BoidType::Environment => {
                self.boid_type = bt;
                self.health = ENVIRONMENT_HEALTH;
                self.colour = BLACK;
            }
        };
    }

    fn new_boid(bt: BoidType) -> Boid {
        match bt {
            BoidType::Prey => Boid {
                boid_type: bt,
                health: PREY_HEALTH,
                colour: GREEN,
            },
            BoidType::Predator => Boid {
                boid_type: bt,
                health: PREDATOR_HEALTH,
                colour: RED,
            },
            BoidType::Environment => Boid {
                boid_type: bt,
                health: ENVIRONMENT_HEALTH,
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

#[macroquad::main("PredatorVsPrey")]
async fn main() {
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
    let prey_percent = 0.10;
    let predator_percent = 0.01;

    // Populate map
    let mut map: Vec<Vec<Boid>> = Vec::with_capacity(h);
    for _ in 0..h {
        map.push(vec![Boid::new_boid(BoidType::Environment); w]);
    }

    // Create establish number of predators and prey
    let number_of_prey: u64 = (map_size as f64 * prey_percent) as u64;
    let number_of_predator: u64 = (map_size as f64 * predator_percent) as u64;

    let mut image = Image::gen_image_color(w as u16, h as u16, BLACK);
    let texture = Texture2D::from_image(&image);

    for _ in 0..number_of_prey {
        let cx = RandomRange::gen_range(0, w - 1);
        let cy = RandomRange::gen_range(0, h - 1);
        map[cy][cx].change(BoidType::Prey);
    }

    for _ in 0..number_of_predator {
        let cx = RandomRange::gen_range(0, w - 1);
        let cy = RandomRange::gen_range(0, h - 1);
        map[cy][cx].change(BoidType::Predator);
    }

    loop {
        // Add game logic here
        // Predator and prey lose a life every move, and the move a random direction
        // each time
        //
        // Update Predators first, if predators have a prey right next to them
        // eat the prey, and that prey becomes a predator, otherwise move in a
        // random direction.
        //
        // Prey moves in a random directin each turn if there is not path blocked. If there
        // are 3 empty spaces around them, then they spawn a new prey and move.
        //

        // Predators first
        for y in 0..map.len() {
            for x in 0..map[y].len() {
                match map[y][x].boid_type {
                    BoidType::Predator => {
                        let mut safe_directions: Vec<(isize, isize)> =
                            Vec::with_capacity(directions.len());
                        if map[y][x].health == 1 {
                            map[y][x].change(BoidType::Environment);
                            continue;
                        }
                        map[y][x].health -= 1;
                        // Check if any prey are nearby
                        for dir in directions.iter() {
                            let new_y = dir.0 + y as isize;
                            let new_x = dir.1 + x as isize;
                            if 0 <= new_y && new_y < h as isize && 0 <= new_x && new_x < w as isize
                            {
                                match map[new_y as usize][new_x as usize].boid_type {
                                    BoidType::Prey => {
                                        map[y][x].health +=
                                            map[new_y as usize][new_x as usize].health;
                                        map[new_y as usize][new_x as usize]
                                            .change(BoidType::Predator);
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
                            map[newy][newx].boid_type = BoidType::Predator;
                            map[newy][newx].health = map[y][x].health;
                            map[newy][newx].colour = RED;
                            map[newy][newx] = Boid { ..map[y][x] };
                            map[y][x].change(BoidType::Environment);
                        }
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
                                let update_val = map[new_y as usize][new_x as usize];
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
                            continue;
                        }
                        //let rand_idx = RandomRange::gen_range(0, safe_directions.len());
                        let rand_idx = rand() % directions.len();
                        let new_dir = directions[rand_idx];
                        if !safe_directions.contains(&new_dir) {
                            continue;
                        }
                        let newy = (y as isize + new_dir.0) as usize;
                        let newx = (x as isize + new_dir.1) as usize;
                        // Create another prey boid if ther are 2, and safe directions are greater than 2
                        if map[y][x].health >= PREY_SPLIT {
                            // Flip a coin breed or move
                            map[newy][newx].change(BoidType::Prey);
                            map[y][x].health = PREY_HEALTH;
                        } else {
                            map[newy][newx].boid_type = BoidType::Prey;
                            map[newy][newx].health = map[y][x].health;
                            map[newy][newx].colour = GREEN;
                            map[y][x].change(BoidType::Environment);
                        }
                    }
                    BoidType::Environment => continue,
                }
            }
        }
        let mut predators = 0;
        let mut prey = 0;
        for y in 0..map.len() {
            for x in 0..map[y].len() {
                image.set_pixel(x as u32, y as u32, map[y][x].colour);
                match map[y][x].boid_type {
                    BoidType::Prey => prey += 1,
                    BoidType::Predator => predators += 1,
                    _ => continue,
                }
            }
        }
        texture.update(&image);
        draw_texture(&texture, 0., 0., WHITE);
        draw_text_ex(
            &format!("Predators: {} Prey: {}", predators, prey),
            30.0,
            30.0,
            TextParams {
                color: WHITE,
                ..TextParams::default()
            },
        );
        next_frame().await;
    }
}
