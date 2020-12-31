use core::time;
use ggez::{*, graphics, graphics::spritebatch};
use ggez::{event, input::keyboard};
use ggez::nalgebra as na;
use std::{env, path};
use rand;
use std::collections::{VecDeque};

#[derive(PartialEq)]
enum PlayState {
    Space,
    Play,
    Dead
}

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

    fn heading(&self) -> na::Vector2::<f32> {
        na::Rotation2::new(self.angle)
                     * na::Vector2::new(-1.0, 0.0)
    }

    fn translate(&mut self) {
        let velocity = self.heading() * self.speed;

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
    pos: na::Point2<f32>,
    n: usize
}

impl Fruit {
    fn new(w: f32, h: f32) -> Fruit {
        Fruit {
            pos: na::Point2::new(
                     rand::random::<f32>() * w,
                     rand::random::<f32>() * h
                     ),
            n: (rand::random::<u8>() % 5) as usize
        }
    }
}

struct Pop {
    pos: na::Point2<f32>,
    delay: i32
}

struct Explosion {
    images: Vec<graphics::Image>,
    pops: Vec<Pop>,
    step: i32
}

impl Explosion {
    fn new(segments: std::slice::Iter<Segment>, 
           ctx: &mut Context) -> GameResult<Explosion> {
        let mut images = Vec::<graphics::Image>::new();
        for i in 0..7 {
            let s = format!("/pop0{}.png", i);
            images.push(graphics::Image::new(ctx, s)?);
        }

        let mut pops = Vec::<Pop>::new();
        for s in segments {
            if rand::random::<i32>() % 10 < 1 {
                pops.push(
                    Pop {
                        pos: na::Point2::new(
                                s.pos.x + 20.0 * (
                                     rand::random::<f32>() - 0.5),
                                s.pos.y + 20.0 * (
                                     rand::random::<f32>() - 0.5),
                                 ),
                        delay: (rand::random::<u32>() % 60) as i32
                    }
                    );
            }
        }

        Ok(Explosion {
            images,
            pops,
            step: 0
        })
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.step += 1;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut batches = Vec::<graphics::spritebatch::SpriteBatch>::new();
        for image in self.images.iter() {
            batches.push(spritebatch::SpriteBatch::new(image.clone()));
        }

        let n = batches.len() as i32;

        for pop in self.pops.iter() {
            let frame = (self.step - pop.delay) * n / 60;
            if frame >= 0 && frame < n {
                batches[frame as usize].add(
                    graphics::DrawParam::new()
                    .offset(na::Point2::new(0.5, 0.5))
                    .dest(pop.pos)
                );
            }
        }

        for batch in batches.iter() {
            graphics::draw(ctx, batch, graphics::DrawParam::new())?;
        }

        Ok(())
    }
}

struct Snake {
    image: graphics::Image,
    nose: na::Point2<f32>,
    head_radius: f32,
    head: Segment,
    body: VecDeque<Segment>,
    desired_length: f32,
    current_length: f32,
}

impl Snake {
    fn new(ctx: &mut Context) -> GameResult<Snake> {
        let image = graphics::Image::new(ctx, "/train00.png")?;
        let (w, h) = graphics::drawable_size(ctx);

        let head_radius = (image.width() as f32) * 0.1 / 2.0;

        Ok(Snake {
            image,
            nose: na::Point2::<f32>::new(0.0, 0.0),
            head_radius,
            head: Segment::new(
                    na::Point2::<f32>::new(w / 2.0, h / 2.0),
                    0.0, 1.0),
            body: VecDeque::<Segment>::new(),
            desired_length: 100.0,
            current_length: 0.0,
        })
    }

    fn collide(&self, b: &na::Point2<f32>, rb: f32) -> bool {
        collide(&self.nose, self.head_radius, b, rb)
    }

    fn collide_self(&self) -> bool {
        self.body
            .iter()
            .rev()
            .enumerate()
            .any(|(i, s)| i > 100
                        && collide(&self.nose,
                                     self.head_radius,
                                     &s.pos,
                                     self.head_radius / 2.0))
    }

    fn segments(&mut self) -> Option<std::slice::Iter<Segment>> {
        self.body.make_contiguous();
        if let (slice, &[]) = self.body.as_slices() {
            Some(slice.iter())
        } else {
            None
        }
    }

    fn update(&mut self, screen:(f32, f32),
              direction: &Direction,
              accelerate: &Speed) {
        self.body.push_back(self.head.clone());
        self.current_length += self.head.speed;

        while self.current_length > self.desired_length {
            if let Some(s) = self.body.pop_front() {
                self.current_length -= s.speed;
            }
        }

        let (w, h) = screen;
        self.head.update((na::Vector2::<f32>::new(0.0, 0.0),
                na::Vector2::<f32>::new(w, h)),
                &direction,
                &accelerate);

        self.nose = self.head.pos + self.head.heading() * self.head_radius;
    }

