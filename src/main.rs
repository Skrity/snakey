use std::collections::VecDeque;
use rand::{thread_rng, Rng};
use std::env::args;
use std::sync::mpsc;
use std::thread;


// TODO: spawn more food later on in the game
// TODO: detect collision with whole body
// TODO: fix food spawning inside snake

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            match getch() {
                'w' => tx.send(SnakeDirection::Up).unwrap(),
                'a' => tx.send(SnakeDirection::Left).unwrap(),
                's' => tx.send(SnakeDirection::Down).unwrap(),
                'd' => tx.send(SnakeDirection::Right).unwrap(),
                'q' => std::process::exit(0),
                x => {
                    println!("Invalid input: {}", x);
                }
            }
        }
    });
    let cmdline: Vec<String> = args().collect();
    let mut x = match cmdline.len() {
        1 => {Field{size_x: 25, size_y: 10, ..Default::default()}},
        4 => {Field{size_x: cmdline[1].parse().unwrap(), size_y: cmdline[2].parse().unwrap(), wraps: cmdline[3].parse().unwrap(), ..Default::default()}},
        _ => {println!("Takes 0 or 3 arguments: [wrap_around] [size_x] [size_y]"); return}
    };
    x.snake = Snake::new();
    x.food.push(Point(3, 3));
    x.print();
    println!("Controls: w, a, s, d | Field size: {}x{} | Wrap around: {}", x.size_x, x.size_y, if x.wraps {"allowed"} else {"forbidden"});
    let mut iteration = 0;
    loop {
        iteration += 1;
        if let Ok(dir) = rx.try_recv() {
            x.snake.change_direction(dir);
        }
        if iteration % 10 == 0 {
            x.snake.make_move((x.size_x, x.size_y), &mut x.food);
            if x.snake.detect_self_collision() || (!x.wraps && x.snake.detect_wrapping()) { println!("Game over"); break }
        }
        if x.snake.points.len() == (x.size_x*x.size_y) as usize { println!("You win!"); break }
        if x.food.is_empty() { // Put new food if no left
            x.food.push(x.find_place_for_food())
        }
        if iteration % 100 == 0 { // Put new food every 100 move
            x.food.push(x.find_place_for_food())
        }
        x.print();
        thread::sleep(std::time::Duration::from_millis(10))
    };
}

#[derive(Default)]
struct Field {
    size_x: u32,
    size_y: u32,
    snake: Snake,
    food: Vec<Point>,
    wraps: bool
}

#[derive(Default, Clone, Copy, PartialEq)]
struct Point (u32, u32);

#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum SnakeTexture {
    #[default]
    Horizontal,
    Vertical,
    DownAndLeft,
    UpAndLeft,
    DownAndRight,
    UpAndRight
}

impl From<SnakeTexture> for char {
    fn from(item: SnakeTexture) -> char {
        match item {
            SnakeTexture::Horizontal => '\u{2501}',
            SnakeTexture::Vertical => '\u{2503}',
            SnakeTexture::DownAndLeft => '\u{250F}',
            SnakeTexture::UpAndLeft => '\u{2513}',
            SnakeTexture::DownAndRight => '\u{2517}',
            SnakeTexture::UpAndRight => '\u{251B}',
        }
    }
}
#[derive(Default)]
struct Snake {
    points: VecDeque<(Point, SnakeTexture)>,
    direction: SnakeDirection,
    prev_direction: SnakeDirection,
    wrapped: bool
}


#[derive(Default, PartialEq, Clone, Copy)]
enum SnakeDirection {
    #[default]
    Up,
    Down,
    Left,
    Right
}

impl SnakeDirection {
    fn is_valid_move(&self, rhs: &Self) -> bool {
        *rhs != match self {
            SnakeDirection::Up => SnakeDirection::Down,
            SnakeDirection::Down => SnakeDirection::Up,
            SnakeDirection::Left => SnakeDirection::Right,
            SnakeDirection::Right => SnakeDirection::Left,
        }
    }
    fn get_texture(&self, next: &Self) -> SnakeTexture {
        match (self, next) {
            (SnakeDirection::Up, SnakeDirection::Left) | (SnakeDirection::Right, SnakeDirection::Down) => SnakeTexture::UpAndLeft,
            (SnakeDirection::Down, SnakeDirection::Left) | (SnakeDirection::Right, SnakeDirection::Up) => SnakeTexture::UpAndRight,
            (SnakeDirection::Up, SnakeDirection::Right) | (SnakeDirection::Left, SnakeDirection::Down) => SnakeTexture::DownAndLeft,
            (SnakeDirection::Down, SnakeDirection::Right) | (SnakeDirection::Left, SnakeDirection::Up) => SnakeTexture::DownAndRight,
            (SnakeDirection::Up, SnakeDirection::Up) | (SnakeDirection::Down, SnakeDirection::Down) => SnakeTexture::Vertical,
            (SnakeDirection::Left, SnakeDirection::Left) | (SnakeDirection::Right, SnakeDirection::Right) => SnakeTexture::Horizontal,

            (SnakeDirection::Up, SnakeDirection::Down) | (SnakeDirection::Right, SnakeDirection::Left) |
            (SnakeDirection::Down, SnakeDirection::Up) | (SnakeDirection::Left, SnakeDirection::Right) => unreachable!(),
        }
    }
}

