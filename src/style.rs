use csscolorparser as css;

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    curve: String,
    pub width: usize,
    pub dashed: bool,
}

impl Stroke {
    pub fn new(curve: impl Into<String>, width: usize, dashed: bool) -> Self {
        Self {
            width,
            dashed,
            ..Default::default()
        }
        .with_curve(curve)
    }
    pub fn curve(&self) -> &str {
        &self.curve
    }
    pub fn with_curve(mut self, curve: impl Into<String>) -> Self {
        let curve = curve.into();
        let curve = match curve.as_str() {
            "smooth" | "straight" | "stepline" => curve,
            _ => "smooth".to_string(),
        };
        self.curve = curve;
        self
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            curve: "smooth".to_string(),
            width: 0,
            dashed: false,
        }
    }
}

impl<S: Into<String>> From<(S, usize, bool)> for Stroke {
    fn from(stroke: (S, usize, bool)) -> Self {
        Stroke::new(stroke.0, stroke.1, stroke.2)
    }
}

impl From<Stroke> for (String, usize, bool) {
    fn from(stroke: Stroke) -> Self {
        (stroke.curve, stroke.width, stroke.dashed)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    shape: String,
    pub size: usize,
    pub filled: bool,
}

impl Marker {
    pub fn new(shape: impl Into<String>, size: usize, filled: bool) -> Self {
        Self {
            size,
            filled,
            ..Default::default()
        }
        .with_shape(shape)
    }
    pub fn shape(&self) -> &str {
        &self.shape
    }
    pub fn with_shape(mut self, shape: impl Into<String>) -> Self {
        let shape = shape.into();
        let shape = match shape.as_str() {
            "circle" | "square" => shape,
            _ => "circle".to_string(),
        };
        self.shape = shape;
        self
    }
}

impl Default for Marker {
    fn default() -> Self {
        Self {
            shape: "circle".to_string(),
            size: 4,
            filled: true,
        }
    }
}

impl<S: Into<String>> From<(S, usize, bool)> for Marker {
    fn from(marker: (S, usize, bool)) -> Self {
        Marker::new(marker.0, marker.1, marker.2)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    typ: String,
    color: Option<String>,
    pub stroke: Stroke,
    pub marker: Marker,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: None,
            typ: "line".to_string(),
            stroke: Stroke::default(),
            marker: Marker::default(),
        }
    }
}

impl Style {
    pub fn new(
        typ: &str,
        color: &str,
        stroke: impl Into<Stroke>,
        marker: impl Into<Marker>,
    ) -> Self {
        Style::default()
            .with_typ(typ)
            .with_color(color)
            .with_stroke(stroke)
            .with_marker(marker)
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = css::parse(&color.into()).map(|c| c.to_hex_string()).ok();
        self
    }

    pub fn typ(&self) -> &str {
        &self.typ
    }

    pub fn with_typ(mut self, typ: impl Into<String>) -> Self {
        let typ = typ.into();
        self.typ = match typ.as_str() {
            "line" | "area" | "column" => typ,
            _ => "line".to_string(),
        };
        self
    }

    pub fn with_marker(mut self, marker: impl Into<Marker>) -> Self {
        self.marker = marker.into();
        self
    }

    pub fn with_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }
}

impl From<Style> for String {
    fn from(s: Style) -> Self {
        let mut color = String::new();
        if let Some(c) = &s.color {
            color.push_str(c);
        };

        let mut stroke = match s.stroke.curve() {
            "smooth" => "~",
            "stepline" => "-",
            "straight" => "/",
            _ => unreachable!(),
        }
        .repeat(s.stroke.dashed as usize + 1);
        stroke.push_str(&s.stroke.width.to_string());

        let mut marker = match (s.marker.shape(), s.marker.filled) {
            ("circle", true) => ".",
            ("circle", false) => ">",
            ("square", true) => ",",
            ("square", false) => "<",
            _ => unreachable!(),
        }
        .to_string();
        marker.push_str(&s.marker.size.to_string());

        let typ = match s.typ() {
            "line" => "",
            "area" => "@",
            "column" => "%",
            _ => unreachable!(),
        };

        format!("{color}{stroke}{marker}{typ}")
    }
}

impl From<String> for Style {
    fn from(style: String) -> Self {
        style.as_str().into()
    }
}

impl From<&str> for Style {
    fn from(style: &str) -> Self {
        let chars: Vec<char> = style.chars().collect();
        let (mut i, len) = (0, chars.len());

        let trailing = |idx: usize, f: fn(c: char) -> bool| -> String {
            chars[idx..].iter().take_while(|&c| f(*c)).collect()
        };

        let mut style = Style::default();
        while i < len {
            style = match chars[i] {
                '@' => style.with_typ("area"),
                '%' => style.with_typ("column"),

                'b' => style.with_color("blue"),
                'g' => style.with_color("green"),
                'r' => style.with_color("red"),
                'c' => style.with_color("cyan"),
                'm' => style.with_color("magenta"),
                'y' => style.with_color("yellow"),
                'o' => style.with_color("orange"),
                'k' => style.with_color("black"),
                'w' => style.with_color("white"),

                '#' => {
                    let digits = trailing(i + 1, |d| d.is_ascii_alphanumeric());
                    i += digits.len();
                    style.with_color(format!("#{digits}"))
                }

                '~' | '-' | '/' => {
                    let repeated = i + 1 < len && chars[i + 1] == chars[i];
                    i += repeated as usize;

                    let digits = trailing(i + 1, |d| d.is_ascii_digit());
                    let width = digits.parse().unwrap_or(2);

                    let curve = match chars[i] {
                        '~' => "smooth",
                        '-' => "stepline",
                        '/' => "straight",
                        _ => unreachable!(),
                    };
                    i += digits.len();
                    style.with_stroke((curve, width, repeated))
                }

                '.' | '>' | ',' | '<' => {
                    let digits = trailing(i + 1, |d| d.is_ascii_digit());
                    let size = digits.parse().unwrap_or(4);

                    let (shape, filled) = match chars[i] {
                        '.' => ("circle", true),
                        '>' => ("circle", false),
                        ',' => ("square", true),
                        '<' => ("square", false),
                        _ => unreachable!(),
                    };
                    i += digits.len();
                    style.with_marker((shape, size, filled))
                }
                _ => style,
            };
            i += 1;
        }
        style
    }
}

#[macro_export]
macro_rules! style {
    ($s:literal) => {
        $crate::style::Style::from($s)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_nonsense() {
        let default = Style::default();
        let s = style!("#f333.12//4@");
        assert_eq!(
            s,
            default
                .with_color("#f333")
                .with_marker(("circle", 12, true))
                .with_stroke(("straight", 4, true))
                .with_typ("area")
        );
    }
}
