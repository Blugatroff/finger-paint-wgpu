use wgpu_glyph::{FontId, VerticalAlign, HorizontalAlign};
use cgmath::Vector2;

/// one Piece of Text with a given Font, Color, and Scale
/// a TextSection is put into a Paragraph to render it
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct TextSection {
    pub text: String,
    pub color: [f32; 4],
    pub scale: f32,
    pub font: FontId,
}

/// one block of text with individual text-sections inside
/// and a position on screen
/// updating the text inside it causes can cause an expensive
/// update to the internal buffers depending on the amount of text
#[derive(Debug, Clone)]
pub struct Paragraph {
    /// how the paragraph should be aligned on the y-axis
    /// Bottom -> the y-position describes the bottom of the paragraph
    /// Top -> the y-position describes the top of the paragraph
    pub vertical_alignment: VerticalAlign,
    /// how the paragraph should be aligned on the x-axis
    /// Left -> the x-position describes the Left side of the paragraph
    /// Right -> the x-position describes the Right side of the paragraph
    pub horizontal_alignment: HorizontalAlign,
    /// the position of the Paragraph on the screen in pixels
    pub position: Vector2<f32>,
    /// sections of text in the paragraph
    /// each section can have a different font, color and scale
    pub sections: Vec<TextSection>,
}
