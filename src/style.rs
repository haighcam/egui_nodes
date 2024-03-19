use super::*;

/// Represents different color style values used by a Context
#[derive(Debug, Clone, Copy)]
pub enum ColorStyle {
    NodeBackground = 0,
    NodeBackgroundHovered,
    NodeBackgroundSelected,
    NodeOutline,
    TitleBar,
    TitleBarHovered,
    TitleBarSelected,
    Link,
    LinkHovered,
    LinkSelected,
    Pin,
    PinHovered,
    BoxSelector,
    BoxSelectorOutline,
    GridBackground,
    GridLine,
    Count,
}

/// Represents different style values used by a Context
#[derive(Debug, Clone, Copy)]
pub enum StyleVar {
    GridSpacing = 0,
    NodeCornerRounding,
    NodePaddingHorizontal,
    NodePaddingVertical,
    NodeBorderThickness,
    LinkThickness,
    LinkLineSegmentsPerLength,
    LinkHoverDistance,
    PinCircleRadius,
    PinQuadSideLength,
    PinTriangleSideLength,
    PinLineThickness,
    PinHoverRadius,
    PinOffset,
}

/// Controls some style aspects
#[derive(Debug)]
pub enum StyleFlags {
    None = 0,
    NodeOutline = 1 << 0,
    GridLines = 1 << 2,
}

