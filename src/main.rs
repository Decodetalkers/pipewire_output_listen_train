//! An animated solar system.
//!
//! This example showcases how to use a `Canvas` widget with transforms to draw
//! using different coordinate systems.
//!
//! Inspired by the example found in the MDN docs[1].
//!
//! [1]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_animations#An_animated_solar_system
mod backend;

use std::fmt::Display;

use iced::mouse;
use iced::widget::canvas::{Geometry, Path, Stroke, stroke};
use iced::widget::{canvas, column, pick_list};
use iced::window;
use iced::{Color, Element, Fill, Point, Rectangle, Renderer, Subscription, Theme};

use crate::backend::{FFT_SIZE, MIN_FREQ, Matrix, MatrixFixed, POINTS_PER_OCTAVE, PwEvent};

pub fn main() -> iced::Result {
    iced::application(SolarSystem::new, SolarSystem::update, SolarSystem::view)
        .subscription(SolarSystem::subscription)
        .theme(SolarSystem::theme)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShowType {
    Raw,
    Spectrum,
}

impl Display for ShowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw => f.write_str("raw"),
            Self::Spectrum => f.write_str("spectrum"),
        }
    }
}

struct SolarSystem {
    state: State,
    show_type: ShowType,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Pw(PwEvent),
    ShowTypeChanged(ShowType),
}

impl SolarSystem {
    fn new() -> Self {
        Self {
            state: State::new(),
            show_type: ShowType::Raw,
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
                self.state.set_rate(format.rate());
            }
            Message::Pw(PwEvent::Spectrum(spectrum)) => {
                self.state.set_spectrum(spectrum);
            }
            Message::Pw(PwEvent::DataNew(data)) => {
                self.state.append_data(data);
            }
            Message::ShowTypeChanged(ty) => {
                self.show_type = ty;
                self.state.show_type = ty;
            }
            _ => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            pick_list(
                [ShowType::Raw, ShowType::Spectrum],
                Some(&self.show_type),
                Message::ShowTypeChanged
            ),
            canvas(&self.state).width(Fill).height(Fill)
        ]
        .into()
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

#[derive(Debug, Default, Clone)]
struct LineData {
    data: Vec<Point>,
    color: iced::Color,
}

#[derive(Debug)]
struct LineDatas {
    raw_matrix: MatrixFixed,
    spectrum: Vec<f32>,
    rate: u32,
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
            raw_matrix: MatrixFixed::new(500, 2),
            spectrum: vec![0.; FFT_SIZE],
            rate: 50000,
        }
    }
    pub fn set_rate(&mut self, rate: u32) {
        self.rate = rate;
    }

    fn append_data(&mut self, matrix: Matrix) {
        self.raw_matrix.append(matrix);
    }
    pub fn set_spectrum(&mut self, spectrum: Vec<f32>) {
        self.spectrum = spectrum;
    }
    fn reset_matrix(&mut self, len: usize, channel: usize) {
        self.raw_matrix = MatrixFixed::new(len, channel);
    }

    fn generate_spectrum(&self, size: iced::Size) -> LineData {
        let rate = self.rate as f64;

        let log_min = MIN_FREQ.log10();
        let log_max = rate.log10();

        let octaves = (log_max - log_min) / (2.0_f64).log10();
        let num_points = (octaves * POINTS_PER_OCTAVE as f64).round().max(32.0) as usize;
        let step = size.width as f64 / num_points as f64;
        let color = COLOR_ALL[1];
        let data: Vec<Point> = (0..num_points)
            .zip(&self.spectrum)
            .map(|(index, db)| Point::new(index as f32 * step as f32, db * -10.))
            .collect();

        LineData { data, color }
    }

    fn generate_raw_datas(&self, size: iced::Size) -> Vec<LineData> {
        let len = self.raw_matrix.len();
        let width = size.width;
        let step = width / len as f32;
        let datas = self.raw_matrix.data();
        let mut output: Vec<LineData> = vec![];
        for (index, data) in datas.iter().enumerate() {
            let color = COLOR_ALL[index % COLOR_ALL.len()];
            let data: Vec<Point> = data
                .iter()
                .enumerate()
                .map(|(index, wav)| Point::new(index as f32 * step, *wav * -400.))
                .collect();
            output.push(LineData { data, color });
        }
        output
    }
}

#[derive(Debug)]
struct State {
    line_cache: canvas::Cache,
    data: LineDatas,
    show_type: ShowType,
}

impl State {
    pub fn new() -> State {
        State {
            line_cache: canvas::Cache::default(),
            data: LineDatas::new(),
            show_type: ShowType::Raw,
        }
    }

    pub fn set_rate(&mut self, rate: u32) {
        self.data.set_rate(rate);
    }

    pub fn set_spectrum(&mut self, spectrum: Vec<f32>) {
        self.data.set_spectrum(spectrum);
    }

    pub fn generate_datas(&self, size: iced::Size) -> Vec<LineData> {
        self.data.generate_raw_datas(size)
    }

    pub fn generate_spectrum(&self, size: iced::Size) -> LineData {
        self.data.generate_spectrum(size)
    }

    pub fn update_canvas(&mut self) {
        self.line_cache.clear();
    }

    pub fn append_data(&mut self, matrix: Matrix) {
        self.data.append_data(matrix);
    }
    pub fn reset_matrix(&mut self, len: usize, channel: usize) {
        self.data.reset_matrix(len, channel);
    }
}

#[derive(Default, Debug)]
struct CarvaState {
    raw: Vec<LineData>,
    spectrum: LineData,
}

impl CarvaState {
    pub fn get_data<'a>(&'a self, show_type: ShowType) -> Vec<&'a LineData> {
        match show_type {
            ShowType::Raw => self.raw.iter().collect(),
            ShowType::Spectrum => vec![&self.spectrum],
        }
    }
}

impl<Message> canvas::Program<Message> for State {
    type State = CarvaState;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &iced::Event,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        state.raw = self.generate_datas(bounds.size());
        state.spectrum = self.generate_spectrum(bounds.size());
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

            let the_data = datas.get_data(self.show_type);
            for data in the_data {
                let chat = Path::new(|path| {
                    for p in &data.data {
                        path.line_to(*p);
                    }
                    if matches!(self.show_type, ShowType::Spectrum) {
                        path.line_to(Point {
                            x: frame.width(),
                            y: 0.,
                        });
                        path.line_to(Point { x: 0., y: 0. });
                        path.close();
                    }
                });

                let translation = if matches!(self.show_type, ShowType::Raw) {
                    Point {
                        x: Point::ORIGIN.x,
                        y: frame.center().y,
                    }
                } else {
                    Point {
                        x: Point::ORIGIN.x,
                        y: frame.height() - 2.,
                    }
                };

                frame.translate(translation - Point::ORIGIN);
                if matches!(self.show_type, ShowType::Raw) {
                    frame.stroke(
                        &chat,
                        Stroke {
                            width: 3.,
                            style: stroke::Style::Solid(data.color),
                            line_dash: canvas::LineDash {
                                offset: 0,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    );
                } else {
                    frame.fill(&chat, data.color);
                }

                frame.translate(Point::ORIGIN - translation);
            }
        });

        vec![background]
    }
}
