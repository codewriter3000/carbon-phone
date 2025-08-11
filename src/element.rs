use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                texture::{TextureBuffer, TextureRenderElement},
                AsRenderElements,
            },
            gles::{element::PixelShaderElement, GlesRenderer},
            ImportAll, ImportMem, Renderer, Texture,
        },
    },
    input::pointer::CursorImageStatus,
    render_elements,
    utils::{Clock, Monotonic, Physical, Point, Scale, Transform},
};
use std::{collections::BTreeMap, env::var, fs::File, io::Read, ops::Bound, time::Duration};
use xcursor::{parser::parse_xcursor, CursorTheme};

pub struct PointerElement<T: Texture> {
    default: BTreeMap<u64, TextureBuffer<T>>,
    total_delay: u64,
    current_delay: u64,
    status: CursorImageStatus,
}

impl<T: Texture> PointerElement<T> {
    pub fn new<R>(renderer: &mut R) -> Self
    where
        R: Renderer<TextureId = T> + ImportMem,
    {

        let theme = var("XCURSOR_THEME").unwrap_or_else(|_| "DMZ-White".to_string());
        let size: i32 = var("XCURSOR_SIZE").ok().and_then(|s| s.parse().ok()).unwrap_or(24);

        // Load the theme and get the default cursor of that theme.
        let cursor_theme = CursorTheme::load(&theme);
        
        // Try current theme, then fallback to Adwaita/DMZ-White before giving up.
        let mut cursor_path = cursor_theme.load_icon("left_ptr");

        if cursor_path.is_none() {
            let fallback_theme = CursorTheme::load("Adwaita");
            cursor_path = fallback_theme.load_icon("left_ptr");
        }

        if cursor_path.is_none() {
            let fallback_theme = CursorTheme::load("DMZ-White");
            cursor_path = fallback_theme.load_icon("left_ptr");
        }

        let cursor_path = cursor_path.expect(
            "Cursor 'left_ptr' not found. Install DMZ-White or set XCURSOR_THEME to a valid theme."
        );

        // Open the xcursor file and read the data.
        let mut cursor_file = File::open(&cursor_path)
            .expect("failed to open cursor file");
        let mut cursor_data = vec![];
        cursor_file.read_to_end(&mut cursor_data).unwrap();

        // Parse the data into xcursor::parser::Image structs.
        let cursor_images = parse_xcursor(&cursor_data)
            .expect("failed to parse Xcursor file");

        // xcursor can contain an animation of a cursor (for example a cursor with a spinner).
        // Each image can contain a delay, the time period until showing the next image of the
        // cursor animation, the total delay from the start is used as the key.
        let mut default = BTreeMap::new();

        // Loop through the xcursor images and insert it into the BTreeMap.
        let mut total_delay = 0;
        for image in cursor_images {
            // u64 should be plenty of room to store the delay.
            total_delay += image.delay as u64;

            // Generates and binds textures in OpenGL since it is using GLES2.
            let texture = renderer
                .import_memory(
                    image.pixels_rgba.as_slice(),
                    Fourcc::Abgr8888,
                    (size, size).into(),
                    false,
                )
                .unwrap();

            // A buffer that represents the texture and can be turned into a TextureRenderElement
            // which provides damage tracking. It can then be rendered as an element and stacked
            // on the output.
            let texture_buffer =
                TextureBuffer::from_texture(renderer, texture, 1, Transform::Normal, None);

            default.insert(total_delay, texture_buffer);
        }

        Self {
            default,
            total_delay,
            current_delay: 0,
            status: CursorImageStatus::Default,
        }
    }

    pub fn set_current_delay(&mut self, clock: &Clock<Monotonic>) {
        // Updates the current delay, meaning the current position in the xcursor animation. For
        // example if the current delay is 30ms, then the cursor image that has 30ms worth of
        // animation delay before it should be shown.
        let current_duration = Duration::from(clock.now());
        self.current_delay = self.total_delay % current_duration.as_millis() as u64;
    }

    pub fn set_status(&mut self, status: CursorImageStatus) {
        self.status = status;
    }
}

// This macro combines the two possible elements into one, a WaylandSurfaceRenderElement which
// is provided by the client, or the TextureRenderElement which is the default cursor.
render_elements! {
    pub PointerRenderElement<R> where
        R: ImportAll;
    Surface=WaylandSurfaceRenderElement<R>,
    Texture=TextureRenderElement<<R as Renderer>::TextureId>,
}

// Implement the AsRenderElements which determines which of the elements should be rendered, the
// default cursor or the cursor provided by the client.
impl<T: Texture + Clone + 'static, R> AsRenderElements<R> for PointerElement<T>
where
    R: Renderer<TextureId = T> + ImportAll,
{
    type RenderElement = PointerRenderElement<R>;

    fn render_elements<E>(
        &self,
        renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<E>
    where
        E: From<PointerRenderElement<R>>,
    {
        match &self.status {
            CursorImageStatus::Hidden => vec![],
            CursorImageStatus::Default => {
                let Some((_, texture)) = self
                    .default
                    .range((Bound::Included(self.current_delay), Bound::Unbounded))
                    .next()
                else {
                    log::warn!("No cursor texture found for delay {:?}", self.current_delay);
                    return vec![]; // Or return a default invisible element if needed
                };

                let element = PointerRenderElement::<R>::from(TextureRenderElement::from_texture_buffer(
                    location.to_f64(),
                    texture, // ✅ FIXED: use the texture directly, not `texture.1`
                    None,
                    None,
                    None,
                ))
                .into();
                            

                vec![element]
            }
            CursorImageStatus::Surface(surface) => {
                // Return the elements from the surface provided by the client.
                render_elements_from_surface_tree(renderer, surface, location, scale, alpha)
                    .into_iter()
                    .map(E::from)
                    .collect()
            }
        }
    }
}

// Combine all render element types into one.
// GlesRenderer must be used because PixelShaderElement does not have a generic implementation.
render_elements! {
    pub MyRenderElement<=GlesRenderer>;
    Pointer=PointerRenderElement<GlesRenderer>,
    Pixel=PixelShaderElement,
}