impl Snake {
    fn new() -> Self {
        let mut me = Self {
            points: VecDeque::new(),
            direction: SnakeDirection::Down,
            prev_direction: SnakeDirection::Down,
            wrapped: false
        };
        me.points.push_front((Point(0,0), SnakeTexture::Vertical));
        me.points.push_front((Point(0,1), SnakeTexture::Vertical));
        me.points.push_front((Point(0,2), SnakeTexture::Vertical));
        me.points.push_front((Point(0,3), SnakeTexture::Vertical));
        // Valid direction is Down
        me
    }

    fn change_direction(&mut self, direction: SnakeDirection) {
        self.direction = direction;
    }

    fn make_move(&mut self, constraints: (u32, u32), food: &mut Vec<Point>) {
        let mut head = self.points.front_mut().unwrap();
        if !self.direction.is_valid_move(&self.prev_direction) {
            self.direction = self.prev_direction
        }
        let &mut (Point(mut x,mut y), _texture) = head;
        match self.direction {
            SnakeDirection::Down => y = wrap_inc(y, constraints.1, &mut self.wrapped),
            SnakeDirection::Up => y = wrap_dec(y, constraints.1, &mut self.wrapped),
            SnakeDirection::Left => x = wrap_dec(x, constraints.0, &mut self.wrapped),
            SnakeDirection::Right => x = wrap_inc(x, constraints.0, &mut self.wrapped),
        };
        let new_texture = match self.direction {
            SnakeDirection::Down | SnakeDirection::Up => SnakeTexture::Vertical,
            SnakeDirection::Left | SnakeDirection::Right => SnakeTexture::Horizontal,
        };

        head.1 = self.prev_direction.get_texture(&self.direction);
        self.points.push_front((Point(x,y), new_texture));
        self.prev_direction = self.direction;
        if !self.detect_food_collision(food) {
            self.points.pop_back();
        }
    }

    fn detect_wrapping(&self) -> bool {
        self.wrapped
    }

    fn detect_food_collision(&self, food: &mut Vec<Point>) -> bool {
        let (head, _texture) = self.points.front().unwrap();
        for (i, f) in food.iter_mut().enumerate() {
            if head == f {
                food.remove(i);
                return true
            }
        }
        false
    }

    fn detect_self_collision(&self) -> bool {
        let &head = self.points.front().unwrap();
        let mut counter = 0;
        for &snake_point in &self.points {
            if head.0 == snake_point.0 {
                counter += 1;
            }
            if counter > 1 {
                return true
            }
        }
        false
    }

}

fn wrap_inc(u: u32, constraint: u32, wrapped: &mut bool) -> u32 {
    if u + 1 == constraint {
        *wrapped = true;
        0
    } else {
        u + 1
    }
}

fn wrap_dec(u: u32, constraint: u32, wrapped: &mut bool) -> u32 {
    if u == 0 {
        *wrapped = true;
        constraint - 1
    } else {
        u - 1
    }
}

impl Field {
    fn print(&self) {
        print!("\x1B[2J\x1B[1;1H");
        Buffer::new(self.size_x, self.size_y).draw_food(&self.food).draw_snake(&self.snake.points).print(true);
    }

    fn find_place_for_food(&self) -> Point {
         Point(thread_rng().gen_range(0..self.size_x), thread_rng().gen_range(0..self.size_y))
    }
}

struct Buffer {
    buffer: Box<[char]>,
    columns: u32,
}

impl Buffer {
    fn new(x: u32, y: u32) -> Self {
        Self {
            buffer: vec![' '; (x*y) as usize].into_boxed_slice(),
            columns: x,
        }
    }
    fn _draw(&mut self, x: u32, y: u32, symbol: char) {
        let offset = y*self.columns + x;
        self.buffer[offset as usize] = symbol;
    }
    fn draw_snake(mut self, snake: &VecDeque<(Point, SnakeTexture)>) -> Self {
        for &(Point(i, j), texture) in snake {
            self._draw(i, j,  texture.into());
        }
        self
    }

    fn draw_food(mut self, food: &Vec<Point>) -> Self {
        for &Point(i, j) in food {
            let symbol = '*';
            self._draw(i, j, symbol)
        }
        self
    }

    fn print(&self, boxed: bool) { // TODO: Optimize
        let x = &self.buffer;
        let walls = if boxed {"\u{2551}"} else {""};
        let ceiling = "\u{2550}";
        let mut res = String::with_capacity(self.buffer.len()+(4*self.columns) as usize);
        if boxed {
            res.push('\u{2554}');
            res.push_str(&vec![ceiling; self.columns as usize].into_iter().collect::<String>());
            res.push('\u{2557}');
            res.push('\n');
            res.push_str(walls);
        }
        res.push_str(&x
            .chunks(self.columns as usize)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(&format!("{0}\n{0}", walls)));
        if boxed {
            res.push_str(walls);
            res.push('\n');
            res.push('\u{255A}');
            res.push_str(&vec![ceiling; self.columns as usize].into_iter().collect::<String>());
            res.push('\u{255D}');
        }
        println!("{0}", res);
    }
}

extern {
    fn _getch() -> core::ffi::c_char;
}

fn getch() -> char {
    unsafe {
        _getch() as u8 as char
    }
}
