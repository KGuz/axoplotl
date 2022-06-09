use image::buffer::ConvertBuffer;
use image::{
    DynamicImage, GrayAlphaImage, GrayImage, Rgb32FImage, RgbImage, Rgba32FImage, RgbaImage, Pixel
};
use image::{ImageBuffer, Luma, LumaA, Rgb, Rgba};

type Rgb16Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
type Rgba16Image = ImageBuffer<Rgba<u16>, Vec<u16>>;
type Gray16Image = ImageBuffer<Luma<u16>, Vec<u16>>;
type GrayAlpha16Image = ImageBuffer<LumaA<u16>, Vec<u16>>;
type Gray32fImage = ImageBuffer<Luma<f32>, Vec<f32>>;
type GrayAlpha32fImage = ImageBuffer<LumaA<f32>, Vec<f32>>;

pub trait WebImage  {
    fn encode64(&self) -> String;
    fn colormap(&self, cm: &str) -> RgbImage;
}

macro_rules! encode {
    ($img:expr) => {{
        let mut buf: Vec<u8> = Vec::new();
        $img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageOutputFormat::Png).unwrap();
        base64::encode(buf)
    }};
}

macro_rules! recolor {
    ($img:expr, $cm:expr) => {{
        let gradient = match $cm {
            "br_bg" => Some(colorgrad::br_bg()),
            "pr_gn" => Some(colorgrad::pr_gn()),
            "pi_yg" => Some(colorgrad::pi_yg()),
            "pu_or" => Some(colorgrad::pu_or()),
            "rd_bu" => Some(colorgrad::rd_bu()),
            "rd_gy" => Some(colorgrad::rd_gy()),
            "rd_yl_bu" => Some(colorgrad::rd_yl_bu()),
            "rd_yl_gn" => Some(colorgrad::rd_yl_gn()),
            "spectral" => Some(colorgrad::spectral()),
            "blues" => Some(colorgrad::blues()),
            "greens" => Some(colorgrad::greens()),
            "greys" => Some(colorgrad::greys()),
            "oranges" => Some(colorgrad::oranges()),
            "purples" => Some(colorgrad::purples()),
            "reds" => Some(colorgrad::reds()),
            "turbo" => Some(colorgrad::turbo()),
            "viridis" => Some(colorgrad::viridis()),
            "inferno" => Some(colorgrad::inferno()),
            "magma" => Some(colorgrad::magma()),
            "plasma" => Some(colorgrad::plasma()),
            "cividis" => Some(colorgrad::cividis()),
            "warm" => Some(colorgrad::warm()),
            "cool" => Some(colorgrad::cool()),
            "cubehelix" => Some(colorgrad::cubehelix_default()),
            "bu_gn" => Some(colorgrad::bu_gn()),
            "bu_pu" => Some(colorgrad::bu_pu()),
            "gn_bu" => Some(colorgrad::gn_bu()),
            "or_rd" => Some(colorgrad::or_rd()),
            "pu_bu_gn" => Some(colorgrad::pu_bu_gn()),
            "pu_bu" => Some(colorgrad::pu_bu()),
            "pu_rd" => Some(colorgrad::pu_rd()),
            "rd_pu" => Some(colorgrad::rd_pu()),
            "yl_gn_bu" => Some(colorgrad::yl_gn_bu()),
            "yl_gn" => Some(colorgrad::yl_gn()),
            "yl_or_br" => Some(colorgrad::yl_or_br()),
            "yl_or_rd" => Some(colorgrad::yl_or_rd()),
            "rainbow" => Some(colorgrad::rainbow()),
            "sinebow" => Some(colorgrad::sinebow()),
            _ => None,
        };
        
        if let Some(grad) = gradient {
            let (mut max, mut min) = (f64::MIN, f64::MAX);
            let buf: Vec<f64> = $img.pixels().map(|&px| {
                let val = px.to_luma()[0] as f64;
                if val > max { max = val }
                if val < min { min = val }
                val
            }).collect();

            let (width, height) = $img.dimensions();
            RgbImage::from_fn(width, height, |x, y| {
                let p = (buf[(x + y * width) as usize] - min) / max;
                let (r, g, b, _) = grad.at(p).rgba_u8();
                Rgb([r, g, b])
            })
        } else {
            $img.convert()
        }
    }};
}


