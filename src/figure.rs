use io::Write;
use std::{env, fs, io, path};

use crate::webimg::WebImage;
use crate::{js, series::*};
use image::RgbImage;

const COLOR_PALLETS: [[&str; 5]; 10] = [
    ["#008ffb", "#00e396", "#feb019", "#ff4560", "#775dd0"],
    ["#3f51b5", "#03a9f4", "#4caf50", "#f9ce1d", "#ff9800"],
    ["#33b2df", "#546e7a", "#d4526e", "#13d8aa", "#a5978b"],
    ["#4ecdc4", "#c7f464", "#81d4fa", "#546e7a", "#fd6a6a"],
    ["#2b908f", "#f9a3a4", "#90ee7e", "#fa4443", "#69d2e7"],
    ["#449dd1", "#f86624", "#ea3546", "#662e9b", "#c5d86d"],
    ["#d7263d", "#1b998b", "#2e294e", "#f46036", "#e2c044"],
    ["#662e9b", "#f86624", "#f9c80e", "#ea3546", "#43bccd"],
    ["#5c4742", "#a5978b", "#8d5b4c", "#5a2a27", "#C4bbaf"],
    ["#a300d6", "#7d02eb", "#5653fe", "#2983ff", "#00b1f2"],
];

#[derive(Debug, Clone, PartialEq)]
pub struct FigureBuilder<T> {
    pub title: Option<String>,
    pub width: usize,
    pub height: usize,

    palette: usize,
    data: T,
}

impl<T: Default> Default for FigureBuilder<T> {
    fn default() -> Self {
        Self {
            title: None,
            width: 1280,
            height: 720,
            palette: 0,
            data: T::default(),
        }
    }
}

impl<T> FigureBuilder<T> {
    pub fn new(title: &str, width: usize, height: usize, data: T) -> Self {
        Self {
            title: Some(title.to_string()),
            width,
            height,
            palette: 0,
            data,
        }
    }
    
    pub fn with_size(mut self, width: usize, height: usize) -> Self {
        (self.width, self.height) = (width, height);
        self
    }
    
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    fn stylesheet(id: &str) -> String {
        format!("#{id} {{height: 100%; width: auto; padding: 0; margin: 0; display: flex; align-items: center; justify-content: center;}}")
    }
}

impl<Img: WebImage> FigureBuilder<Img> {
    pub fn with_image(mut self, image: Img) -> Self {
        self.data = image;
        self
    }
    
    pub fn with_color_map(self, color_map: &str) -> FigureBuilder<RgbImage> {
        FigureBuilder {
            title: self.title,
            width: self.width,
            height: self.height,
            palette: self.palette,
            data: self.data.colormap(color_map),
        }
    }
    
    pub fn build(self) -> Figure {
        let name = self.title.unwrap_or_else(|| "figure".to_string());
        let id = "chart";
        let css = Self::stylesheet(id);

        #[rustfmt::skip]
        let html = format!(
"<style>{css}</style>
<img id='{id}' src='data:image/png;base64,{}'>", self.data.encode64());
        Figure { name, html }
    }
}

impl FigureBuilder<Vec<Series>> {
    pub fn with_palette(mut self, palette: usize) -> Self {
        self.palette = palette % 10;
        self
    }
    
    pub fn palette(&self) -> &[&str] {
        COLOR_PALLETS[self.palette].as_slice()
    }
    
    pub fn with_series(mut self, series: Series) -> Self {
        self.data.push(series);
        self
    }
    
