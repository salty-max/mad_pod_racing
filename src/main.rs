use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[derive(Default, Debug)]
struct Game {
    used_boost: bool,
    x: i32,
    y: i32,
    next_checkpoint_x: i32,
    next_checkpoint_y: i32,
    next_checkpoint_dist: i32,
    next_checkpoint_angle: i32,
    opponent_x: i32,
    opponent_y: i32,
    breaking_point: i32,
    decel_point: i32,
    min_dist_to_boost: i32,
    max_angle_to_boost: i32,
    speed: i32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            used_boost: false,
            breaking_point: 500,
            decel_point: 1000,
            min_dist_to_boost: 4000,
            max_angle_to_boost: 20,
            speed: 100,
            ..Default::default()
        }
    }
    pub fn tick(&mut self) {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();

        let inputs = input_line.split(' ').collect::<Vec<_>>();

        self.x = parse_input!(inputs[0], i32);
        self.y = parse_input!(inputs[1], i32);
        self.next_checkpoint_x = parse_input!(inputs[2], i32); // x position of the next check point
        self.next_checkpoint_y = parse_input!(inputs[3], i32); // y position of the next check point
        self.next_checkpoint_dist = parse_input!(inputs[4], i32); // distance to the next checkpoint
        self.next_checkpoint_angle = parse_input!(inputs[5], i32); // angle between your pod orientation and the direction of the next checkpoint

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();

        let inputs = input_line.split(' ').collect::<Vec<_>>();

        self.opponent_x = parse_input!(inputs[0], i32);
        self.opponent_y = parse_input!(inputs[1], i32);
    }

    pub fn run(&mut self) -> String {
        // Use boost strategically
        let speed = if self.should_boost() {
            self.used_boost = true; // Allow boost only once
            "BOOST".to_owned()
        } else {
            self.speed.to_string()
        };

        format!(
            "{} {} {speed}",
            self.next_checkpoint_x, self.next_checkpoint_y
        )
    }

    pub fn adjust_speed(&mut self) {
        // Adjust thrust based on the distance to the next checkpoint
        if self.next_checkpoint_dist > self.decel_point {
            self.speed = 100;
        } else if self.next_checkpoint_dist > self.breaking_point {
            self.speed = 80;
        } else {
            self.speed = 50;
        }

        // Adjust thrust based on checkpoint angle
        if self.next_checkpoint_angle.abs() > 90 {
            self.speed = 0;
        }
    }

    fn should_boost(&mut self) -> bool {
        !self.used_boost
            && self.next_checkpoint_dist > self.min_dist_to_boost
            && self.next_checkpoint_angle.abs() < self.max_angle_to_boost
    }
}

fn main() {
    let mut game = Game::new();

    loop {
        game.tick();
        game.adjust_speed();
        let res = game.run();

        println!("{res}")
    }
}