macro_rules! impl_webimage  {
    ($($Image:ty),*) => {$(
        impl WebImage for $Image {
            fn encode64(&self) -> String {
                encode!(self)
            }
            fn colormap(&self, cm: &str) -> RgbImage {
                recolor!(self, cm)
            }
        }
    )*};
}

macro_rules! impl_webimage_lossy {
    ($cast:ty; $($Image:ty),*) => {$(
        impl WebImage for $Image {
            fn encode64(&self) -> String {
                encode!(self.convert() as $cast)
            }
            fn colormap(&self, cm: &str) -> RgbImage {
                recolor!(self, cm)
            }
        }
    )*};
}

macro_rules! impl_webimage_dynamic {
    ($($Image:ty),*) => {$(
        impl WebImage for $Image {
            fn encode64(&self) -> String {
                use DynamicImage::*;
                match self {
                    ImageRgb32F(_) => encode!(self.to_rgb16()),
                    ImageRgba32F(_) => encode!(self.to_rgba16()),
                    _ => encode!(self),
                }
            }
            fn colormap(&self, cm: &str) -> RgbImage {
                use DynamicImage::*;
                match self {
                    ImageLuma8(img)   => recolor!(img, cm),
                    ImageLumaA8(img)  => recolor!(img, cm),
                    ImageRgb8(img)    => recolor!(img, cm),
                    ImageRgba8(img)   => recolor!(img, cm),
                    ImageLuma16(img)  => recolor!(img, cm),
                    ImageLumaA16(img) => recolor!(img, cm),
                    ImageRgb16(img)   => recolor!(img, cm),
                    ImageRgba16(img)  => recolor!(img, cm),
                    ImageRgb32F(img)  => recolor!(img, cm),
                    ImageRgba32F(img) => recolor!(img, cm),
                    _ => recolor!(self.to_luma8(), cm),
                }
            }
        }
    )*};
}

impl_webimage!(RgbImage, RgbaImage, GrayImage, GrayAlphaImage, 
    &RgbImage, &RgbaImage, &GrayImage, &GrayAlphaImage);
impl_webimage!(Rgb16Image, Rgba16Image, Gray16Image, GrayAlpha16Image, 
    &Rgb16Image, &Rgba16Image, &Gray16Image, &GrayAlpha16Image);

impl_webimage_lossy!(Rgb16Image; Rgb32FImage, &Rgb32FImage);
impl_webimage_lossy!(Rgba16Image; Rgba32FImage, &Rgba32FImage);
impl_webimage_lossy!(Gray16Image; Gray32fImage, &Gray32fImage);
impl_webimage_lossy!(GrayAlpha16Image; GrayAlpha32fImage, &GrayAlpha32fImage);

