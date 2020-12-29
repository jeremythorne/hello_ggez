use ggez::{*, graphics, event, input::keyboard};
use ggez::nalgebra as na;
use std::{env, path, VecDeque};

enum Direction {
    Left,
    Right,
    Straight
}

enum Speed {
    Accelerate,
    Brake,
    Coast
}

struct Segment {
    pos: na::Point2<f32>,
    angle: f32,
    speed: f32
}

impl Segment {
    fn new(pos: na::Point2<f32>, angle: f32, speed: f32) -> Segment {
        pos: pos,
        angle: angle,
        speed: speed
    }

    fn move() -> &mut Self {
        let velocity = na::Rotation2::new(self.angle)
                     * na::Vector2::new(-1.0, 0.0) * self.speed;

        self.pos += velocity;
        self
    }

    fn wrap(min: na::Vector2<f32>, max: na::Vector2<f32>) -> &mut Self {
        self.pos.x = ::wrap(self.pos.x, min.x, max.x);
        self.pos.y = ::wrap(self.pos.y, min.y, max.y);
        self
    }

    fn turn(direction: Direction) -> &mut Self {
        match direction {
            Direction::Left => self.angle -= 0.05,
            Direction::Right => self.angle += 0.05,
            _ => {},
        }
        self
    }

    fn accelerate(accel: Speed) -> &mut Self {
        match accel {
            Speed::Accelerate => self.speed += 0.1,
            Speed::Brake => self.speed -= 0.1,
            _ => {}
        }
        self.speed = na::clamp(self.speed, -2.0, 4.0);
        self
    }
}

struct State {
    image: graphics::Image,
    head: Segment,
    body: VeqDeque<Segment>
    direction: Direction,
    accelerate: Speed,
    desired_length: f32,
    current_length: f32
}

fn wrap(a: f32, min: f32, max: f32) -> f32 {
    if a < min {
        a + (max - min)
    } else if a > max {
        a - (max - min)
    } else {
        a
    }
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let image = graphics::Image::new(ctx, "/train00.png")?;
        let (w, h) = graphics::drawable_size(ctx);

        Ok(State {
            image,
            head: Segment::new(
                    na::Point2<f32>::new(w / 2.0, h / 2.0),
                    0.0, 1.0),
            body: VecDeque::<Segment>::new(),
            direction: Direction::Straight,
            accelerate: Speed::Coast,
            desired_length: 100.0,
            current_length: 0.0
        })
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.body.push_back(self.head);
        self.current_length += self.head.speed;

        while self.current_length > self.desired_length {
            let s = self.body.pop_front();
            self.current_length -= s.speed;
        }

        let (w, h) = graphics::drawable_size(ctx);
        self.head
            .move()
            .wrap(na::Vector2<f32>::new(0.0, 0.0),
                na::Vector2<f32>::new(w, h))
            .turn(self.direction)
            .accelerate(self.accelerate)

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context,
                      keycode: keyboard::KeyCode,
                      _keymods: keyboard::KeyMods,
                      _repeat: bool) {
        match keycode {
            keyboard::KeyCode::Escape => event::quit(ctx),
            keyboard::KeyCode::A => self.direction = Direction::Left,
            keyboard::KeyCode::D => self.direction = Direction::Right,
            keyboard::KeyCode::W => self.accelerate = Speed::Accelerate,
            keyboard::KeyCode::S => self.accelerate = Speed::Brake,
            _ => { 
                self.direction = Direction::Straight;
                self.accelerate = Speed::Coast;
            }
        };
    }

    fn key_up_event(&mut self, _ctx: &mut Context,
                    _keycode: keyboard::KeyCode,
                    _keymods: keyboard::KeyMods) {
        self.direction = Direction::Straight;
        self.accelerate = Speed::Coast;
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        
        graphics::clear(ctx, (0.1, 0.2, 0.3, 1.0).into());

        graphics::draw(ctx,
            &self.image,
            graphics::DrawParam::new()
                .src(graphics::Rect::new(0.0, 0.0, 0.2, 1.0))
                .offset(na::Point2::new(0.7, 0.5)) // turn on back wheels
                .dest(self.pos)
                .rotation(self.angle),
        )?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() {
    let resource_dir = if let Ok(manifest_dir) =
        env::var("CARGO_MANIFEST_DIR") {
            let mut path = path::PathBuf::from(manifest_dir);
            path.push("resources");
            path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ref mut ctx, ref mut event_loop) =
        ContextBuilder::new("hello_ggez", "Jez")
            .add_resource_path(resource_dir)
            .build().unwrap();

    let state = &mut State::new(ctx).unwrap();

    event::run(ctx, event_loop, state).unwrap();
}
