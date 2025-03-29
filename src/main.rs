extern crate piston_window;
extern crate rand;

use piston_window::*;
use piston_window::types::Color;
use rand::Rng;
use std::collections::LinkedList;

const FOOD_COLOR: Color = [0.80, 0.00, 0.00, 1.0];
const BORDER_COLOR: Color = [0.00, 0.00, 0.00, 1.0];
const GAMEOVER_COLOR: Color = [0.90, 0.00, 0.00, 0.5];

const MOVING_PERIOD: f64 = 0.1;
const RESTART_TIME: f64 = 1.0;

const FOOD_SIZE: f64 = 10.0;
const SNAKE_BLOCK_SIZE: f64 = 10.0;
const BORDER_WIDTH: f64 = 1.0;
const BOARD_WIDTH: u32 = 50;
const BOARD_HEIGHT: u32 = 50;

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let h = h % 1.0;
    let hi = (h * 6.0).floor() as i32;
    let f = h * 6.0 - hi as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match hi {
        0 => [v, t, p, 1.0],
        1 => [q, v, p, 1.0],
        2 => [p, v, t, 1.0],
        3 => [p, q, v, 1.0],
        4 => [t, p, v, 1.0],
        _ => [v, p, q, 1.0],
    }
}

fn get_rainbow_color(index: usize, time_offset: f64) -> Color {
    let hue = ((index as f64 * 0.05 + time_offset * 0.3) % 1.0) as f32;
    hsv_to_rgb(hue, 0.8, 0.8)
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Block {
    x: i32,
    y: i32,
}

struct Snake {
    direction: Direction,
    body: LinkedList<Block>,
    tail: Option<Block>,
}

impl Snake {
    fn new(x: i32, y: i32) -> Snake {
        let mut body = LinkedList::new();
        body.push_back(Block { x, y });
        Snake {
            direction: Direction::Right,
            body,
            tail: None,
        }
    }

    fn head_position(&self) -> (i32, i32) {
        let head = self.body.front().unwrap();
        (head.x, head.y)
    }

    fn move_forward(&mut self, dir: Option<Direction>) {
        match dir {
            Some(d) => self.direction = d,
            None => (),
        }

        let (last_x, last_y) = self.head_position();

        let new_block = match self.direction {
            Direction::Up => Block {
                x: last_x,
                y: last_y - 1,
            },
            Direction::Down => Block {
                x: last_x,
                y: last_y + 1,
            },
            Direction::Left => Block {
                x: last_x - 1,
                y: last_y,
            },
            Direction::Right => Block {
                x: last_x + 1,
                y: last_y,
            },
        };
        self.body.push_front(new_block);
        self.tail = self.body.pop_back();
    }

    fn head_direction(&self) -> Direction {
        self.direction
    }

    fn next_head(&self, dir: Option<Direction>) -> (i32, i32) {
        let (head_x, head_y) = self.head_position();

        let mut moving_dir = self.direction;
        match dir {
            Some(d) => moving_dir = d,
            None => {}
        }

        match moving_dir {
            Direction::Up => (head_x, head_y - 1),
            Direction::Down => (head_x, head_y + 1),
            Direction::Left => (head_x - 1, head_y),
            Direction::Right => (head_x + 1, head_y),
        }
    }

    fn restore_tail(&mut self) {
        let blk = self.tail.clone().unwrap();
        self.body.push_back(blk);
    }

    fn overlap_tail(&self, x: i32, y: i32) -> bool {
        let mut ch = 0;
        for block in &self.body {
            if x == block.x && y == block.y {
                return true;
            }
            ch += 1;
            if ch == self.body.len() - 1 {
                break;
            }
        }
        return false;
    }
}

struct Game {
    snake: Snake,
    food_exists: bool,
    food_x: i32,
    food_y: i32,
    width: i32,
    height: i32,
    game_over: bool,
    waiting_time: f64,
    total_time: f64, 
}

impl Game {
    fn new(width: i32, height: i32) -> Game {
        Game {
            snake: Snake::new(2, 2),
            food_exists: true,
            food_x: 6,
            food_y: 4,
            width,
            height,
            game_over: false,
            waiting_time: 0.0,
            total_time: 0.0, 
        }
    }

    fn key_pressed(&mut self, key: Key) {
        if self.game_over {
            return;
        }

        let dir = match key {
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            _ => None,
        };

        if let Some(d) = dir {
            if d == self.snake.head_direction().opposite() {
                return;
            }
        }

        self.update_snake(dir);
    }

    fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.food_exists = true;
        self.food_x = 6;
        self.food_y = 4;
        self.game_over = false;
        self.waiting_time = 0.0;
        self.total_time = 0.0; 
    }

    fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;
        self.total_time += delta_time; 

        if self.game_over {
            if self.waiting_time > RESTART_TIME {
                self.restart();
            }
            return;
        }

        if !self.food_exists {
            self.add_food();
        }

        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None);
        }
    }

    fn check_eating(&mut self) {
        let (head_x, head_y) = self.snake.head_position();
        if self.food_exists && self.food_x == head_x && self.food_y == head_y {
            self.food_exists = false;
            self.snake.restore_tail();
        }
    }

    fn check_if_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head(dir);
        if self.snake.overlap_tail(next_x, next_y) {
            return false;
        }
        next_x > 0 && next_y > 0 && next_x < self.width - 1 && next_y < self.height - 1
    }

    fn add_food(&mut self) {
        let mut rng = rand::thread_rng();
        let mut new_x = rng.gen_range(1..(self.width - 1));
        let mut new_y = rng.gen_range(1..(self.height - 1));
        while self.snake.overlap_tail(new_x, new_y) {
            new_x = rng.gen_range(1..(self.width - 1));
            new_y = rng.gen_range(1..(self.height - 1));
        }
        self.food_x = new_x;
        self.food_y = new_y;
        self.food_exists = true;
    }

    fn update_snake(&mut self, dir: Option<Direction>) {
        if self.check_if_snake_alive(dir) {
            self.snake.move_forward(dir);
            self.check_eating();
        } else {
            self.game_over = true;
        }
        self.waiting_time = 0.0;
    }

    fn draw(&self, con: &Context, g: &mut G2d) {
        let mut i = 0;
        for block in &self.snake.body {
            let color = get_rainbow_color(i, self.total_time);
            draw_block(color, block.x, block.y, con, g);
            i += 1;
        }

        if self.food_exists {
            draw_block(FOOD_COLOR, self.food_x, self.food_y, con, g);
        }

        draw_rectangle(
            BORDER_COLOR,
            0,
            0,
            self.width,
            1,
            con,
            g,
        );
        draw_rectangle(
            BORDER_COLOR,
            0,
            self.height - 1,
            self.width,
            1,
            con,
            g,
        );
        draw_rectangle(
            BORDER_COLOR,
            0,
            0,
            1,
            self.height,
            con,
            g,
        );
        draw_rectangle(
            BORDER_COLOR,
            self.width - 1,
            0,
            1,
            self.height,
            con,
            g,
        );

        if self.game_over {
            draw_rectangle(
                GAMEOVER_COLOR,
                0,
                0,
                self.width,
                self.height,
                con,
                g,
            );
        }
    }
}