    fn generate_options(self) -> String {
        let mut color_gen = (0..).map(|i| COLOR_PALLETS[self.palette][i % 5]);

        let mut colors = vec![];
        let mut fill = vec![];

        let mut series = vec![];
        let mut markers = (vec![], vec![], vec![], vec![]);
        let mut stroke = (vec![], vec![], vec![]);

        for ser in self.data {
            let (style, name) = (ser.style.clone(), ser.name.clone());
            series.push(js!({
                type: (style.typ()),
                name: (name)?,
                data: (ser.into_data())
            }));
            let c = style.color().unwrap_or_else(|| color_gen.next().unwrap());
            colors.push(c.to_owned());

            fill.push(if style.typ() != "area" {
                "solid"
            } else {
                "gradient"
            });

            markers.0.push(style.marker.shape().to_owned());
            markers.1.push(style.marker.size);
            markers.2.push(if style.marker.filled { 1 } else { -1 });
            markers
                .3
                .push(if style.marker.filled { "#ffffff00" } else { c }.to_owned());

            stroke.0.push(style.stroke.curve().to_owned());
            stroke.1.push(style.stroke.width);
            stroke.2.push(if style.stroke.dashed {
                3 * style.stroke.width
            } else {
                0
            });
        }

        js!({
            title: {
                text: (self.title)?
            },
            chart: {
                type: "area",
                width: "90%",
                height: "90%",
                zoom: {
                    type: "x",
                    enabled: true,
                    autoScaleYaxis: true
                },
                toolbar: {
                    autoSelected: "zoom"
                },
            },
            series: series,
            fill: {
                type: fill
            },
            colors: colors,
            markers: {
                shape: (markers.0),
                size: (markers.1),
                fillOpacity: (markers.2),
                strokeColors: (markers.3),
                hover: {
                    sizeOffset: 0
                },
                radius: 1
            },
            stroke: {
                curve: (stroke.0),
                width: (stroke.1),
                dashArray: (stroke.2),
                lineCap: "square",
            },
            dataLabels: {
                enabled: false,
            },
            xaxis: {
                type: "numeric",
                tickPlacement: "dataPoints",
                tooltip: {
                    enabled: false,
                },
            }
        })
        .pretty()
    }
    
    pub fn build(self) -> Figure {
        let name = self.title.clone().unwrap_or_else(|| "figure".to_string());
        let id = "chart";
        let css = Self::stylesheet(id);

        #[rustfmt::skip]
        let html = format!(
"<script src='https://cdn.jsdelivr.net/npm/apexcharts'></script>
<style>{css}</style>
<div id='{id}'></div>
<script>
    const options = {};
    const chart = new ApexCharts(document.querySelector('#{id}'), options);
    chart.render();
</script>", self.generate_options());

        Figure { name, html }
    }
}

#[derive(PartialEq)]
pub struct Figure {
    name: String,
    html: String,
}

impl Figure {
    pub fn save_to(&self, directory: impl AsRef<path::Path>) -> Result<path::PathBuf, io::Error> {
        if !directory.as_ref().is_dir() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "oh no!"));
        }
        let file_name = format!(
            "{}-{}.html",
            self.name,
            chrono::Local::now().format("%H%M%S")
        );
        let path = directory.as_ref().join(file_name);

        let mut file = fs::File::create(&path)?;
        file.write_all(self.html.as_bytes())?;
        Ok(path)
    }
    
    pub fn save(&self) -> Result<path::PathBuf, io::Error> {
        self.save_to(&env::current_dir()?)
    }
    
    fn webview(path: impl AsRef<path::Path>) -> Result<(), io::Error> {
        let path = path.as_ref().display();
        let config = |browser| match browser {
            "chrome" | "msedge" => format!("--app=file:///{}", path),
            "firefox" => format!("-new-window file:///{}", path),
            _ => unreachable!(),
        };

        // TODO: detect installed browsers and choose the best one
        // chromium-based browsers support app view so they are preferable
        for browser in ["chrome", "msedge", "firefox"] {
            if open::with(config(browser), browser).is_ok() {
                return Ok(());
            }
        }
        open::that(format!("file:///{}", path))
    }
    pub fn open(&self) -> Result<(), io::Error> {
        let path = self.save_to(&env::temp_dir())?;
        let result = Self::webview(&path);
        if result.is_ok() {
            println!("Press enter to continue...");
            io::stdin().read_line(&mut String::new())?;
        }
        fs::remove_file(path)?;
        result
    }
}

#[macro_export]
macro_rules! plot {
    ($ ($ ($token:tt), *); *) => {{
        let mut fig = $crate::figure::FigureBuilder::<Vec<$crate::series::Series>>::default();
        $(
            fig = fig.with_series($crate::series!($($token),*));
        )*
        fig.build()
    }};
}

#[macro_export]
macro_rules! imshow {
    ($image:ident) => {
        $crate::figure::FigureBuilder::new(
            stringify!($image),
            $image.width() as usize,
            $image.height() as usize,
            &$image,
        )
        .build()
    };

    ($image:ident, $color_map:literal) => {{
        use image::GenericImageView;
        let (w, h) = $image.dimensions();
        $crate::figure::FigureBuilder::new(
            stringify!($image),
            w as usize,
            h as usize,
            &$image,
        )
        .with_color_map($color_map)
        .build()
    }};
}
