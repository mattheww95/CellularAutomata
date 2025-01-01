use macroquad::prelude::*;
use macroquad::rand;
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
    health: u8,
    colour: Color,
}

const PREY_HEALTH: u8 = 3;
const PREDATOR_HEALTH: u8 = 10;
const ENVIRONMENT_HEALTH: u8 = 0;

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
                self.colour = BLACK
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
    let predator_percent = 0.02;

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
                        // Check if any prey are nearby
                        for dir in directions.iter() {
                            let new_y = dir.0 + y as isize;
                            let new_x = dir.1 + x as isize;
                            if 0 <= new_y && new_y < h as isize && 0 <= new_x && new_x < w as isize
                            {
                                let mut update_val = map[new_y as usize][new_x as usize];
                                match update_val.boid_type {
                                    BoidType::Prey => {
                                        update_val.change(BoidType::Predator);
                                        map[y][x].health = PREDATOR_HEALTH;
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
                        if map[y][x].health == 1 {
                            map[y][x].change(BoidType::Environment);
                        }
                        if safe_directions.len() == 0 {
                            map[y][x].health -= 1;
                        } else {
                            let new_dir = safe_directions
                                [(rand::rand() % safe_directions.len() as u32) as usize];
                            let newy = (y as isize + new_dir.0) as usize;
                            let newx = (x as isize + new_dir.1) as usize;
                            map[newy][newx].change(BoidType::Predator);
                            map[newy][newx].health -= 1;
                            map[y][x].change(BoidType::Environment);
                        }
                    }
                    BoidType::Prey => {
                        if map[y][x].health == 1 {
                            map[y][x].change(BoidType::Environment);
                            continue;
                        }
                        let mut prey_count = 0;
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
                                        safe_directions.push((dir.0, dir.1));
                                        prey_count += 1;
                                    }
                                    BoidType::Environment => {
                                        safe_directions.push((dir.0, dir.1));
                                    }
                                    BoidType::Predator => {
                                        continue;
                                    }
                                }
                            }
                            if safe_directions.len() == 0 {
                                map[y][x].health -= 1;
                                continue;
                            }
                            let new_dir = safe_directions
                                [(rand::rand() % safe_directions.len() as u32) as usize];
                            let newy = (y as isize + new_dir.0) as usize;
                            let newx = (x as isize + new_dir.1) as usize;
                            // Create another prey boid if ther are 2, and safe directions are greater than 2
                            if prey_count > 2 && prey_count < 6 {
                                // Flip a coin breed or move
                                let procreate_p = rand::rand() % 2;
                                if procreate_p == 1 {
                                    map[newy][newx].change(BoidType::Prey);
                                    map[y][x].health -= 1;
                                } else {
                                    map[newy][newx].change(BoidType::Prey);
                                    map[newy][newx].health -= 1;
                                    map[y][x].change(BoidType::Environment);
                                }
                            } else {
                                map[newy][newx].change(BoidType::Prey);
                                map[newy][newx].health -= 1;
                                map[y][x].change(BoidType::Environment);
                            }
                        }
                    }
                    BoidType::Environment => continue,
                }
            }
        }

        for y in 0..map.len() {
            for x in 0..map[y].len() {
                image.set_pixel(x as u32, y as u32, map[y][x].colour);
            }
        }
        texture.update(&image);
        draw_texture(&texture, 0., 0., WHITE);
        next_frame().await;
    }
}
