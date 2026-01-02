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

use crate::backend::{Matrix, MatrixFixed, PwEvent};

pub fn main() -> iced::Result {
    iced::application(SolarSystem::new, SolarSystem::update, SolarSystem::view)
        .subscription(SolarSystem::subscription)
        .theme(SolarSystem::theme)
        .run()
}

struct SolarSystem {
    state: State,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Pw(PwEvent),
}

impl SolarSystem {
    fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.state.update_canvas();
            }

            Message::Pw(PwEvent::FormatChange(format)) => {
                let channel = format.channels();
                self.state.reset_matrix(500, channel as usize);
            }
            Message::Pw(PwEvent::DataNew(data)) => {
                self.state.append_data(data);
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
    matrix: MatrixFixed,
}

const COLOR_ALL: &'static [iced::Color] = &[
    iced::Color::WHITE,
    iced::Color::from_rgb(0.4, 0.4, 1.),
    iced::Color::from_rgb(0., 0.5, 1.),
    iced::Color::from_rgb(0.5, 0.5, 0.5),
];

impl LineDatas {
    fn new() -> Self {
        Self {
            matrix: MatrixFixed::new(500, 2),
        }
    }

    fn append_data(&mut self, matrix: Matrix) {
        self.matrix.append(matrix);
    }

    fn reset_matrix(&mut self, len: usize, channel: usize) {
        self.matrix = MatrixFixed::new(len, channel);
    }

    fn generate_datas(&self, size: iced::Size) -> Vec<LineData> {
        let len = self.matrix.len();
        let width = size.width;
        let step = width / len as f32;
        let datas = self.matrix.data();
        let mut output: Vec<LineData> = vec![];
        for (index, data) in datas.iter().enumerate() {
            let color = COLOR_ALL[index % COLOR_ALL.len()];
            let data: Vec<Point> = data
                .iter()
                .enumerate()
                .map(|(index, wav)| Point::new(index as f32 * step, *wav * 400.))
                .collect();
            output.push(LineData { data, color });
        }
        output
    }
}

#[derive(Debug)]
struct State {
    line_cache: canvas::Cache,
    datas: LineDatas,
}

impl State {
    pub fn new() -> State {
        State {
            line_cache: canvas::Cache::default(),
            datas: LineDatas::new(),
        }
    }

    pub fn generate_datas(&self, size: iced::Size) -> Vec<LineData> {
        self.datas.generate_datas(size)
    }

    pub fn update_canvas(&mut self) {
        self.line_cache.clear();
    }

    pub fn append_data(&mut self, matrix: Matrix) {
        self.datas.append_data(matrix);
    }
    pub fn reset_matrix(&mut self, len: usize, channel: usize) {
        self.datas.reset_matrix(len, channel);
    }
}

impl<Message> canvas::Program<Message> for State {
    type State = Vec<LineData>;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &iced::Event,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        *state = self.generate_datas(bounds.size());
        None
    }
    fn draw(
        &self,
        datas: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let background = self.line_cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);
            let center = frame.center();
            for data in datas {
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
