use std::sync::Arc;
use wgpu::{SurfaceError, SurfaceFrame, TextureView, TextureViewDescriptor};

use crate::types::Surface;

pub enum OutputFrame {
    Surface {
        view: TextureView,
        surface: Arc<SurfaceFrame>,
    },
    View(Arc<TextureView>),
}

impl OutputFrame {
    pub fn from_surface(surface: &Surface) -> Result<Self, SurfaceError> {
        let mut retrieved_frame = None;
        for _ in 0..10 {
            profiling::scope!("Inner Acquire Loop");
            match surface.get_current_frame() {
                Ok(frame) => {
                    retrieved_frame = Some(frame);
                    break;
                }
                Err(SurfaceError::Timeout) => {}
                Err(e) => return Err(e),
            }
        }
        let frame = retrieved_frame.expect("Swapchain acquire timed out 10 times.");

        let view = frame.output.texture.create_view(&TextureViewDescriptor::default());

        Ok(OutputFrame::Surface {
            surface: Arc::new(frame),
            view,
        })
    }

    pub fn as_view(&self) -> &TextureView {
        match self {
            Self::Surface { view, .. } => view,
            Self::View(inner) => &**inner,
        }
    }
}
