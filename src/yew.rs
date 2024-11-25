#![cfg(feature = "yew")]

use yew::{function_component, html, AttrValue, Callback, Html, MouseEvent, Properties};

use alloc::format;
use alloc::string::String;
use core::fmt::Write;
use core::ops::Not as _;

use crate::render::{Canvas as RenderCanvas, Pixel};
use crate::types::Color as ModuleColor;
use crate::{EcLevel, Version};

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color<'a>(pub &'a str);

pub struct SvgAttrs {
    width: u32,
    height: u32,
    fg: String,
    bg: String,
    d: String,
}

impl<'a> Pixel for Color<'a> {
    type Canvas = Canvas<'a>;
    type Image = SvgAttrs;

    fn default_color(color: ModuleColor) -> Self {
        Color(color.select("#000", "#fff"))
    }
}

#[doc(hidden)]
pub struct Canvas<'a> {
    d: String,
    w: u32,
    h: u32,
    fg: &'a str,
    bg: &'a str,
}

impl<'a> RenderCanvas for Canvas<'a> {
    type Pixel = Color<'a>;
    type Image = SvgAttrs;

    fn new(width: u32, height: u32, dark_pixel: Color<'a>, light_pixel: Color<'a>) -> Self {
        Self { d: String::new(), w: width, h: height, fg: dark_pixel.0, bg: light_pixel.0 }
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.draw_dark_rect(x, y, 1, 1);
    }

    fn draw_dark_rect(&mut self, left: u32, top: u32, width: u32, height: u32) {
        write!(self.d, "M{left} {top}h{width}v{height}h-{width}z").unwrap();
    }

    fn into_image(self) -> SvgAttrs {
        SvgAttrs { width: self.w, height: self.h, fg: self.fg.to_owned(), bg: self.bg.to_owned(), d: self.d }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub version: Version,
    pub ec_level: EcLevel,

    pub url: AttrValue,
    pub size: AttrValue,
    #[prop_or_default]
    pub on_click: Callback<MouseEvent>,

    #[prop_or("#000000".into())]
    pub dark_color: AttrValue,
    #[prop_or("#ffffff".into())]
    pub light_color: AttrValue,

    /// set this to `false` if you disabled event bubbling as an optimization
    /// see https://yew.rs/docs/concepts/html/events#event-bubbling
    #[prop_or(true)]
    pub event_bubbling: bool,
}

#[function_component]
pub fn QrCode(props: &Props) -> Html {
    let code = crate::QrCode::with_version(&*props.url, props.version, props.ec_level).unwrap();
    let SvgAttrs { width, height, fg, bg, d } =
        code.render().dark_color(Color(&props.dark_color)).light_color(Color(&props.light_color)).build();

    html! {
        <svg
            style={format!("height: {}; aspect-ratio: 1 / 1;", props.size)}
            onclick={props.on_click.clone()}
            xmlns="http://www.w3.org/2000/svg"
            version="1.1"
            viewBox={format!("0 0 {width} {height}")}
            shape-rendering="crispEdges"
        >
            <path
                d={format!("M0 0h{width}v{height}H0z")}
                fill={bg}
                onclick={props.event_bubbling.not().then(|| props.on_click.clone())}
            />
            <path fill={fg} {d} onclick={props.event_bubbling.not().then(|| props.on_click.clone())} />
        </svg>
    }
}
