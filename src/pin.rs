use derivative::Derivative;
use super::*;

#[derive(Default, Debug)]
/// The Visual Style of a Link. 
/// If feilds are None then the Context style is used.
/// shape defualts to CircleFilled
pub struct PinArgs {
    pub shape: PinShape,
    pub flags: Option<usize>,
    pub background: Option<egui::Color32>,
    pub hovered: Option<egui::Color32>
}

impl PinArgs {
    pub const fn new() -> Self {
        Self {
            shape: PinShape::CircleFilled,
            flags: None,
            background: None,
            hovered: None
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum AttributeType {
    None,
    Input,
    Output
}
impl Default for AttributeType { fn default() -> Self {Self::None}}

/// Controls the shape of an attribut pin.
/// Triangle and TriangleFilled are not currently implemented and will not be drawn
#[derive(Clone, Copy, Debug)]
pub enum PinShape {
    Circle,
    CircleFilled,
    Triangle,
    TriangleFilled,
    Quad,
    QuadFilled
}
impl Default for PinShape { fn default() -> Self {Self::CircleFilled}}

/// Controls the way that attribute pins behave
#[derive(Debug)]
pub enum AttributeFlags {
    None = 0,

    /// If there is a link on the node then it will detatch instead of creating a new one.
    /// Requires handling of deleted links via Context::link_destroyed
    EnableLinkDetachWithDragClick = 1 << 0,

    /// Visual snapping will trigger link creation / destruction
    EnableLinkCreationOnSnap = 1 << 1
}

#[derive(Default, Debug)]
pub (crate) struct PinDataColorStyle {
    pub background: egui::Color32,
    pub hovered: egui::Color32
}

#[derive(Derivative)]
#[derivative(Debug)]
pub (crate) struct PinData {
    pub id: usize,
    pub parent_node_idx: usize,
    pub attribute_rect: egui::Rect,
    pub kind: AttributeType,
    pub shape: PinShape,
    pub pos: egui::Pos2,
    pub flags: usize,
    #[derivative(Debug="ignore")]
    pub color_style: PinDataColorStyle,
    #[derivative(Debug="ignore")]
    pub shape_gui: Option<egui::layers::ShapeIdx>
}

impl Id for PinData {
    fn id(&self) -> usize {
        self.id
    }

    fn new(id: usize) -> Self {
        Self {
            id,
            parent_node_idx: Default::default(),
            attribute_rect: [[0.0; 2].into(); 2].into(),
            kind: AttributeType::None,
            shape: PinShape::CircleFilled,
            pos: Default::default(),
            flags: AttributeFlags::None as usize,
            color_style: Default::default(),
            shape_gui: None
        }
    }
}

impl Default for PinData {
    fn default() -> Self {
        Self::new(0)
    }
}