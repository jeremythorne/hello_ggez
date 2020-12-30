use ggez::{*, graphics, graphics::spritebatch, event, input::keyboard};
use ggez::nalgebra as na;
use std::{env, path};
use rand;
use std::collections::{VecDeque};

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

#[derive(Clone)]
struct Segment {
    pos: na::Point2<f32>,
    angle: f32,
    speed: f32
}

impl Segment {
    fn new(pos: na::Point2<f32>, angle: f32, speed: f32) -> Segment {
        Segment {
            pos: pos,
            angle: angle,
            speed: speed
        }
    }

    fn update(&mut self, screen: (na::Vector2<f32>, na::Vector2<f32>),
              direction: &Direction, accel: &Speed) {
        self.translate();
        self.wrap(screen.0, screen.1);
        self.turn(direction);
        self.accelerate(accel);
    }

    fn translate(&mut self) {
        let velocity = na::Rotation2::new(self.angle)
                     * na::Vector2::new(-1.0, 0.0) * self.speed;

        self.pos += velocity;
    }

    fn wrap(&mut self, min: na::Vector2<f32>, max: na::Vector2<f32>) {
        self.pos.x = wrap(self.pos.x, min.x, max.x);
        self.pos.y = wrap(self.pos.y, min.y, max.y);
    }

    fn turn(&mut self, direction: &Direction) {
        match direction {
            Direction::Left => self.angle -= 0.01 * self.speed,
            Direction::Right => self.angle += 0.01 * self.speed,
            _ => {},
        }
    }

    fn accelerate(&mut self, accel: &Speed) {
        match accel {
            Speed::Accelerate => self.speed += 0.1,
            Speed::Brake => self.speed -= 0.1,
            _ => {}
        }
        self.speed = na::clamp(self.speed, 0.0, 4.0);
    }
}

struct Fruit {
    pos: na::Point2<f32>
}

impl Fruit {
    fn new(w: f32, h: f32) -> Fruit {
        Fruit {
            pos: na::Point2::new(
                     rand::random::<f32>() * w,
                     rand::random::<f32>() * h
                     )
        }
    }
}

struct State {
    image: graphics::Image,
    fruit_image: graphics::Image,
    head_radius: f32,
    fruit_radius: f32,
    head: Segment,
    body: VecDeque<Segment>,
    direction: Direction,
    accelerate: Speed,
    desired_length: f32,
    current_length: f32,
    fruit: Fruit
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
        let fruit_image = graphics::Image::new(ctx, "/fruit00.png")?;
        let (w, h) = graphics::drawable_size(ctx);

        let head_radius = (image.height() as f32) / 2.0;
        let fruit_radius = (fruit_image.height() as f32) / 2.0;

        Ok(State {
            image,
            fruit_image,
            head_radius,
            fruit_radius,
            head: Segment::new(
                    na::Point2::<f32>::new(w / 2.0, h / 2.0),
                    0.0, 1.0),
            body: VecDeque::<Segment>::new(),
            direction: Direction::Straight,
            accelerate: Speed::Coast,
            desired_length: 100.0,
            current_length: 0.0,
            fruit: Fruit::new(w, h)
        })
    }
}

fn collide(a: &na::Point2<f32>, ra: f32, b: &na::Point2<f32>, rb: f32) -> bool {
    let d = ra + rb;
    na::distance_squared(a, b) < d * d
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.body.push_back(self.head.clone());
        self.current_length += self.head.speed;

        while self.current_length > self.desired_length {
            if let Some(s) = self.body.pop_front() {
                self.current_length -= s.speed;
            }
        }

        let (w, h) = graphics::drawable_size(ctx);
        self.head.update((na::Vector2::<f32>::new(0.0, 0.0),
                na::Vector2::<f32>::new(w, h)),
                &self.direction,
                &self.accelerate);

        if collide(&self.head.pos, self.head_radius,
            &self.fruit.pos, self.fruit_radius) {
            self.fruit = Fruit::new(w, h);
            self.desired_length = na::clamp(
                self.desired_length + 10.0, 0.0, 10000.0);
            
        }

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

        let w = self.image.width();
        let scale = 2.0 / ( w as f32);

        let mut batch = spritebatch::SpriteBatch::new(self.image.clone());

        for s in self.body.iter() {
            batch.add(    
                graphics::DrawParam::new()
                    .src(graphics::Rect::new(0.1, 0.0,
                                             s.speed * scale, 1.0))
                    .offset(na::Point2::new(0.5, 0.5))
                    .dest(s.pos)
                    .rotation(s.angle),
            );
        }

        batch.add(
            graphics::DrawParam::new()
                .src(graphics::Rect::new(0.0, 0.0, 0.1, 1.0))
                .offset(na::Point2::new(1.0, 0.5))
                .dest(self.head.pos)
                .rotation(self.head.angle),
        );

        graphics::draw(ctx, &batch, graphics::DrawParam::new())?;

        graphics::draw(ctx,
            &self.fruit_image,
            graphics::DrawParam::new()
                .offset(na::Point2::new(0.5, 0.5))
                .dest(self.fruit.pos)
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