fn draw_block(color: Color, x: i32, y: i32, con: &Context, g: &mut G2d) {
    let gui_x = (x as f64) * SNAKE_BLOCK_SIZE;
    let gui_y = (y as f64) * SNAKE_BLOCK_SIZE;

    rectangle(
        color,
        [
            gui_x,
            gui_y,
            SNAKE_BLOCK_SIZE,
            SNAKE_BLOCK_SIZE,
        ],
        con.transform,
        g,
    );
}

fn draw_rectangle(color: Color, x: i32, y: i32, width: i32, height: i32, con: &Context, g: &mut G2d) {
    let x = (x as f64) * SNAKE_BLOCK_SIZE;
    let y = (y as f64) * SNAKE_BLOCK_SIZE;
    let width = (width as f64) * SNAKE_BLOCK_SIZE;
    let height = (height as f64) * SNAKE_BLOCK_SIZE;

    rectangle(
        color,
        [x, y, width, height],
        con.transform,
        g,
    );
}

fn main() {
    let (width, height) = (BOARD_WIDTH, BOARD_HEIGHT);
    let mut window: PistonWindow = WindowSettings::new(
        "Snake Game",
        [
            (width as f64) * SNAKE_BLOCK_SIZE,
            (height as f64) * SNAKE_BLOCK_SIZE,
        ],
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    let mut game = Game::new(width as i32, height as i32);
    
    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            game.key_pressed(key);
        }
        
        window.draw_2d(&event, |c, g, _| {
            clear([0.5, 0.5, 0.5, 1.0], g);
            game.draw(&c, g);
        });
        
        event.update(|args| {
            game.update(args.dt);
        });
    }
}
