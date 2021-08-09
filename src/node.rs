use derivative::Derivative;
use super::*;

#[derive(Default, Debug)]
/// The Style of a Node. If feilds are None then the Context style is used
pub struct NodeArgs {
    pub background: Option<egui::Color32>,
    pub background_hovered: Option<egui::Color32>,
    pub background_selected: Option<egui::Color32>,
    pub outline: Option<egui::Color32>,
    pub titlebar: Option<egui::Color32>,
    pub titlebar_hovered: Option<egui::Color32>,
    pub titlebar_selected: Option<egui::Color32>,
    pub corner_rounding: Option<f32>,
    pub padding: Option<egui::Vec2>,
    pub border_thickness: Option<f32>
}

impl NodeArgs {
    pub const fn new() -> Self {
        Self {
            background: None,
            background_hovered: None,
            background_selected: None,
            outline: None,
            titlebar: None,
            titlebar_hovered: None,
            titlebar_selected: None,
            corner_rounding: None,
            padding: None,
            border_thickness: None
        }
    }
}

#[derive(Default, Debug)]
pub (crate) struct NodeDataColorStyle {
    pub background: egui::Color32,
    pub background_hovered: egui::Color32,
    pub background_selected: egui::Color32,
    pub outline: egui::Color32,
    pub titlebar: egui::Color32,
    pub titlebar_hovered: egui::Color32,
    pub titlebar_selected: egui::Color32
}

#[derive(Default, Debug)]
pub struct NodeDataLayoutStyle {
    pub corner_rounding: f32,
    pub padding: egui::Vec2,
    pub border_thickness: f32
}

#[derive(Derivative)]
#[derivative(Debug)]
pub (crate) struct NodeData {
    pub id: usize,
    pub origin: egui::Pos2,
    pub size: egui::Vec2,
    pub title_bar_content_rect: egui::Rect,
    pub rect: egui::Rect,
    #[derivative(Debug="ignore")]
    pub color_style: NodeDataColorStyle,
    pub layout_style: NodeDataLayoutStyle,
    pub pin_indices: Vec<usize>,
    pub draggable: bool,
    #[derivative(Debug="ignore")]
    pub titlebar_shape: Option<egui::layers::ShapeIdx>,
    #[derivative(Debug="ignore")]
    pub background_shape: Option<egui::layers::ShapeIdx>,
    #[derivative(Debug="ignore")]
    pub outline_shape: Option<egui::layers::ShapeIdx>,
}

impl NodeData {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            origin: [100.0; 2].into(),
            size: [100.0; 2].into(),
            title_bar_content_rect: [[0.0; 2].into(); 2].into(),
            rect:  [[0.0; 2].into(); 2].into(),
            color_style: Default::default(),
            layout_style: Default::default(),
            pin_indices: Default::default(),
            draggable: true,
            titlebar_shape: None,
            background_shape: None,
            outline_shape: None
        }
    }

    #[inline]
    pub fn get_node_title_rect(&self) -> egui::Rect {
        let expanded_title_rect = self.title_bar_content_rect.expand2(self.layout_style.padding);
        egui::Rect::from_min_max(
            expanded_title_rect.min,
            expanded_title_rect.min + egui::vec2(self.rect.width(), expanded_title_rect.height())
        )
    }
}

impl Default for NodeData {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Used to construct a node and stores the relevant ui code for its title and attributes
/// This is used so that the nodes can be rendered in the context depth order
#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct NodeConstructor<'a> {
    //node: &'a mut NodeData,
    pub(crate) id: usize,
    #[derivative(Debug="ignore")]
    pub(crate) title: Option<Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + 'a>>,
    #[derivative(Debug="ignore")]
    pub(crate) attributes: Vec<(usize, AttributeType, PinArgs, Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + 'a>)>,
    pub(crate) pos: Option<egui::Pos2>,
    pub(crate) args: NodeArgs
}

impl<'a, 'b> NodeConstructor<'a> {
    /// Create a new node to be displayed in a Context. 
    /// id should be the same accross frames and should not be the same as any other currently used nodes
    pub fn new(id: usize, args: NodeArgs) -> Self {
        Self {id, args, ..Default::default()}
    }  

    /// Add a title to a node
    pub fn with_title(mut self, title: impl FnOnce(&mut egui::Ui) -> egui::Response + 'a) -> Self {
        self.title.replace(Box::new(title));
        self
    }

    /// Add an input attibute to a node, this attribute can be connected to output attributes of other nodes
    /// id should be the same accross frames and should not be the same as any other currently used attributes
    /// the attribute should return a egui::Response to be checked for interaction
    pub fn with_input_attribute(mut self, id: usize, args: PinArgs, attribute: impl FnOnce(&mut egui::Ui) -> egui::Response + 'a) -> Self {
        self.attributes.push((id, AttributeType::Input, args, Box::new(attribute)));
        self
    }
    /// Add an output attibute to a node, this attribute can be connected to input attributes of other nodes
    /// id should be the same accross frames and should not be the same as any other currently used attributes
    /// the attribute should return a egui::Response to be checked for interaction
    pub fn with_output_attribute(mut self, id: usize, args: PinArgs, attribute: impl FnOnce(&mut egui::Ui) -> egui::Response + 'a) -> Self {
        self.attributes.push((id, AttributeType::Output, args, Box::new(attribute)));
        self
    }
    /// Add a static attibute to a node, this attribute can't be connected to any other attributes
    /// id should be the same accross frames and should not be the same as any other currently used attributes
    /// the attribute should return a egui::Response to be checked for interaction
    pub fn with_static_attribute(mut self, id: usize, attribute: impl FnOnce(&mut egui::Ui) -> egui::Response + 'a) -> Self {
        self.attributes.push((id, AttributeType::None, PinArgs::default(), Box::new(attribute)));
        self
    }
    /// Set the position of the node in screen space when it is first created.
    /// To modify it after creation use one of the set_node_pos methods of the Context
    pub fn with_origin(mut self, origin: egui::Pos2) -> Self {
        self.pos.replace(origin);
        self
    }
    /// Get the id of this NodeConstructor
    pub fn id(&self) -> usize {
        self.id
    }
}