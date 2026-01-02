//! An animated solar system.
//!
//! This example showcases how to use a `Canvas` widget with transforms to draw
//! using different coordinate systems.
//!
//! Inspired by the example found in the MDN docs[1].
//!
//! [1]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_animations#An_animated_solar_system
mod backend;

use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Geometry, Path, Stroke, stroke};
use iced::window;
use iced::{Color, Element, Fill, Point, Rectangle, Renderer, Subscription, Theme};
use std::time::Instant;

use crate::backend::PwEvent;

pub fn main() -> iced::Result {
    iced::application::timed(
        SolarSystem::new,
        SolarSystem::update,
        SolarSystem::subscription,
        SolarSystem::view,
    )
    .theme(SolarSystem::theme)
    .run()
}

struct SolarSystem {
    state: State,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Resize(iced::Size),
    Pw(PwEvent),
}

impl SolarSystem {
    fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    fn update(&mut self, message: Message, now: Instant) {
        match message {
            Message::Tick => {
                self.state.update(now);
            }
            Message::Resize(size) => {
                self.state.regenerate_starts(size);
            }
            Message::Pw(event) => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        canvas(&self.state).width(Fill).height(Fill).into()
    }

    fn theme(&self) -> Theme {
        Theme::Moonfly
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::Subscription::batch(vec![
            window::frames().map(|_| Message::Tick),
            window::resize_events().map(|(_, size)| Message::Resize(size)),
            backend::listen_pw().map(Message::Pw),
        ])
    }
}

#[derive(Debug)]
struct State {
    space_cache: canvas::Cache,
    system_cache: canvas::Cache,
    start: Instant,
    now: Instant,
    stars: Vec<Point>,
}

impl State {
    pub fn new() -> State {
        let now = Instant::now();
        let size = window::Settings::default().size;

        State {
            space_cache: canvas::Cache::default(),
            system_cache: canvas::Cache::default(),
            start: now,
            now,
            stars: Self::generate_stars(size.width, size.height),
        }
    }

    pub fn regenerate_starts(&mut self, iced::Size { width, height }: iced::Size) {
        self.stars = Self::generate_stars(width, height);
    }
    pub fn update(&mut self, now: Instant) {
        self.start = self.start.min(now);
        self.now = now;
        self.system_cache.clear();
    }

    fn generate_stars(width: f32, height: f32) -> Vec<Point> {
        use rand::Rng;

        let gap = width / 100.;

        let mut rng = rand::rng();

        (0..=500)
            .map(|index| {
                Point::new(
                    index as f32 * gap,
                    rng.random_range((-height / 2.0)..(height / 2.0)),
                )
            })
            .collect()
    }
}

impl<Message> canvas::Program<Message> for State {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        //use std::f32::consts::PI;

        let background = self.space_cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

            let stars = Path::new(|path| {
                for p in &self.stars {
                    path.line_to(*p);
                }
            });

            let translation = Point {
                x: Point::ORIGIN.x,
                y: frame.center().y,
            };
            frame.translate(translation - Point::ORIGIN);
            frame.stroke(
                &stars,
                Stroke {
                    width: 3.,
                    style: stroke::Style::Solid(iced::Color::WHITE),
                    line_dash: canvas::LineDash {
                        offset: 0,
                        segments: &[3.0, 6.0],
                    },
                    ..Default::default()
                },
            );
            // frame.fill(&stars, Color::WHITE);
        });

        vec![background]
    }
}