impl ColorStyle {
    /// dark color style
    pub fn colors_dark() -> [egui::Color32; ColorStyle::Count as usize] {
        let mut colors = [egui::Color32::BLACK; ColorStyle::Count as usize];
        colors[ColorStyle::NodeBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(50, 50, 50, 255);
        colors[ColorStyle::NodeBackgroundHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeBackgroundSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 255);
        colors[ColorStyle::TitleBar as usize] =
            egui::Color32::from_rgba_unmultiplied(41, 74, 122, 255);
        colors[ColorStyle::TitleBarHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::TitleBarSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::Link as usize] =
            egui::Color32::from_rgba_unmultiplied(61, 133, 224, 200);
        colors[ColorStyle::LinkHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::LinkSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::Pin as usize] = egui::Color32::from_rgba_unmultiplied(53, 150, 250, 180);
        colors[ColorStyle::PinHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(53, 150, 250, 255);
        colors[ColorStyle::BoxSelector as usize] =
            egui::Color32::from_rgba_unmultiplied(61, 133, 224, 30);
        colors[ColorStyle::BoxSelectorOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(61, 133, 224, 150);
        colors[ColorStyle::GridBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(40, 40, 50, 200);
        colors[ColorStyle::GridLine as usize] =
            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 40);
        colors
    }

    /// classic color style
    pub fn colors_classic() -> [egui::Color32; ColorStyle::Count as usize] {
        let mut colors = [egui::Color32::BLACK; ColorStyle::Count as usize];
        colors[ColorStyle::NodeBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(50, 50, 50, 255);
        colors[ColorStyle::NodeBackgroundHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeBackgroundSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255);
        colors[ColorStyle::NodeOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 255);
        colors[ColorStyle::TitleBar as usize] =
            egui::Color32::from_rgba_unmultiplied(69, 69, 138, 255);
        colors[ColorStyle::TitleBarHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 255);
        colors[ColorStyle::TitleBarSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 255);
        colors[ColorStyle::Link as usize] =
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100);
        colors[ColorStyle::LinkHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(105, 99, 204, 153);
        colors[ColorStyle::LinkSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(105, 99, 204, 153);
        colors[ColorStyle::Pin as usize] = egui::Color32::from_rgba_unmultiplied(89, 102, 156, 170);
        colors[ColorStyle::PinHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(102, 122, 179, 200);
        colors[ColorStyle::BoxSelector as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 100);
        colors[ColorStyle::BoxSelectorOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(82, 82, 161, 255);
        colors[ColorStyle::GridBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(40, 40, 50, 200);
        colors[ColorStyle::GridLine as usize] =
            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 40);
        colors
    }

    /// light color style
    pub fn colors_light() -> [egui::Color32; ColorStyle::Count as usize] {
        let mut colors = [egui::Color32::BLACK; ColorStyle::Count as usize];
        colors[ColorStyle::NodeBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(240, 240, 240, 255);
        colors[ColorStyle::NodeBackgroundHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(240, 240, 240, 255);
        colors[ColorStyle::NodeBackgroundSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(240, 240, 240, 255);
        colors[ColorStyle::NodeOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 255);
        colors[ColorStyle::TitleBar as usize] =
            egui::Color32::from_rgba_unmultiplied(248, 248, 248, 255);
        colors[ColorStyle::TitleBarHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(209, 209, 209, 255);
        colors[ColorStyle::TitleBarSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(209, 209, 209, 255);
        colors[ColorStyle::Link as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 100);
        colors[ColorStyle::LinkHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 242);
        colors[ColorStyle::LinkSelected as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 242);
        colors[ColorStyle::Pin as usize] = egui::Color32::from_rgba_unmultiplied(66, 150, 250, 160);
        colors[ColorStyle::PinHovered as usize] =
            egui::Color32::from_rgba_unmultiplied(66, 150, 250, 255);
        colors[ColorStyle::BoxSelector as usize] =
            egui::Color32::from_rgba_unmultiplied(90, 170, 250, 30);
        colors[ColorStyle::BoxSelectorOutline as usize] =
            egui::Color32::from_rgba_unmultiplied(90, 170, 250, 150);
        colors[ColorStyle::GridBackground as usize] =
            egui::Color32::from_rgba_unmultiplied(225, 225, 225, 255);
        colors[ColorStyle::GridLine as usize] =
            egui::Color32::from_rgba_unmultiplied(180, 180, 180, 100);
        colors
    }
}

/// The style used by a context
/// Example:
/// ``` rust
/// # use egui_nodes::{Context, Style, ColorStyle};
/// let mut ctx = Context::default();
/// let style = Style { colors: ColorStyle::colors_classic(), ..Default::default() };
/// ctx.style = style;
/// ```
#[derive(Debug)]
pub struct Style {
    pub grid_spacing: f32,
    pub node_corner_rounding: f32,
    pub node_padding_horizontal: f32,
    pub node_padding_vertical: f32,
    pub node_border_thickness: f32,

    pub link_thickness: f32,
    pub link_line_segments_per_length: f32,
    pub link_hover_distance: f32,

    pub pin_circle_radius: f32,
    pub pin_quad_side_length: f32,
    pub pin_triangle_side_length: f32,
    pub pin_line_thickness: f32,
    pub pin_hover_radius: f32,
    pub pin_offset: f32,

    pub flags: usize,
    pub colors: [egui::Color32; ColorStyle::Count as usize],
}

impl Default for Style {
    fn default() -> Self {
        Self {
            grid_spacing: 32.0,
            node_corner_rounding: 4.0,
            node_padding_horizontal: 8.0,
            node_padding_vertical: 8.0,
            node_border_thickness: 1.0,
            link_thickness: 3.0,
            link_line_segments_per_length: 0.1,
            link_hover_distance: 10.0,
            pin_circle_radius: 4.0,
            pin_quad_side_length: 7.0,
            pin_triangle_side_length: 9.5,
            pin_line_thickness: 1.0,
            pin_hover_radius: 10.0,
            pin_offset: 0.0,
            flags: StyleFlags::NodeOutline as usize | StyleFlags::GridLines as usize,
            colors: ColorStyle::colors_dark(),
        }
    }
}

impl Style {
    pub(crate) fn get_screen_space_pin_coordinates(
        &self,
        node_rect: &egui::Rect,
        attribute_rect: &egui::Rect,
        kind: AttributeType,
    ) -> egui::Pos2 {
        let x = match kind {
            AttributeType::Input => node_rect.min.x - self.pin_offset,
            _ => node_rect.max.x + self.pin_offset,
        };
        egui::pos2(x, 0.5 * (attribute_rect.min.y + attribute_rect.max.y))
    }

    pub(crate) fn draw_pin_shape(
        &self,
        pin_pos: egui::Pos2,
        pin_shape: PinShape,
        pin_color: egui::Color32,
        shape: egui::layers::ShapeIdx,
        ui: &mut egui::Ui,
    ) {
        let painter = ui.painter();
        match pin_shape {
            PinShape::Circle => painter.set(
                shape,
                egui::Shape::circle_stroke(
                    pin_pos,
                    self.pin_circle_radius,
                    (self.pin_line_thickness, pin_color),
                ),
            ),
            PinShape::CircleFilled => painter.set(
                shape,
                egui::Shape::circle_filled(pin_pos, self.pin_circle_radius, pin_color),
            ),
            PinShape::Quad => painter.set(
                shape,
                egui::Shape::rect_stroke(
                    egui::Rect::from_center_size(
                        pin_pos,
                        [self.pin_quad_side_length / 2.0; 2].into(),
                    ),
                    0.0,
                    (self.pin_line_thickness, pin_color),
                ),
            ),
            PinShape::QuadFilled => painter.set(
                shape,
                egui::Shape::rect_filled(
                    egui::Rect::from_center_size(
                        pin_pos,
                        [self.pin_quad_side_length / 2.0; 2].into(),
                    ),
                    0.0,
                    pin_color,
                ),
            ),
            PinShape::Triangle => {
                let sqrt_3 = 3f32.sqrt();
                let left_offset = -0.166_666_7 * sqrt_3 * self.pin_triangle_side_length;
                let right_offset = 0.333_333_3 * sqrt_3 * self.pin_triangle_side_length;
                let verticacl_offset = 0.5 * self.pin_triangle_side_length;
                painter.set(
                    shape,
                    egui::Shape::closed_line(
                        vec![
                            pin_pos + (left_offset, verticacl_offset).into(),
                            pin_pos + (right_offset, 0.0).into(),
                            pin_pos + (left_offset, -verticacl_offset).into(),
                        ],
                        (self.pin_line_thickness, pin_color),
                    ),
                )
            }
            PinShape::TriangleFilled => {
                let sqrt_3 = 3f32.sqrt();
                let left_offset = -0.166_666_7 * sqrt_3 * self.pin_triangle_side_length;
                let right_offset = 0.333_333_3 * sqrt_3 * self.pin_triangle_side_length;
                let verticacl_offset = 0.5 * self.pin_triangle_side_length;
                painter.set(
                    shape,
                    egui::Shape::convex_polygon(
                        vec![
                            pin_pos + (left_offset, verticacl_offset).into(),
                            pin_pos + (right_offset, 0.0).into(),
                            pin_pos + (left_offset, -verticacl_offset).into(),
                        ],
                        pin_color,
                        egui::Stroke::default(),
                    ),
                )
            }
        }
    }

    pub(crate) fn format_node(&self, node: &mut NodeData, args: NodeArgs) {
        node.color_style.background =
            args.background.unwrap_or(self.colors[ColorStyle::NodeBackground as usize]);
        node.color_style.background_hovered = args
            .background_hovered
            .unwrap_or(self.colors[ColorStyle::NodeBackgroundHovered as usize]);
        node.color_style.background_selected = args
            .background_selected
            .unwrap_or(self.colors[ColorStyle::NodeBackgroundSelected as usize]);
        node.color_style.outline =
            args.outline.unwrap_or(self.colors[ColorStyle::NodeOutline as usize]);
        node.color_style.titlebar =
            args.titlebar.unwrap_or(self.colors[ColorStyle::TitleBar as usize]);
        node.color_style.titlebar_hovered =
            args.titlebar_hovered.unwrap_or(self.colors[ColorStyle::TitleBarHovered as usize]);
        node.color_style.titlebar_selected =
            args.titlebar_selected.unwrap_or(self.colors[ColorStyle::TitleBarSelected as usize]);
        node.layout_style.corner_rounding =
            args.corner_rounding.unwrap_or(self.node_corner_rounding);
        node.layout_style.padding = args.padding.unwrap_or_else(|| {
            egui::vec2(self.node_padding_horizontal, self.node_padding_vertical)
        });
        node.layout_style.border_thickness =
            args.border_thickness.unwrap_or(self.node_border_thickness);
    }

    pub(crate) fn format_pin(&self, pin: &mut PinData, args: PinArgs, flags: usize) {
        pin.shape = args.shape;
        pin.flags = args.flags.unwrap_or(flags);
        pin.color_style.background =
            args.background.unwrap_or(self.colors[ColorStyle::Pin as usize]);
        pin.color_style.hovered =
            args.hovered.unwrap_or(self.colors[ColorStyle::PinHovered as usize]);
    }

    pub(crate) fn format_link(&self, link: &mut LinkData, args: LinkArgs) {
        link.color_style.base = args.base.unwrap_or(self.colors[ColorStyle::Link as usize]);
        link.color_style.hovered =
            args.hovered.unwrap_or(self.colors[ColorStyle::LinkHovered as usize]);
        link.color_style.selected =
            args.selected.unwrap_or(self.colors[ColorStyle::LinkSelected as usize]);
    }
}
