use std::{io, ops::Sub};

const MIN_DIST_TO_BOOST: f32 = 8000.0;
const MIN_VELOCITY: f32 = 300.0;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

enum State {
    Moving(Point),
    ChangingTarget,
}

impl State {
    pub fn move_to(&mut self, point: Point) {
        *self = Self::Moving(point);
    }

    pub fn change_target(&mut self) {
        *self = Self::ChangingTarget;
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        let diff = *self - *other;

        diff.length()
    }

    pub fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Debug, Default)]
struct Checkpoints {
    checkpoints: Vec<Point>,
    current_checkpoint: usize,
    all_mapped: bool,
    boost_on: Option<usize>,
}

impl Checkpoints {
    pub fn add(&mut self, checkpoint: Point) {
        if self.is_checkpoint_mapped(&checkpoint) {
            self.all_mapped = true;
            eprintln!("all checkpoints mapped");
            self.compute_boost_checkpoint();
        } else {
            self.checkpoints.push(checkpoint);
        }
    }

    pub fn next(&mut self) {
        self.current_checkpoint += 1;

        if self.current_checkpoint >= self.checkpoints.len() {
            self.current_checkpoint = 0;
        }
    }

    pub fn get_current(&self) -> Point {
        self.checkpoints[self.current_checkpoint]
    }

    pub fn get_next(&self) -> &Point {
        self.checkpoints
            .iter()
            .cycle()
            .nth(self.current_checkpoint)
            .unwrap()
    }

    fn is_checkpoint_mapped(&self, checkpoint: &Point) -> bool {
        self.checkpoints.contains(checkpoint)
    }

    fn compute_boost_checkpoint(&mut self) {
        let distances = self
            .checkpoints
            .iter()
            .zip(self.checkpoints.iter().skip(1))
            .map(|(p1, p2)| p1.distance_to(p2))
            .collect::<Vec<f32>>();

        // Find the index of the vector that is the endpoint of the longest distance
        if let Some((max_index, _)) = distances.iter().enumerate().fold(None, |acc, (i, &val)| {
            if let Some((max_i, max_val)) = acc {
                if val > max_val {
                    Some((i, val))
                } else {
                    Some((max_i, max_val))
                }
            } else {
                Some((i, val))
            }
        }) {
            let longest_distance_index = (max_index + 1) % self.checkpoints.len(); // Adjust for cyclic vectors
            self.boost_on = Some(longest_distance_index);
        }
    }
}

#[derive(Debug)]
struct Pod {
    position: Point,
    velocity: f32,
    moving: bool,
    angle: f32,
    thrust: i32,
    boosts_used: u8,
    is_boosting: bool,
    distance_to_next: f32,
    ticks_to_skip: u8,
}

impl Pod {
    pub fn calculate_velocity(&mut self, new_pos: Point) {
        self.velocity = if self.moving {
            self.position.distance_to(&new_pos)
        } else {
            self.moving = true;
            0.0
        };
        self.position = new_pos;
    }

    pub fn full_thrust(&mut self) {
        self.thrust = 100;
    }

    pub fn brake(&mut self) {
        self.thrust = 0;
    }

    pub fn clamp_thrust(&mut self) {
        if self.velocity < MIN_VELOCITY {
            self.full_thrust();
        }
    }

    pub fn boost(&mut self) {
        if self.boosts_used > 0 {
            return;
        }
        self.is_boosting = true;
        self.boosts_used += 1;
    }

    pub fn get_thrust(&self) -> String {
        if self.is_boosting {
            "BOOST".to_owned()
        } else {
            self.thrust.to_string()
        }
    }

    pub fn run(&mut self) {
        self.is_boosting = false;

        if self.ticks_to_skip > 0 {
            self.ticks_to_skip -= 1;
            self.brake();
            self.clamp_thrust();
            return;
        }

        self.full_thrust();

        if self.distance_to_next > MIN_DIST_TO_BOOST && self.angle == 0.0 && self.thrust == 100 {
            self.boost();
        }
    }

    pub fn skip_ticks(&mut self, ticks: u8) {
        self.ticks_to_skip = ticks;
    }
}

impl Default for Pod {
    fn default() -> Self {
        Self {
            distance_to_next: f32::MAX,
            thrust: 100,
            position: Default::default(),
            velocity: Default::default(),
            moving: Default::default(),
            angle: Default::default(),
            boosts_used: Default::default(),
            is_boosting: Default::default(),
            ticks_to_skip: Default::default(),
        }
    }
}

fn main() {
    let mut checkpoints = Checkpoints::default();
    let mut state = State::ChangingTarget;
    let mut pod = Pod::default();

    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();

        let inputs = input_line.split(' ').collect::<Vec<_>>();

        let x = parse_input!(inputs[0], f32);
        let y = parse_input!(inputs[1], f32);
        let next_checkpoint_x = parse_input!(inputs[2], f32); // x position of the next check point
        let next_checkpoint_y = parse_input!(inputs[3], f32); // y position of the next check point
        let next_checkpoint_dist = parse_input!(inputs[4], f32); // distance to the next checkpoint
        let next_checkpoint_angle = parse_input!(inputs[5], f32); // angle between your pod orientation and the direction of the next checkpoint

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();

        let inputs = input_line.split(' ').collect::<Vec<_>>();

        let _opponent_x = parse_input!(inputs[0], f32);
        let _opponent_y = parse_input!(inputs[1], f32);

        let position = Point::new(x, y);
        let next_checkpoint = Point::new(next_checkpoint_x, next_checkpoint_y);

        pod.calculate_velocity(position);

        dbg!(checkpoints.boost_on);

        match state {
            State::Moving(target) => {
                eprintln!("moving");

                pod.run();

                if checkpoints.get_current() != next_checkpoint {
                    state.change_target();
                }

                let target = if checkpoints.all_mapped
                    && next_checkpoint_angle.abs() <= 3.0
                    && next_checkpoint_dist <= 2000.0
                {
                    checkpoints.get_next()
                } else {
                    &target
                };

                if next_checkpoint_dist <= pod.velocity * 3.0 {
                    pod.skip_ticks(3);
                }

                println!("{} {} {}", target.x, target.y, pod.get_thrust());
            }
            State::ChangingTarget => {
                eprintln!("changing target");

                let next_checkpoint = Point::new(next_checkpoint_x, next_checkpoint_y);
                checkpoints.add(next_checkpoint);
                checkpoints.next();
                state.move_to(next_checkpoint);

                println!(
                    "{} {} {}",
                    next_checkpoint.x,
                    next_checkpoint.y,
                    pod.get_thrust()
                );
            }
        }

        pod.angle = next_checkpoint_angle;
        pod.distance_to_next = next_checkpoint_dist;
    }
}
