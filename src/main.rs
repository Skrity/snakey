use std::collections::VecDeque;
use rand::{thread_rng, Rng};
use std::env::args;

fn main() {
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
        let input = getch();
        if x.snake.move_allowed(match input {
            'w' => SnakeDirection::Up,
            'a' => SnakeDirection::Left,
            's' => SnakeDirection::Down,
            'd' => SnakeDirection::Right,
            x => {
                println!("Invalid input: {}", x);
                break;
            }
        }) {
            x.snake.make_move((x.size_x, x.size_y), &mut x.food);
            if x.food.is_empty() { // Put new food if no left
                x.food.push(x.find_place_for_food())
            }
            if iteration % 100 == 0 { // Put new food every 100 move
                x.food.push(x.find_place_for_food())
            }
            if x.snake.detect_self_collision() || (!x.wraps && x.snake.detect_wrapping()) { println!("Game over"); break }
            if x.snake.points.len() == (x.size_x*x.size_y) as usize { println!("You win!"); break }
        };
        x.print();
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

#[derive(Default)]
struct Snake {
    points: VecDeque<Point>,
    direction: SnakeDirection,
    wrapped: bool
}


#[derive(Default, PartialEq)]
enum SnakeDirection {
    #[default]
    Up,
    Down,
    Left,
    Right
}

impl SnakeDirection {
    fn reverse_direction(&self) -> Self {
        match self {
            SnakeDirection::Up => SnakeDirection::Down,
            SnakeDirection::Down => SnakeDirection::Up,
            SnakeDirection::Left => SnakeDirection::Right,
            SnakeDirection::Right => SnakeDirection::Left,
        }
    }
}

impl Snake {
    fn new() -> Self {
        let mut me = Self {
            points: VecDeque::new(),
            direction: SnakeDirection::Down,
            wrapped: false
        };
        me.points.push_front(Point(0,0));
        me.points.push_front(Point(0,1));
        me.points.push_front(Point(0,2));
        me.points.push_front(Point(0,3));
        // Valid direction is Down
        me
    }

    fn move_allowed(&mut self, direction: SnakeDirection) -> bool {
        let res = direction.reverse_direction() != self.direction;
        if res {
            self.direction = direction;
        }
        res
    }

    fn make_move(&mut self, constraints: (u32, u32), food: &mut Vec<Point>) {
        let &Point(mut x,mut y) = self.points.front().unwrap();
        match self.direction {
            SnakeDirection::Down => y = wrap_inc(y, constraints.1, &mut self.wrapped),
            SnakeDirection::Up => y = wrap_dec(y, constraints.1, &mut self.wrapped),
            SnakeDirection::Left => x = wrap_dec(x, constraints.0, &mut self.wrapped),
            SnakeDirection::Right => x = wrap_inc(x, constraints.0, &mut self.wrapped),
        };
        let res = Point(x,y);
        self.points.push_front(res);
        if !self.detect_food_collision(food) {
            self.points.pop_back();
        }
    }

    fn detect_wrapping(&self) -> bool {
        self.wrapped
    }

    fn detect_food_collision(&self, food: &mut Vec<Point>) -> bool {
        let head = self.points.front().unwrap();
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
            if head == snake_point {
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
    fn draw_snake(mut self, snake: &VecDeque<Point>) -> Self {
        let mut head = true;
        for &Point(i, j) in snake {
            self._draw(i, j, if head { 'O' } else { 'o' });
            head = false;
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
        // let bottom = ""
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