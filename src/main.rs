use std::{io, ops::Sub};

const MIN_DIST_TO_BOOST: f32 = 8000.0;
const MIN_VELOCITY: f32 = 300.0;
const TUNE_CARDINAL_BY: f32 = 500.0;
const TUNE_ANGLE_BY: f32 = 350.0;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
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
            State::Moving(Target { original, tuned }) => {
                eprintln!("moving");

                pod.run();

                if original.distance_to(&pod.position) < 600.0 {
                    state.change_target();
                }

                let target = if checkpoints.all_mapped
                    && pod.angle.abs() <= 3.0
                    && tuned.unwrap_or(original).distance_to(&pod.position) <= 2000.0
                {
                    checkpoints.get_next().to_owned()
                } else {
                    tuned.unwrap_or(original)
                };

                if next_checkpoint_dist <= pod.velocity * 3.0 {
                    pod.skip_ticks(3);
                }

                println!("{} {} {}", target.x, target.y, pod.get_thrust());
            }
            State::ChangingTarget => {
                eprintln!("changing target");

                if !checkpoints.all_mapped {
                    checkpoints.add(next_checkpoint);
                }

                checkpoints.next();

                let target = checkpoints.get_current();
                state.move_to(target);

                let point = target.tuned.unwrap_or(target.original);

                println!("{} {} {}", point.x, point.y, pod.get_thrust());
            }
        }

        pod.angle = next_checkpoint_angle;
        pod.distance_to_next = next_checkpoint_dist;
    }
}

enum State {
    Moving(Target),
    ChangingTarget,
}

impl State {
    pub fn move_to(&mut self, target: Target) {
        *self = Self::Moving(target);
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

    pub fn compare(&self, other: &Point, threshold: f32) -> Cardinal {
        let y = if other.y + threshold > self.y {
            Cardinal::Down
        } else if other.y - threshold < self.y {
            Cardinal::Up
        } else {
            Cardinal::None
        };

        let x = if other.x + threshold > self.x {
            Cardinal::Right
        } else if other.x - threshold < self.x {
            Cardinal::Left
        } else {
            Cardinal::None
        };

        Cardinal::combine(x, y)
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
    tuned_checkpoints: Vec<Point>,
    current_checkpoint: usize,
    all_mapped: bool,
    boost_on: Option<usize>,
}

impl Checkpoints {
    pub fn add(&mut self, checkpoint: Point) {
        if self.is_checkpoint_mapped(&checkpoint) {
            self.all_mapped = true;
            self.tune_checkpoints();
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

    pub fn get_current(&self) -> Target {
        let original = self.checkpoints[self.current_checkpoint];
        let tuned = self.tuned_checkpoints.get(self.current_checkpoint).copied();

        Target { original, tuned }
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

    fn tune_checkpoints(&mut self) {
        self.tuned_checkpoints = self
            .checkpoints
            .iter()
            .enumerate()
            .map(|(index, &next)| {
                let current = self
                    .checkpoints
                    .get(index - 1)
                    .copied()
                    .unwrap_or_else(|| self.checkpoints.last().copied().unwrap());
                let cardinal_direction = current.compare(&next, 10.0);
                // Define the shifts based on the cardinal direction
                let (x_shift, y_shift) = match cardinal_direction {
                    Cardinal::Left => (TUNE_CARDINAL_BY, 0.0),
                    Cardinal::Right => (-TUNE_CARDINAL_BY, 0.0),
                    Cardinal::Down => (0.0, -TUNE_CARDINAL_BY),
                    Cardinal::Up => (0.0, TUNE_CARDINAL_BY),
                    Cardinal::UpLeft => (TUNE_ANGLE_BY, TUNE_ANGLE_BY),
                    Cardinal::UpRight => (-TUNE_ANGLE_BY, TUNE_ANGLE_BY),
                    Cardinal::DownLeft => (TUNE_ANGLE_BY, -TUNE_ANGLE_BY),
                    Cardinal::DownRight => (-TUNE_ANGLE_BY, -TUNE_ANGLE_BY),
                    _ => (0.0, 0.0), // Handle other cases as needed
                };

                Point::new(next.x + x_shift, next.y + y_shift)
            })
            .collect()
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

#[derive(Debug, Clone, Copy)]
struct Target {
    original: Point,
    tuned: Option<Point>,
}

#[derive(Debug, Clone, Copy)]
enum Cardinal {
    None = 0b0000,
    Up = 0b0001,
    Right = 0b0010,
    Down = 0b0100,
    Left = 0b1000,
    UpRight = 0b0011,
    DownRight = 0b0110,
    DownLeft = 0b1100,
    UpLeft = 0b1001,
}

impl Cardinal {
    pub fn combine(first: Self, second: Self) -> Self {
        let combined = first as u8 | second as u8;
        match combined {
            0b0001 => Cardinal::Up,
            0b0010 => Cardinal::Right,
            0b0100 => Cardinal::Down,
            0b1000 => Cardinal::Left,
            0b0011 => Cardinal::UpRight,
            0b0110 => Cardinal::DownRight,
            0b1100 => Cardinal::DownLeft,
            0b1001 => Cardinal::UpLeft,
            _ => Cardinal::None,
        }
    }
}