    fn increase_length(&mut self, length: f32) {
        self.desired_length = na::clamp(self.desired_length + length,
                                        0.0, 10000.0);
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut batch = spritebatch::SpriteBatch::new(self.image.clone());

        let w = self.image.width();
        let scale = 2.0 / ( w as f32);

        let mut f = 0.0;
        for s in self.body.iter() {
            let sw = s.speed * scale;
            let off = 1.0 - (f + sw).rem_euclid(0.9);
            batch.add(    
                graphics::DrawParam::new()
                    .src(graphics::Rect::new(off, 0.0, sw, 1.0))
                    .offset(na::Point2::new(0.5, 0.5))
                    .dest(s.pos)
                    .rotation(s.angle)
            );
            f += sw;
        }

        batch.add(
            graphics::DrawParam::new()
                .src(graphics::Rect::new(0.0, 0.0, 0.1, 1.0))
                .offset(na::Point2::new(1.0, 0.5))
                .dest(self.head.pos)
                .rotation(self.head.angle),
        );

        graphics::draw(ctx, &batch, graphics::DrawParam::new())?;
        Ok(())
    } 
}

struct State {
    play_state: PlayState,
    space_image: graphics::Image,
    fruit_images: Vec<graphics::Image>,
    fruit_radius: f32,
    snake: Snake,
    direction: Direction,
    accelerate: Speed,
    fruit: Fruit,
    dead_timer: Option<time::Duration>,
    explosion: Option<Explosion>
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
        let mut fruit_images = Vec::<graphics::Image>::new();
        for i in 0..=4 {
            let s = format!("/fruit{}0.png", i);
            fruit_images.push(graphics::Image::new(ctx, s)?);
        }
        let space_image = graphics::Image::new(ctx, "/space0.png")?;
        let (w, h) = graphics::drawable_size(ctx);

        let fruit_radius = (fruit_images[0].height() as f32) / 2.0;

        Ok(State {
            play_state: PlayState::Space,
            space_image,
            fruit_images,
            fruit_radius,
            snake: Snake::new(ctx)?,
            direction: Direction::Straight,
            accelerate: Speed::Coast,
            fruit: Fruit::new(w, h),
            dead_timer: None,
            explosion: None
        })
    }
}

fn collide(a: &na::Point2<f32>, ra: f32, b: &na::Point2<f32>, rb: f32) -> bool {
    let d = ra + rb;
    na::distance_squared(a, b) < d * d
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {

        let (w, h) = graphics::drawable_size(ctx);
        self.snake.update((w, h), &self.direction, &self.accelerate);

        if self.snake.collide(&self.fruit.pos, self.fruit_radius) {
            self.fruit = Fruit::new(w, h);
            if self.play_state == PlayState::Play {
                self.snake.increase_length(100.0);
           }
        }

        if self.play_state == PlayState::Play &&
                self.snake.collide_self() {
            self.play_state = PlayState::Dead;
            self.dead_timer = Some(timer::time_since_start(ctx));
            if let Some(segments) = self.snake.segments() {
                self.explosion = Some(Explosion::new(segments, ctx)?);
            }
        }

        if self.play_state == PlayState::Dead {
            if (timer::time_since_start(ctx) -
                    self.dead_timer.unwrap()).as_secs() > 2 {
                self.play_state = PlayState::Space;
                self.dead_timer = None;
                self.explosion = None;
                self.snake = Snake::new(ctx)?;
            }
        }

        if let Some(explosion) = &mut self.explosion {
            explosion.update(ctx)?;
        }

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context,
                      keycode: keyboard::KeyCode,
                      _keymods: keyboard::KeyMods,
                      _repeat: bool) {
        if keycode == keyboard::KeyCode::Escape {
            event::quit(ctx);
        }

        match self.play_state {
            PlayState::Space => {
                if keycode == keyboard::KeyCode::Space {
                    self.play_state = PlayState::Play
                }
            },
            PlayState::Play => {
                match keycode {
                    keyboard::KeyCode::A => self.direction = Direction::Left,
                    keyboard::KeyCode::D => self.direction = Direction::Right,
                    keyboard::KeyCode::W => self.accelerate = Speed::Accelerate,
                    keyboard::KeyCode::S => self.accelerate = Speed::Brake,
                    _ => { 
                        self.direction = Direction::Straight;
                        self.accelerate = Speed::Coast;
                    }
                };
            },
            _ => {}
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context,
                    _keycode: keyboard::KeyCode,
                    _keymods: keyboard::KeyMods) {
        self.direction = Direction::Straight;
        self.accelerate = Speed::Coast;
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        
        graphics::clear(ctx, (0.1, 0.2, 0.3, 1.0).into());

        if self.play_state != PlayState::Dead {
            self.snake.draw(ctx)?;
        }

        if let Some(explosion) = &mut self.explosion {
            explosion.draw(ctx)?;
        }

        graphics::draw(ctx,
            &self.fruit_images[self.fruit.n],
            graphics::DrawParam::new()
                .offset(na::Point2::new(0.5, 0.5))
                .dest(self.fruit.pos)
        )?;

        if self.play_state == PlayState::Space {
            let (w, h) = graphics::drawable_size(ctx);
            graphics::draw(ctx,
                &self.space_image,
                graphics::DrawParam::new()
                    .offset(na::Point2::new(0.5, 0.5))
                    .dest(na::Point2::new(w / 2.0, h / 2.0))
            )?;
        }

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
