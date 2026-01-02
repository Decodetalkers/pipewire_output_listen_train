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

use crate::backend::{Matrix, MatrixFixed, PwEvent};

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

    fn update(&mut self, message: Message, _now: Instant) {
        match message {
            Message::Tick => {
                self.state.update();
            }
            Message::Resize(size) => {
                self.state.regenerate_starts(size);
            }
            Message::Pw(PwEvent::FormatChange(format)) => {
                let channel = format.channels();
                self.state.lines_mut().reset_matrix(1000, channel as usize);
            }
            Message::Pw(PwEvent::DataNew(data)) => {
                self.state.lines_mut().append_data(data);
            }
            _ => {}
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
struct LineData {
    data: Vec<Point>,
    color: iced::Color,
}

#[derive(Debug)]
struct LineDatas {
    inner: Vec<LineData>,
    size: iced::Size,
    matrix: MatrixFixed,
}

const COLOR_ALL: &'static [iced::Color] = &[
    iced::Color::WHITE,
    iced::Color::from_rgb(0.4, 0.4, 1.),
    iced::Color::from_rgb(0., 0.5, 1.),
    iced::Color::from_rgb(0.5, 0.5, 0.5),
];

impl LineDatas {
    fn new(size: iced::Size) -> Self {
        Self {
            inner: Vec::new(),
            size,
            matrix: MatrixFixed::new(1000, 2),
        }
    }

    fn append_data(&mut self, matrix: Matrix) {
        self.matrix.append(matrix);
        self.generate_datas();
    }

    fn update_size(&mut self, size: iced::Size) {
        self.size = size;
        self.generate_datas();
    }

    fn reset_matrix(&mut self, len: usize, channel: usize) {
        self.matrix = MatrixFixed::new(len, channel);
        self.generate_datas();
    }

    fn generate_datas(&mut self) {
        let len = self.matrix.len();
        let width = self.size.width;
        let step = width / len as f32;
        let datas = self.matrix.data();
        self.inner.clear();
        for (index, data) in datas.iter().enumerate() {
            let color = COLOR_ALL[index % COLOR_ALL.len()];
            let data: Vec<Point> = data
                .iter()
                .enumerate()
                .map(|(index, wav)| Point::new(index as f32 * step, *wav * 200.))
                .collect();
            self.inner.push(LineData { data, color });
        }
    }

    fn data(&self) -> &[LineData] {
        &self.inner
    }
}

#[derive(Debug)]
struct State {
    space_cache: canvas::Cache,
    datas: LineDatas,
}

impl State {
    pub fn new() -> State {
        let size = window::Settings::default().size;

        State {
            space_cache: canvas::Cache::default(),
            datas: LineDatas::new(size),
        }
    }

    pub fn regenerate_starts(&mut self, size: iced::Size) {
        self.datas.update_size(size);
    }

    pub fn lines_mut(&mut self) -> &mut LineDatas {
        &mut self.datas
    }
    pub fn lines(&self) -> &LineDatas {
        &self.datas
    }
    pub fn update(&mut self) {
        self.space_cache.clear();
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
        let background = self.space_cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);
            let center = frame.center();
            let datas = self.lines().data();
            for data in datas.as_ref() {
                let stars = Path::new(|path| {
                    for p in &data.data {
                        path.line_to(*p);
                    }
                });

                let translation = Point {
                    x: Point::ORIGIN.x,
                    y: center.y,
                };
                frame.translate(translation - Point::ORIGIN);
                frame.stroke(
                    &stars,
                    Stroke {
                        width: 3.,
                        style: stroke::Style::Solid(data.color),
                        line_dash: canvas::LineDash {
                            offset: 0,
                            segments: &[3.0, 6.0],
                        },
                        ..Default::default()
                    },
                );
                frame.translate(Point::ORIGIN - translation);
            }
        });

        vec![background]
    }
}