impl_webimage_dynamic!(DynamicImage, &DynamicImage);

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> DynamicImage {
        DynamicImage::ImageLuma8(GrayImage::from_raw(1, 1, vec![0]).unwrap())
    }

    #[test]
    fn rgb_test() {
        let rgb8 = image().to_rgb8().encode64();
        let rgb8_alpha = image().to_rgba8().encode64();
        let rgb16 = image().to_rgb16().encode64();
        let rgb16_alpha = image().to_rgba16().encode64();

        assert_eq!(rgb8, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAACklEQVR4nGMAAgAABAABilw1LQAAAABJRU5ErkJggg==");
        assert_eq!(rgb8_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVR4nGMAgv8AAQQBAMtS9h0AAAAASUVORK5CYII=");
        assert_eq!(rgb16, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAIAAADA54+dAAAACklEQVR4nGOAAAAABwABTcTAjQAAAABJRU5ErkJggg==");
        assert_eq!(rgb16_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAYAAABPhRjKAAAADElEQVR4nGOAgP//AQMGAf/d+o2sAAAAAElFTkSuQmCC");
    }

    #[test]
    fn luma_test() {
        let luma8 = image().to_luma8().encode64();
        let luma8_alpha = image().to_luma_alpha8().encode64();
        let luma16 = image().to_luma16().encode64();
        let luma16_alpha = image().to_luma_alpha16().encode64();

        assert_eq!(luma8, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNgAAAAAgABSK+kcQAAAABJRU5ErkJggg==");
        assert_eq!(luma8_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR4nGNg+A8AAQIBAEK+vGgAAAAASUVORK5CYII=");
        assert_eq!(luma16, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAAAAABq7kcWAAAAC0lEQVR4nGNgYAAAAAMAAbitOmMAAAAASUVORK5CYII=");
        assert_eq!(luma16_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAQAAADljNBBAAAADUlEQVR4nGNgYPj/HwADAgH/5ncLrgAAAABJRU5ErkJggg==");
    }

    #[test]
    fn lossy_test() {
        let rgb32f = image().to_rgb32f().encode64();
        let rgb32f_alpha = image().to_rgba32f().encode64();
        let luma32f = image().to_luma32f().encode64();
        let luma32f_alpha = image().to_luma_alpha32f().encode64();

        assert_eq!(rgb32f, image().to_rgb16().encode64());
        assert_eq!(rgb32f_alpha, image().to_rgba16().encode64());
        assert_eq!(luma32f, image().to_luma16().encode64());
        assert_eq!(luma32f_alpha, image().to_luma_alpha16().encode64());
    }

    #[test]
    fn dynamic_test() {
        let dyn_rgb8 = DynamicImage::ImageRgb8(image().to_rgb8()).encode64();
        let dyn_rgb8_alpha = DynamicImage::ImageRgba8(image().to_rgba8()).encode64();
        let dyn_rgb16 = DynamicImage::ImageRgb16(image().to_rgb16()).encode64();
        let dyn_rgb16_alpha = DynamicImage::ImageRgba16(image().to_rgba16()).encode64();
        let dyn_luma8 = DynamicImage::ImageLuma8(image().to_luma8()).encode64();
        let dyn_luma8_alpha = DynamicImage::ImageLumaA8(image().to_luma_alpha8()).encode64();
        let dyn_luma16 = DynamicImage::ImageLuma16(image().to_luma16()).encode64();
        let dyn_luma16_alpha = DynamicImage::ImageLumaA16(image().to_luma_alpha16()).encode64();
        let dyn_rgb32f = DynamicImage::ImageRgb32F(image().to_rgb32f()).encode64();
        let dyn_rgb32f_alpha = DynamicImage::ImageRgba32F(image().to_rgba32f()).encode64();

        assert_eq!(dyn_rgb8, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAACklEQVR4nGMAAgAABAABilw1LQAAAABJRU5ErkJggg==");
        assert_eq!(dyn_rgb8_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVR4nGMAgv8AAQQBAMtS9h0AAAAASUVORK5CYII=");
        assert_eq!(dyn_rgb16, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAIAAADA54+dAAAACklEQVR4nGOAAAAABwABTcTAjQAAAABJRU5ErkJggg==");
        assert_eq!(dyn_rgb16_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAYAAABPhRjKAAAADElEQVR4nGOAgP//AQMGAf/d+o2sAAAAAElFTkSuQmCC");
        assert_eq!(dyn_luma8, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNgAAAAAgABSK+kcQAAAABJRU5ErkJggg==");
        assert_eq!(dyn_luma8_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR4nGNg+A8AAQIBAEK+vGgAAAAASUVORK5CYII=");
        assert_eq!(dyn_luma16, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAAAAABq7kcWAAAAC0lEQVR4nGNgYAAAAAMAAbitOmMAAAAASUVORK5CYII=");
        assert_eq!(dyn_luma16_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAQAAADljNBBAAAADUlEQVR4nGNgYPj/HwADAgH/5ncLrgAAAABJRU5ErkJggg==");
        assert_eq!(dyn_rgb32f, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAIAAADA54+dAAAACklEQVR4nGOAAAAABwABTcTAjQAAAABJRU5ErkJggg==");
        assert_eq!(dyn_rgb32f_alpha, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABEAYAAABPhRjKAAAADElEQVR4nGOAgP//AQMGAf/d+o2sAAAAAElFTkSuQmCC");
    }
}