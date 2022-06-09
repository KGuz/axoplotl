use crate::style::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Series {
    x: Vec<f64>,
    y: Vec<f64>,
    pub style: Style,
    pub name: Option<String>,
}

impl Series {
    pub fn new<T, U>(x: &[T], y: &[U]) -> Self
    where
        T: Into<f64> + Copy,
        U: Into<f64> + Copy,
    {
        Series::default().with_data(x, y)
    }

    pub fn data(&self) -> Vec<[&f64; 2]> {
        self.x
            .iter()
            .zip(self.y.iter())
            .map(|(x, y)| [x, y])
            .collect()
    }

    pub fn into_data(self) -> Vec<[f64; 2]> {
        self.x
            .into_iter()
            .zip(self.y.into_iter())
            .map(|(x, y)| [x, y])
            .collect()
    }

    pub fn with_data<T, U>(mut self, x: &[T], y: &[U]) -> Self
    where
        T: Into<f64> + Copy,
        U: Into<f64> + Copy,
    {
        // TODO: resample x depending on difference of lengths between x and y
        assert_eq!(x.len(), y.len(), "x and y have different lengths");

        self.x = x.iter().map(|&v| v.into()).collect();
        self.y = y.iter().map(|&v| v.into()).collect();
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[macro_export]
macro_rules! series {
    ($y:ident) => {{
        let x: Vec<f64> = (0..$y.len()).map(|v| v as f64).collect();
        $crate::series::Series::new(&x, &$y).with_name(stringify!($y))
    }};

    ($x:expr, $y:ident) => {{
        $crate::series::Series::new(&$x, &$y).with_name(stringify!($y))
    }};

    // ($y:ident, $style:expr) => {{
    //     let x: Vec<f64> = (0..$y.len()).map(|v| v as f64).collect();
    //     $crate::series::Series::new(&x, &$y)
    //         .with_name(stringify!($y))
    //         .with_style($style)
    // }};

    ($y:ident, $style:literal) => {{
        let s = $crate::style::Style::from($style);
        let x: Vec<f64> = (0..$y.len()).map(|v| v as f64).collect();
        $crate::series::Series::new(&x, &$y)
            .with_name(stringify!($y))
            .with_style(s)
    }};

    // ($x:expr, $y:ident, $style:expr) => {{
    //     $crate::series::Series::new(&$x, &$y)
    //         .with_name(stringify!($y))
    //         .with_style($style)
    // }};

    ($x:expr, $y:ident, $style:literal) => {{
        let s = $crate::style::Style::from($style);
        $crate::series::Series::new(&$x, &$y)
            .with_name(stringify!($y))
            .with_style(s)
    }};
}