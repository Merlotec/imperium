use crate::*;


/// The 3D vertex structure which represents the vetex data which is passed to the shader.
/// This contains both the position of the vertex (in world space) and the uv coordinate used for texturing.
#[derive(Copy, Clone)]
pub struct UvVertex3f {

    pub pos: Vector3f,
    pub uv: Vector2f,

}


impl UvVertex3f {

    pub fn new(pos: Vector3f, uv: Vector2f) -> UvVertex3f {
        return UvVertex3f { pos, uv };
    }

}

#[derive(Copy, Clone)]
pub struct UvVertex2f {

    pub pos: Vector2f,
    pub uv: Vector2f,

}

impl UvVertex2f {

    pub fn new(pos: Vector2f, uv: Vector2f) -> UvVertex2f {

        return UvVertex2f { pos, uv };

    }

    pub fn create_rect_array(rect: Rect2f) -> [UvVertex2f; 4] {

        return [
            UvVertex2f::new(Vector2f::new(rect.x, rect.y), Vector2f::new(0.0, 0.0)),
            UvVertex2f::new(Vector2f::new(rect.x + rect.width, rect.y), Vector2f::new(1.0, 0.0)),
            UvVertex2f::new(Vector2f::new(rect.x + rect.width, rect.y + rect.height), Vector2f::new(1.0, 1.0)),
            UvVertex2f::new(Vector2f::new(rect.x, rect.y + rect.height), Vector2f::new(0.0, 1.0)),
        ]

    }

}

#[derive(Clone)]
pub struct Texture {

    pub data: Vec<u8>,
    pub dimensions: Vector2u,

}

impl Texture {

    pub fn new() -> Texture {

        return Texture { data: vec![0, 0, 0, 0], dimensions: Vector2u::new(2, 2) };

    }

    pub fn from_bytes(data: &[u8], dimensions: Vector2u) -> Texture {

        return Texture { data: Vec::from(data), dimensions };

    }

    pub fn from_image(image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> Texture {

        return Texture::from_bytes(image.as_ref(), Vector2u::new(image.dimensions().0, image.dimensions().1));

    }

    pub fn from_file(path: &str) -> Result<Texture, &'static str> {
        if let Ok(image) = image::open(path) {
            let img = image.to_rgba();
            let (width, height) = img.dimensions();
            return Ok(Texture { data: Vec::from(img.as_ref()), dimensions: Vector2u::new(width, height) });
        }
        return Err("Could not find a valid image file at the path specified.");

    }

    pub fn from_image_bytes(bytes: &[u8]) -> Result<Texture, &'static str> {
        if let Ok(image) = image::load_from_memory(bytes) {
            let img = image.to_rgba();
            let (width, height) = img.dimensions();
            return Ok(Texture { data: Vec::from(img.as_ref()), dimensions: Vector2::new(width, height) });
        }
        return Err("Failed to load texture from bytes. Perhaps the bytes were of invalid format?");
    }

}

pub trait TextureRenderComponent {

    fn get_texture(&self) -> &buffer::TextureBuffer;

}

