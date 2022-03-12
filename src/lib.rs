//! egui_nodes: A Egui port of [imnodes](https://github.com/Nelarius/imnodes)
//!
//! # Using egui_nodes
//!
//! There are some simple examples [here](https://github.com/haighcam/egui_nodes/examples)
//!
//! Here is the basic usage:
//! ``` rust
//! use egui_nodes::{Context, NodeConstructor, LinkArgs};
//! use egui::Ui;
//!
//! pub fn example_graph(ctx: &mut Context, links: &mut Vec<(usize, usize)>, ui: &mut Ui) {
//!     // add nodes with attributes
//!     let nodes = vec![
//!         NodeConstructor::new(0, Default::default())
//!             .with_title(|ui| ui.label("Example Node A"))
//!             .with_input_attribute(0, Default::default(), |ui| ui.label("Input"))
//!             .with_static_attribute(1, |ui| ui.label("Can't Connect to Me"))
//!             .with_output_attribute(2, Default::default(), |ui| ui.label("Output")),
//!         NodeConstructor::new(1, Default::default())
//!             .with_title(|ui| ui.label("Example Node B"))
//!             .with_static_attribute(3, |ui| ui.label("Can't Connect to Me"))
//!             .with_output_attribute(4, Default::default(), |ui| ui.label("Output"))
//!             .with_input_attribute(5, Default::default(), |ui| ui.label("Input"))
//!     ];
//!
//!     // add them to the ui
//!     ctx.show(
//!         nodes,
//!         links.iter().enumerate().map(|(i, (start, end))| (i, *start, *end, LinkArgs::default())),
//!         ui
//!     );
//!     
//!     // remove destroyed links
//!     if let Some(idx) = ctx.link_destroyed() {
//!         links.remove(idx);
//!     }
//!
//!     // add created links
//!     if let Some((start, end, _)) = ctx.link_created() {
//!         links.push((start, end))
//!     }
//! }
//! ```

use derivative::Derivative;
use std::collections::HashMap;

mod link;
mod node;
mod pin;
mod style;

use link::*;
use node::*;
use pin::*;

pub use {
    link::LinkArgs,
    node::{NodeArgs, NodeConstructor},
    pin::{AttributeFlags, PinArgs, PinShape},
    style::{ColorStyle, Style, StyleFlags, StyleVar},
};

/// The Context that tracks the state of the node editor
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct Context {
    node_idx_submission_order: Vec<usize>,
    node_indices_overlapping_with_mouse: Vec<usize>,
    occluded_pin_indices: Vec<usize>,

    canvas_origin_screen_space: egui::Vec2,
    #[derivative(Default(value = "[[0.0; 2].into(); 2].into()"))]
    canvas_rect_screen_space: egui::Rect,

    #[derivative(Debug = "ignore")]
    pub io: IO,
    #[derivative(Debug = "ignore")]
    pub style: Style,
    color_modifier_stack: Vec<ColorStyleElement>,
    style_modifier_stack: Vec<StyleElement>,
    text_buffer: String,

    current_attribute_flags: usize,
    attribute_flag_stack: Vec<usize>,

    hovered_node_index: Option<usize>,
    interactive_node_index: Option<usize>,
    hovered_link_idx: Option<usize>,
    hovered_pin_index: Option<usize>,
    hovered_pin_flags: usize,
    ui_element_hovered: bool,

    deleted_link_idx: Option<usize>,
    snap_link_idx: Option<usize>,

    element_state_change: usize,

    active_attribute_id: usize,
    active_attribute: bool,

    mouse_pos: egui::Pos2,
    mouse_delta: egui::Vec2,

    left_mouse_clicked: bool,
    left_mouse_released: bool,
    alt_mouse_clicked: bool,
    left_mouse_dragging: bool,
    alt_mouse_dragging: bool,
    mouse_in_canvas: bool,
    link_detatch_with_modifier_click: bool,

    nodes: ObjectPool<NodeData>,
    pins: ObjectPool<PinData>,
    links: ObjectPool<LinkData>,
    nodes_map: HashMap<usize, usize>,
    nodes_free: Vec<usize>,

    node_depth_order: Vec<usize>,

    panning: egui::Vec2,

    selected_node_indices: Vec<usize>,
    selected_link_indices: Vec<usize>,

    #[derivative(Default(value = "ClickInteractionType::None"))]
    click_interaction_type: ClickInteractionType,
    click_interaction_state: ClickInteractionState,
}

impl Context {
    /// Displays the current state of the editor on a give Egui Ui as well as updating user input to the context
    pub fn show<'a>(
        &mut self,
        nodes: impl IntoIterator<Item = NodeConstructor<'a>>,
        links: impl IntoIterator<Item = (usize, usize, usize, LinkArgs)>,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        let rect = ui.available_rect_before_wrap();
        self.canvas_rect_screen_space = rect;
        self.canvas_origin_screen_space = self.canvas_rect_screen_space.min.to_vec2();
        {
            self.nodes.reset();
            self.pins.reset();
            self.links.reset();

            self.hovered_node_index.take();
            self.interactive_node_index.take();
            self.hovered_link_idx.take();
            self.hovered_pin_flags = AttributeFlags::None as usize;
            self.deleted_link_idx.take();
            self.snap_link_idx.take();

            self.node_indices_overlapping_with_mouse.clear();
            self.element_state_change = ElementStateChange::None as usize;

            self.active_attribute = false;
        }

        {
            ui.set_min_size(self.canvas_rect_screen_space.size());
            let mut ui = ui.child_ui(
                self.canvas_rect_screen_space,
                egui::Layout::top_down(egui::Align::Center),
            );
            {
                let ui = &mut ui;
                let rect = ui.ctx().input().screen_rect();
                ui.set_clip_rect(
                    self.canvas_rect_screen_space.intersect(rect),
                );
                ui.painter().rect_filled(
                    self.canvas_rect_screen_space,
                    0.0,
                    self.style.colors[ColorStyle::GridBackground as usize],
                );

                if (self.style.flags & StyleFlags::GridLines as usize) != 0 {
                    self.draw_grid(self.canvas_rect_screen_space.size(), ui);
                }

                let links = links.into_iter().collect::<Vec<_>>();
                for (id, start, end, args) in links {
                    self.add_link(id, start, end, args, ui);
                }

                let mut nodes = nodes
                    .into_iter()
                    .map(|x| (self.node_pool_find_or_create_index(x.id, x.pos), x))
                    .collect::<HashMap<_, _>>();
                for idx in self.node_depth_order.clone() {
                    if let Some(node_builder) = nodes.remove(&idx) {
                        self.add_node(idx, node_builder, ui);
                    }
                }
            }
            let response = ui.interact(
                self.canvas_rect_screen_space,
                ui.id().with("Input"),
                egui::Sense::click_and_drag(),
            );
            {
                let io = ui.ctx().input();
                let mouse_pos = if let Some(mouse_pos) = response.hover_pos() {
                    self.mouse_in_canvas = true;
                    mouse_pos
                } else {
                    self.mouse_in_canvas = false;
                    self.mouse_pos
                };
                self.mouse_delta = mouse_pos - self.mouse_pos;
                self.mouse_pos = mouse_pos;
                let left_mouse_clicked = io.pointer.button_down(egui::PointerButton::Primary);
                self.left_mouse_released =
                    (self.left_mouse_clicked || self.left_mouse_dragging) && !left_mouse_clicked;
                self.left_mouse_dragging =
                    (self.left_mouse_clicked || self.left_mouse_dragging) && left_mouse_clicked;
                self.left_mouse_clicked =
                    left_mouse_clicked && !(self.left_mouse_clicked || self.left_mouse_dragging);

                let alt_mouse_clicked = self.io.emulate_three_button_mouse.is_active(&io.modifiers)
                    || self.io.alt_mouse_button.map_or(false, |x| io.pointer.button_down(x));
                self.alt_mouse_dragging =
                    (self.alt_mouse_clicked || self.alt_mouse_dragging) && alt_mouse_clicked;
                self.alt_mouse_clicked =
                    alt_mouse_clicked && !(self.alt_mouse_clicked || self.alt_mouse_dragging);
                self.link_detatch_with_modifier_click =
                    self.io.link_detatch_with_modifier_click.is_active(&io.modifiers);
            }
            {
                let ui = &mut ui;
                if self.mouse_in_canvas {
                    self.resolve_occluded_pins();
                    self.resolve_hovered_pin();

                    if self.hovered_pin_index.is_none() {
                        self.resolve_hovered_node();
                    }

                    if self.hovered_node_index.is_none() {
                        self.resolve_hovered_link();
                    }
                }

                for node_idx in self.node_depth_order.clone() {
                    if self.nodes.in_use[node_idx] {
                        self.draw_node(node_idx, ui);
                    }
                }

                for (link_idx, in_use) in self.links.in_use.clone().into_iter().enumerate() {
                    if in_use {
                        self.draw_link(link_idx, ui);
                    }
                }

                if self.left_mouse_clicked || self.alt_mouse_clicked {
                    self.begin_canvas_interaction();
                }

                self.click_interaction_update(ui);

                self.node_pool_update();
                self.pins.update();
                self.links.update();
            }
            ui.painter().rect_stroke(
                self.canvas_rect_screen_space,
                0.0,
                (1.0, self.style.colors[ColorStyle::GridLine as usize]),
            );
            response
        }
    }

    /// Push a sigle AttributeFlags value, by default only None is set.
    /// Used for pins that don't have a specific attribute flag specified
    pub fn attribute_flag_push(&mut self, flag: AttributeFlags) {
        self.attribute_flag_stack.push(self.current_attribute_flags);
        self.current_attribute_flags |= flag as usize;
    }

    /// Remove the last added AttributeFlags value
    pub fn attribute_flag_pop(&mut self) {
        if let Some(flags) = self.attribute_flag_stack.pop() {
            self.current_attribute_flags = flags;
        }
    }

    /// Changes the current colors used by the editor
    pub fn color_style_push(&mut self, item: ColorStyle, color: egui::Color32) {
        self.color_modifier_stack.push(ColorStyleElement::new(
            self.style.colors[item as usize],
            item,
        ));
        self.style.colors[item as usize] = color;
    }

    /// Revert the last color change
    pub fn color_style_pop(&mut self) {
        if let Some(elem) = self.color_modifier_stack.pop() {
            self.style.colors[elem.item as usize] = elem.color;
        }
    }

    /// Change a context style value
    pub fn style_var_push(&mut self, item: StyleVar, value: f32) {
        let style_var = self.lookup_style_var(item);
        let elem = StyleElement::new(*style_var, item);
        *style_var = value;
        self.style_modifier_stack.push(elem);
    }

    /// Revert the last context style change
    pub fn style_var_pop(&mut self) {
        if let Some(elem) = self.style_modifier_stack.pop() {
            let style_var = self.lookup_style_var(elem.item);
            *style_var = elem.value;
        }
    }

    pub fn set_node_pos_screen_space(&mut self, node_id: usize, screen_space_pos: egui::Pos2) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].origin = self.screen_space_to_grid_space(screen_space_pos);
    }

    pub fn set_node_pos_editor_space(&mut self, node_id: usize, editor_space_pos: egui::Pos2) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].origin = self.editor_space_to_grid_spcae(editor_space_pos);
    }

    pub fn set_node_pos_grid_space(&mut self, node_id: usize, grid_pos: egui::Pos2) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].origin = grid_pos;
    }

    pub fn set_node_draggable(&mut self, node_id: usize, draggable: bool) {
        let idx = self.node_pool_find_or_create_index(node_id, None);
        self.nodes.pool[idx].draggable = draggable;
    }

    pub fn get_node_pos_screen_space(&self, node_id: usize) -> Option<egui::Pos2> {
        self.nodes.find(node_id).map(|x| self.grid_space_to_screen_space(self.nodes.pool[x].origin))
    }

    pub fn get_node_pos_editor_space(&self, node_id: usize) -> Option<egui::Pos2> {
        self.nodes.find(node_id).map(|x| self.grid_space_to_editor_spcae(self.nodes.pool[x].origin))
    }

    pub fn get_node_pos_grid_space(&self, node_id: usize) -> Option<egui::Pos2> {
        self.nodes.find(node_id).map(|x| self.nodes.pool[x].origin)
    }

    /// Check if there is a node that is hovered by the pointer
    pub fn node_hovered(&self) -> Option<usize> {
        self.hovered_node_index.map(|x| self.nodes.pool[x].id)
    }

    /// Check if there is a link that is hovered by the pointer
    pub fn link_hovered(&self) -> Option<usize> {
        self.hovered_link_idx.map(|x| self.links.pool[x].id)
    }

    /// Check if there is a pin that is hovered by the pointer
    pub fn pin_hovered(&self) -> Option<usize> {
        self.hovered_pin_index.map(|x| self.pins.pool[x].id)
    }

    pub fn num_selected_nodes(&self) -> usize {
        self.selected_link_indices.len()
    }

    pub fn get_selected_nodes(&self) -> Vec<usize> {
        self.selected_node_indices.iter().map(|x| self.nodes.pool[*x].id).collect()
    }

    pub fn get_selected_links(&self) -> Vec<usize> {
        self.selected_link_indices.iter().map(|x| self.links.pool[*x].id).collect()
    }

    pub fn clear_node_selection(&mut self) {
        self.selected_node_indices.clear()
    }

    pub fn clear_link_selection(&mut self) {
        self.selected_link_indices.clear()
    }

    /// Check if an attribute is currently being interacted with
    pub fn active_attribute(&self) -> Option<usize> {
        if self.active_attribute {
            Some(self.active_attribute_id)
        } else {
            None
        }
    }

    /// Has a new link been created from a pin?
    pub fn link_started(&self) -> Option<usize> {
        if (self.element_state_change & ElementStateChange::LinkStarted as usize) != 0 {
            Some(self.pins.pool[self.click_interaction_state.link_creation.start_pin_idx].id)
        } else {
            None
        }
    }

    /// Has a link been dropped? if including_detached_links then links that were detached then dropped are included
    pub fn link_dropped(&self, including_detached_links: bool) -> Option<usize> {
        if (self.element_state_change & ElementStateChange::LinkDropped as usize) != 0
            && (including_detached_links
                || self.click_interaction_state.link_creation.link_creation_type
                    != LinkCreationType::FromDetach)
        {
            Some(self.pins.pool[self.click_interaction_state.link_creation.start_pin_idx].id)
        } else {
            None
        }
    }

    /// Has a new link been created?
    /// -> Option<start_pin, end_pin created_from_snap>
    pub fn link_created(&self) -> Option<(usize, usize, bool)> {
        if (self.element_state_change & ElementStateChange::LinkCreated as usize) != 0 {
            let (start_pin_id, end_pin_id) = {
                let start_pin =
                    &self.pins.pool[self.click_interaction_state.link_creation.start_pin_idx];
                let end_pin = &self.pins.pool
                    [self.click_interaction_state.link_creation.end_pin_index.unwrap()];
                if start_pin.kind == AttributeType::Output {
                    (start_pin.id, end_pin.id)
                } else {
                    (end_pin.id, start_pin.id)
                }
            };
            let created_from_snap =
                self.click_interaction_type == ClickInteractionType::LinkCreation;
            Some((start_pin_id, end_pin_id, created_from_snap))
        } else {
            None
        }
    }

    /// Has a new link been created? Includes start and end node
    /// -> Option<start_pin, start_node, end_pin, end_node created_from_snap>
    pub fn link_created_node(&self) -> Option<(usize, usize, usize, usize, bool)> {
        if (self.element_state_change & ElementStateChange::LinkCreated as usize) != 0 {
            let (start_pin_id, start_node_id, end_pin_id, end_node_id) = {
                let start_pin =
                    &self.pins.pool[self.click_interaction_state.link_creation.start_pin_idx];
                let end_pin = &self.pins.pool
                    [self.click_interaction_state.link_creation.end_pin_index.unwrap()];
                let start_node = &self.nodes.pool[start_pin.parent_node_idx];
                let end_node = &self.nodes.pool[end_pin.parent_node_idx];
                if start_pin.kind == AttributeType::Output {
                    (start_pin.id, start_node.id, end_pin.id, end_node.id)
                } else {
                    (end_pin.id, end_node.id, start_pin.id, start_node.id)
                }
            };
            let created_from_snap =
                self.click_interaction_type == ClickInteractionType::LinkCreation;
            Some((
                start_pin_id,
                start_node_id,
                end_pin_id,
                end_node_id,
                created_from_snap,
            ))
        } else {
            None
        }
    }

    // Was an existing link detached?
    pub fn link_destroyed(&self) -> Option<usize> {
        self.deleted_link_idx
    }

    pub fn get_panning(&self) -> egui::Vec2 {
        self.panning
    }

    pub fn reset_panniing(&mut self, panning: egui::Vec2) {
        self.panning = panning;
    }

    pub fn get_node_dimensions(&self, id: usize) -> Option<egui::Vec2> {
        self.nodes.find(id).map(|x| self.nodes.pool[x].rect.size())
    }
}

impl Context {
    fn add_node<'a>(
        &mut self,
        idx: usize,
        NodeConstructor {
            id,
            title,
            attributes,
            pos: _,
            args,
        }: NodeConstructor<'a>,
        ui: &mut egui::Ui,
    ) {
        let node = &mut self.nodes.pool[idx];
        self.style.format_node(node, args);
        node.background_shape.replace(ui.painter().add(egui::Shape::Noop));
        node.id = id;
        let node_origin = node.origin;
        let node_size = node.size;
        let title_space = node.layout_style.padding.y;

        let response = ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(self.grid_space_to_screen_space(node_origin), node_size),
            |ui| {
                let mut title_info = None;
                if let Some(title) = title {
                    let titlebar_shape = ui.painter().add(egui::Shape::Noop);
                    let response = ui.allocate_ui(ui.available_size(), title);
                    let title_bar_content_rect = response.response.rect;
                    title_info.replace((titlebar_shape, title_bar_content_rect));
                    ui.add_space(title_space);
                }
                let outline_shape = ui.painter().add(egui::Shape::Noop);
                for (id, kind, args, attribute) in attributes {
                    let response = ui.allocate_ui(ui.available_size(), attribute);
                    let shape = ui.painter().add(egui::Shape::Noop);
                    let response = response.response.union(response.inner);
                    self.add_attribute(id, kind, args, response, idx, shape);
                }
                (title_info, outline_shape)
            },
        );
        let node = &mut self.nodes.pool[idx];
        let (title_info, outline_shape) = response.inner;
        if let Some((titlebar_shape, title_bar_content_rect)) = title_info {
            node.titlebar_shape.replace(titlebar_shape);
            node.title_bar_content_rect = title_bar_content_rect;
        }
        node.outline_shape.replace(outline_shape);
        node.rect = response.response.rect.expand2(node.layout_style.padding);
        if response.response.hovered() {
            self.node_indices_overlapping_with_mouse.push(idx);
        }
    }

    fn add_attribute(
        &mut self,
        id: usize,
        kind: AttributeType,
        args: PinArgs,
        response: egui::Response,
        node_idx: usize,
        shape: egui::layers::ShapeIdx,
    ) {
        if kind != AttributeType::None {
            let pin_idx = self.pins.find_or_create_index(id);
            let pin = &mut self.pins.pool[pin_idx];
            pin.id = id;
            pin.parent_node_idx = node_idx;
            pin.kind = kind;
            pin.shape_gui.replace(shape);
            self.style.format_pin(pin, args, self.current_attribute_flags);
            self.pins.pool[pin_idx].attribute_rect = response.rect;
            self.nodes.pool[node_idx].pin_indices.push(pin_idx);
        }

        if response.is_pointer_button_down_on() {
            self.active_attribute = true;
            self.active_attribute_id = id;
            self.interactive_node_index.replace(node_idx);
        }
    }

    fn add_link(
        &mut self,
        id: usize,
        start_attr_id: usize,
        end_attr_id: usize,
        args: LinkArgs,
        ui: &mut egui::Ui,
    ) {
        let link_idx = self.links.find_or_create_index(id);
        let link = &mut self.links.pool[link_idx];
        link.id = id;
        link.start_pin_index = self.pins.find_or_create_index(start_attr_id);
        link.end_pin_index = self.pins.find_or_create_index(end_attr_id);
        link.shape.replace(ui.painter().add(egui::Shape::Noop));
        self.style.format_link(link, args);

        if (self.click_interaction_type == ClickInteractionType::LinkCreation
            && (self.pins.pool[link.end_pin_index].flags
                & AttributeFlags::EnableLinkCreationOnSnap as usize)
                != 0
            && self.click_interaction_state.link_creation.start_pin_idx == link.start_pin_index
            && self.click_interaction_state.link_creation.end_pin_index == Some(link.end_pin_index))
            || (self.click_interaction_state.link_creation.start_pin_idx == link.end_pin_index
                && self.click_interaction_state.link_creation.end_pin_index
                    == Some(link.start_pin_index))
        {
            self.snap_link_idx.replace(link_idx);
        }
    }

    fn lookup_style_var(&mut self, item: StyleVar) -> &mut f32 {
        match item {
            StyleVar::GridSpacing => &mut self.style.grid_spacing,
            StyleVar::NodeCornerRounding => &mut self.style.node_corner_rounding,
            StyleVar::NodePaddingHorizontal => &mut self.style.node_padding_horizontal,
            StyleVar::NodePaddingVertical => &mut self.style.node_padding_vertical,
            StyleVar::NodeBorderThickness => &mut self.style.node_border_thickness,
            StyleVar::LinkThickness => &mut self.style.link_thickness,
            StyleVar::LinkLineSegmentsPerLength => &mut self.style.link_line_segments_per_length,
            StyleVar::LinkHoverDistance => &mut self.style.link_hover_distance,
            StyleVar::PinCircleRadius => &mut self.style.pin_circle_radius,
            StyleVar::PinQuadSideLength => &mut self.style.pin_quad_side_length,
            StyleVar::PinTriangleSideLength => &mut self.style.pin_triangle_side_length,
            StyleVar::PinLineThickness => &mut self.style.pin_line_thickness,
            StyleVar::PinHoverRadius => &mut self.style.pin_hover_radius,
            StyleVar::PinOffset => &mut self.style.pin_offset,
        }
    }

    fn draw_grid(&self, canvas_size: egui::Vec2, ui: &mut egui::Ui) {
        let mut x = self.panning.x.rem_euclid(self.style.grid_spacing);
        while x < canvas_size.x {
            ui.painter().line_segment(
                [
                    self.editor_space_to_screen_space([x, 0.0].into()),
                    self.editor_space_to_screen_space([x, canvas_size.y].into()),
                ],
                (1.0, self.style.colors[ColorStyle::GridLine as usize]),
            );
            x += self.style.grid_spacing;
        }

        let mut y = self.panning.y.rem_euclid(self.style.grid_spacing);
        while y < canvas_size.y {
            ui.painter().line_segment(
                [
                    self.editor_space_to_screen_space([0.0, y].into()),
                    self.editor_space_to_screen_space([canvas_size.x, y].into()),
                ],
                (1.0, self.style.colors[ColorStyle::GridLine as usize]),
            );
            y += self.style.grid_spacing;
        }
    }

    fn screen_space_to_grid_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v - self.canvas_origin_screen_space - self.panning
    }

    fn grid_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.canvas_origin_screen_space + self.panning
    }

    fn grid_space_to_editor_spcae(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.panning
    }

    fn editor_space_to_grid_spcae(&self, v: egui::Pos2) -> egui::Pos2 {
        v - self.panning
    }

    fn editor_space_to_screen_space(&self, v: egui::Pos2) -> egui::Pos2 {
        v + self.canvas_origin_screen_space
    }

    fn get_screen_space_pin_coordinates(&self, pin: &PinData) -> egui::Pos2 {
        let parent_node_rect = self.nodes.pool[pin.parent_node_idx].rect;
        self.style.get_screen_space_pin_coordinates(
            &parent_node_rect,
            &pin.attribute_rect,
            pin.kind,
        )
    }

    fn resolve_occluded_pins(&mut self) {
        self.occluded_pin_indices.clear();
        let depth_stack = &self.node_depth_order;
        if depth_stack.len() < 2 {
            return;
        }

        for depth_idx in 0..(depth_stack.len() - 1) {
            let node_below = &self.nodes.pool[depth_stack[depth_idx]];
            for next_depth in &depth_stack[(depth_idx + 1)..(depth_stack.len())] {
                let rect_above = self.nodes.pool[*next_depth].rect;
                for idx in node_below.pin_indices.iter() {
                    let pin_pos = self.pins.pool[*idx].pos;
                    if rect_above.contains(pin_pos) {
                        self.occluded_pin_indices.push(*idx);
                    }
                }
            }
        }
    }

    fn resolve_hovered_pin(&mut self) {
        let mut smallest_distance = f32::MAX;
        self.hovered_pin_index.take();

        let hover_radius_sqr = self.style.pin_hover_radius.powi(2);
        for idx in 0..self.pins.pool.len() {
            if !self.pins.in_use[idx] || self.occluded_pin_indices.contains(&idx) {
                continue;
            }

            let pin_pos = self.pins.pool[idx].pos;
            let distance_sqr = (pin_pos - self.mouse_pos).length_sq();
            if distance_sqr < hover_radius_sqr && distance_sqr < smallest_distance {
                smallest_distance = distance_sqr;
                self.hovered_pin_index.replace(idx);
            }
        }
    }

    fn resolve_hovered_node(&mut self) {
        match self.node_indices_overlapping_with_mouse.len() {
            0 => {
                self.hovered_node_index.take();
            }
            1 => {
                self.hovered_node_index.replace(self.node_indices_overlapping_with_mouse[0]);
            }
            _ => {
                let mut largest_depth_idx = -1;

                for node_idx in self.node_indices_overlapping_with_mouse.iter() {
                    for (depth_idx, depth_node_idx) in self.node_depth_order.iter().enumerate() {
                        if *depth_node_idx == *node_idx && depth_idx as isize > largest_depth_idx {
                            largest_depth_idx = depth_idx as isize;
                            self.hovered_node_index.replace(*node_idx);
                        }
                    }
                }
            }
        }
    }

    fn resolve_hovered_link(&mut self) {
        let mut smallest_distance = f32::MAX;
        self.hovered_link_idx.take();

        for idx in 0..self.links.pool.len() {
            if !self.links.in_use[idx] {
                continue;
            }

            let link = &self.links.pool[idx];
            if self.hovered_pin_index == Some(link.start_pin_index)
                || self.hovered_pin_index == Some(link.end_pin_index)
            {
                self.hovered_link_idx.replace(idx);
                return;
            }

            let start_pin = &self.pins.pool[link.start_pin_index];
            let end_pin = &self.pins.pool[link.end_pin_index];

            let link_data = LinkBezierData::get_link_renderable(
                start_pin.pos,
                end_pin.pos,
                start_pin.kind,
                self.style.link_line_segments_per_length,
            );
            let link_rect = link_data
                .bezier
                .get_containing_rect_for_bezier_curve(self.style.link_hover_distance);

            if link_rect.contains(self.mouse_pos) {
                let distance = link_data.get_distance_to_cubic_bezier(&self.mouse_pos);
                if distance < self.style.link_hover_distance && distance < smallest_distance {
                    smallest_distance = distance;
                    self.hovered_link_idx.replace(idx);
                }
            }
        }
    }

    fn draw_link(&mut self, link_idx: usize, ui: &mut egui::Ui) {
        let link = &mut self.links.pool[link_idx];
        let start_pin = &self.pins.pool[link.start_pin_index];
        let end_pin = &self.pins.pool[link.end_pin_index];
        let link_data = LinkBezierData::get_link_renderable(
            start_pin.pos,
            end_pin.pos,
            start_pin.kind,
            self.style.link_line_segments_per_length,
        );
        let link_shape = link.shape.take().unwrap();
        let link_hovered = self.hovered_link_idx == Some(link_idx)
            && self.click_interaction_type != ClickInteractionType::BoxSelection;

        if link_hovered && self.left_mouse_clicked {
            self.begin_link_interaction(link_idx);
        }

        if self.deleted_link_idx == Some(link_idx) {
            return;
        }

        let link = &self.links.pool[link_idx];
        let mut link_color = link.color_style.base;
        if self.selected_link_indices.contains(&link_idx) {
            link_color = link.color_style.selected;
        } else if link_hovered {
            link_color = link.color_style.hovered;
        }

        ui.painter().set(
            link_shape,
            link_data.draw((self.style.link_thickness, link_color)),
        );
    }

    fn draw_node(&mut self, node_idx: usize, ui: &mut egui::Ui) {
        let node = &mut self.nodes.pool[node_idx];

        let node_hovered = self.hovered_node_index == Some(node_idx)
            && self.click_interaction_type != ClickInteractionType::BoxSelection;

        let mut node_background = node.color_style.background;
        let mut titlebar_background = node.color_style.titlebar;

        if self.selected_node_indices.contains(&node_idx) {
            node_background = node.color_style.background_selected;
            titlebar_background = node.color_style.titlebar_selected;
        } else if node_hovered {
            node_background = node.color_style.background_hovered;
            titlebar_background = node.color_style.titlebar_hovered;
        }

        let painter = ui.painter();

        painter.set(
            node.background_shape.take().unwrap(),
            egui::Shape::rect_filled(
                node.rect,
                node.layout_style.corner_rounding,
                node_background,
            ),
        );
        if node.title_bar_content_rect.height() > 0.0 {
            painter.set(
                node.titlebar_shape.take().unwrap(),
                egui::Shape::rect_filled(
                    node.get_node_title_rect(),
                    node.layout_style.corner_rounding,
                    titlebar_background,
                ),
            );
        }
        if (self.style.flags & StyleFlags::NodeOutline as usize) != 0 {
            painter.set(
                node.outline_shape.take().unwrap(),
                egui::Shape::rect_stroke(
                    node.rect,
                    node.layout_style.corner_rounding,
                    (node.layout_style.border_thickness, node.color_style.outline),
                ),
            );
        }

        for pin_idx in node.pin_indices.clone() {
            self.draw_pin(pin_idx, ui);
        }

        if node_hovered && self.left_mouse_clicked && self.interactive_node_index != Some(node_idx)
        {
            self.begin_node_selection(node_idx);
        }
    }

    fn draw_pin(&mut self, pin_idx: usize, ui: &mut egui::Ui) {
        let pin = &mut self.pins.pool[pin_idx];
        let parent_node_rect = self.nodes.pool[pin.parent_node_idx].rect;

        pin.pos = self.style.get_screen_space_pin_coordinates(
            &parent_node_rect,
            &pin.attribute_rect,
            pin.kind,
        );

        let mut pin_color = pin.color_style.background;

        let pin_hovered = self.hovered_pin_index == Some(pin_idx)
            && self.click_interaction_type != ClickInteractionType::BoxSelection;
        let pin_shape = pin.shape;
        let pin_pos = pin.pos;
        let pin_shape_gui = pin
            .shape_gui
            .take()
            .expect("Unable to take pin shape. Perhaps your pin id is not unique?");

        if pin_hovered {
            self.hovered_pin_flags = pin.flags;
            pin_color = pin.color_style.hovered;

            if self.left_mouse_clicked {
                self.begin_link_creation(pin_idx);
            }
        }

        self.style.draw_pin_shape(pin_pos, pin_shape, pin_color, pin_shape_gui, ui);
    }

    fn begin_canvas_interaction(&mut self) {
        let any_ui_element_hovered = self.hovered_node_index.is_some()
            || self.hovered_link_idx.is_some()
            || self.hovered_pin_index.is_some();

        let mouse_not_in_canvas = !self.mouse_in_canvas;

        if self.click_interaction_type != ClickInteractionType::None
            || any_ui_element_hovered
            || mouse_not_in_canvas
        {
            return;
        }

        if self.alt_mouse_clicked {
            self.click_interaction_type = ClickInteractionType::Panning;
        } else {
            self.click_interaction_type = ClickInteractionType::BoxSelection;
            self.click_interaction_state.box_selection.min = self.mouse_pos;
        }
    }

    fn translate_selected_nodes(&mut self) {
        if self.left_mouse_dragging {
            let delta = self.mouse_delta;
            for idx in self.selected_node_indices.iter() {
                let node = &mut self.nodes.pool[*idx];
                if node.draggable {
                    node.origin += delta;
                }
            }
        }
    }

    fn should_link_snap_to_pin(
        &self,
        start_pin: &PinData,
        hovered_pin_idx: usize,
        duplicate_link: Option<usize>,
    ) -> bool {
        let end_pin = &self.pins.pool[hovered_pin_idx];
        if start_pin.parent_node_idx == end_pin.parent_node_idx {
            return false;
        }

        if start_pin.kind == end_pin.kind {
            return false;
        }

        if duplicate_link.map_or(false, |x| Some(x) != self.snap_link_idx) {
            return false;
        }
        true
    }

    fn box_selector_update_selection(&mut self) -> egui::Rect {
        let mut box_rect = self.click_interaction_state.box_selection;
        if box_rect.min.x > box_rect.max.x {
            std::mem::swap(&mut box_rect.min.x, &mut box_rect.max.x);
        }

        if box_rect.min.y > box_rect.max.y {
            std::mem::swap(&mut box_rect.min.y, &mut box_rect.max.y);
        }

        self.selected_node_indices.clear();
        for (idx, node) in self.nodes.pool.iter().enumerate() {
            if self.nodes.in_use[idx] && box_rect.intersects(node.rect) {
                self.selected_node_indices.push(idx);
            }
        }

        self.selected_link_indices.clear();
        for (idx, link) in self.links.pool.iter().enumerate() {
            if self.links.in_use[idx] {
                let pin_start = &self.pins.pool[link.start_pin_index];
                let pin_end = &self.pins.pool[link.end_pin_index];
                let node_start_rect = self.nodes.pool[pin_start.parent_node_idx].rect;
                let node_end_rect = self.nodes.pool[pin_end.parent_node_idx].rect;
                let start = self.style.get_screen_space_pin_coordinates(
                    &node_start_rect,
                    &pin_start.attribute_rect,
                    pin_start.kind,
                );
                let end = self.style.get_screen_space_pin_coordinates(
                    &node_end_rect,
                    &pin_end.attribute_rect,
                    pin_end.kind,
                );

                if self.rectangle_overlaps_link(&box_rect, &start, &end, pin_start.kind) {
                    self.selected_link_indices.push(idx);
                }
            }
        }
        box_rect
    }

    #[inline]
    fn rectangle_overlaps_link(
        &self,
        rect: &egui::Rect,
        start: &egui::Pos2,
        end: &egui::Pos2,
        start_type: AttributeType,
    ) -> bool {
        let mut lrect = egui::Rect::from_min_max(*start, *end);
        if lrect.min.x > lrect.max.x {
            std::mem::swap(&mut lrect.min.x, &mut lrect.max.x);
        }

        if lrect.min.y > lrect.max.y {
            std::mem::swap(&mut lrect.min.y, &mut lrect.max.y);
        }

        if rect.intersects(lrect) {
            if rect.contains(*start) || rect.contains(*end) {
                return true;
            }

            let link_data = LinkBezierData::get_link_renderable(
                *start,
                *end,
                start_type,
                self.style.link_line_segments_per_length,
            );
            return link_data.rectangle_overlaps_bezier(rect);
        }
        false
    }

    fn click_interaction_update(&mut self, ui: &mut egui::Ui) {
        match self.click_interaction_type {
            ClickInteractionType::BoxSelection => {
                self.click_interaction_state.box_selection.max = self.mouse_pos;
                let rect = self.box_selector_update_selection();

                let box_selector_color = self.style.colors[ColorStyle::BoxSelector as usize];
                let box_selector_outline =
                    self.style.colors[ColorStyle::BoxSelectorOutline as usize];
                ui.painter().rect(rect, 0.0, box_selector_color, (1.0, box_selector_outline));

                if self.left_mouse_released {
                    let mut idxs = Vec::with_capacity(self.selected_node_indices.len());
                    let depth_stack = &mut self.node_depth_order;
                    let selected_nodes = &self.selected_node_indices;
                    depth_stack.retain(|x| {
                        if selected_nodes.contains(x) {
                            idxs.push(*x);
                            false
                        } else {
                            true
                        }
                    });
                    self.node_depth_order.extend(idxs);
                    self.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Node => {
                self.translate_selected_nodes();
                if self.left_mouse_released {
                    self.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::Link => {
                if self.left_mouse_released {
                    self.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::LinkCreation => {
                let maybe_duplicate_link_idx = self.hovered_pin_index.and_then(|idx| {
                    self.find_duplicate_link(
                        self.click_interaction_state.link_creation.start_pin_idx,
                        idx,
                    )
                });

                let should_snap = self.hovered_pin_index.map_or(false, |idx| {
                    let start_pin =
                        &self.pins.pool[self.click_interaction_state.link_creation.start_pin_idx];
                    self.should_link_snap_to_pin(start_pin, idx, maybe_duplicate_link_idx)
                });

                let snapping_pin_changed = self
                    .click_interaction_state
                    .link_creation
                    .end_pin_index
                    .map_or(false, |idx| self.hovered_pin_index != Some(idx));

                if snapping_pin_changed && self.snap_link_idx.is_some() {
                    self.begin_link_detach(
                        self.snap_link_idx.unwrap(),
                        self.click_interaction_state.link_creation.end_pin_index.unwrap(),
                    );
                }

                let start_pin =
                    &self.pins.pool[self.click_interaction_state.link_creation.start_pin_idx];
                let start_pos = self.get_screen_space_pin_coordinates(start_pin);

                let end_pos = if should_snap {
                    self.get_screen_space_pin_coordinates(
                        &self.pins.pool[self.hovered_pin_index.unwrap()],
                    )
                } else {
                    self.mouse_pos
                };

                let link_data = LinkBezierData::get_link_renderable(
                    start_pos,
                    end_pos,
                    start_pin.kind,
                    self.style.link_line_segments_per_length,
                );
                ui.painter().add(link_data.draw((
                    self.style.link_thickness,
                    self.style.colors[ColorStyle::Link as usize],
                )));

                let link_creation_on_snap = self.hovered_pin_index.map_or(false, |idx| {
                    (self.pins.pool[idx].flags & AttributeFlags::EnableLinkCreationOnSnap as usize)
                        != 0
                });

                if !should_snap {
                    self.click_interaction_state.link_creation.end_pin_index.take();
                }

                let create_link =
                    should_snap && (self.left_mouse_released || link_creation_on_snap);

                if create_link && maybe_duplicate_link_idx.is_none() {
                    if !self.left_mouse_released
                        && self.click_interaction_state.link_creation.end_pin_index
                            == self.hovered_pin_index
                    {
                        return;
                    }
                    self.element_state_change |= ElementStateChange::LinkCreated as usize;
                    self.click_interaction_state.link_creation.end_pin_index =
                        self.hovered_pin_index;
                }

                if self.left_mouse_released {
                    self.click_interaction_type = ClickInteractionType::None;
                    if !create_link {
                        self.element_state_change |= ElementStateChange::LinkDropped as usize;
                    }
                }
            }
            ClickInteractionType::Panning => {
                if self.alt_mouse_dragging || self.alt_mouse_clicked {
                    self.panning += self.mouse_delta;
                } else {
                    self.click_interaction_type = ClickInteractionType::None;
                }
            }
            ClickInteractionType::None => (),
        }
    }

    fn begin_link_detach(&mut self, idx: usize, detach_idx: usize) {
        self.click_interaction_state.link_creation.end_pin_index.take();
        let link = &self.links.pool[idx];
        self.click_interaction_state.link_creation.start_pin_idx =
            if detach_idx == link.start_pin_index {
                link.end_pin_index
            } else {
                link.start_pin_index
            };
        self.deleted_link_idx.replace(idx);
    }

    fn begin_link_interaction(&mut self, idx: usize) {
        if self.click_interaction_type == ClickInteractionType::LinkCreation {
            if (self.hovered_pin_flags & AttributeFlags::EnableLinkDetachWithDragClick as usize)
                != 0
            {
                self.begin_link_detach(idx, self.hovered_pin_index.unwrap());
                self.click_interaction_state.link_creation.link_creation_type =
                    LinkCreationType::FromDetach;
            }
        } else if self.link_detatch_with_modifier_click {
            let link = &self.links.pool[idx];
            let start_pin = &self.pins.pool[link.start_pin_index];
            let end_pin = &self.pins.pool[link.end_pin_index];
            let dist_to_start = start_pin.pos.distance(self.mouse_pos);
            let dist_to_end = end_pin.pos.distance(self.mouse_pos);
            let closest_pin_idx = if dist_to_start < dist_to_end {
                link.start_pin_index
            } else {
                link.end_pin_index
            };
            self.click_interaction_type = ClickInteractionType::LinkCreation;
            self.begin_link_detach(idx, closest_pin_idx);
        } else {
            self.begin_link_selection(idx);
        }
    }

    fn begin_link_creation(&mut self, hovered_pin_idx: usize) {
        self.click_interaction_type = ClickInteractionType::LinkCreation;
        self.click_interaction_state.link_creation.start_pin_idx = hovered_pin_idx;
        self.click_interaction_state.link_creation.end_pin_index.take();
        self.click_interaction_state.link_creation.link_creation_type = LinkCreationType::Standard;
        self.element_state_change |= ElementStateChange::LinkStarted as usize;
    }

    fn begin_link_selection(&mut self, idx: usize) {
        self.click_interaction_type = ClickInteractionType::Link;
        self.selected_node_indices.clear();
        self.selected_link_indices.clear();
        self.selected_link_indices.push(idx);
    }

    fn find_duplicate_link(&self, start_pin_idx: usize, end_pin_idx: usize) -> Option<usize> {
        let mut test_link = LinkData::new(0);
        test_link.start_pin_index = start_pin_idx;
        test_link.end_pin_index = end_pin_idx;
        for (idx, (link, in_use)) in
            self.links.pool.iter().zip(self.links.in_use.iter()).enumerate()
        {
            if *in_use && *link == test_link {
                return Some(idx);
            }
        }
        None
    }

    fn begin_node_selection(&mut self, idx: usize) {
        if self.click_interaction_type != ClickInteractionType::None {
            return;
        }
        self.click_interaction_type = ClickInteractionType::Node;
        if !self.selected_node_indices.contains(&idx) {
            self.selected_node_indices.clear();
            self.selected_link_indices.clear();
            self.selected_node_indices.push(idx);

            self.node_depth_order.retain(|x| *x != idx);
            self.node_depth_order.push(idx);
        }
    }
}

#[derive(Debug)]
enum ElementStateChange {
    None = 0,
    LinkStarted = 1 << 0,
    LinkDropped = 1 << 1,
    LinkCreated = 1 << 2,
}

#[derive(PartialEq, Debug)]
enum ClickInteractionType {
    Node,
    Link,
    LinkCreation,
    Panning,
    BoxSelection,
    None,
}

#[derive(PartialEq, Debug)]
enum LinkCreationType {
    Standard,
    FromDetach,
}

#[derive(Derivative, Debug)]
#[derivative(Default)]
struct ClickInteractionStateLinkCreation {
    start_pin_idx: usize,
    end_pin_index: Option<usize>,
    #[derivative(Default(value = "LinkCreationType::Standard"))]
    link_creation_type: LinkCreationType,
}

#[derive(Derivative, Debug)]
#[derivative(Default)]
struct ClickInteractionState {
    link_creation: ClickInteractionStateLinkCreation,
    #[derivative(Default(value = "[[0.0; 2].into(); 2].into()"))]
    box_selection: egui::Rect,
}

#[derive(Debug)]
struct ColorStyleElement {
    color: egui::Color32,
    item: ColorStyle,
}

impl ColorStyleElement {
    fn new(color: egui::Color32, item: ColorStyle) -> Self {
        Self { color, item }
    }
}

#[derive(Debug)]
struct StyleElement {
    item: StyleVar,
    value: f32,
}

impl StyleElement {
    fn new(value: f32, item: StyleVar) -> Self {
        Self { value, item }
    }
}

/// This controls the modifers needed for certain mouse interactions
#[derive(Derivative, Debug)]
#[derivative(Default)]
pub struct IO {
    /// The Modfier that needs to pressed to pan the editor
    #[derivative(Default(value = "Modifiers::None"))]
    pub emulate_three_button_mouse: Modifiers,

    // The Modifier that needs to be pressed to detatch a link instead of creating a new one
    #[derivative(Default(value = "Modifiers::None"))]
    pub link_detatch_with_modifier_click: Modifiers,

    // The mouse button that pans the editor. Should probably not be set to Primary.
    #[derivative(Default(value = "Some(egui::PointerButton::Middle)"))]
    pub alt_mouse_button: Option<egui::PointerButton>,
}

/// Used to track which Egui Modifier needs to be pressed for certain IO actions
#[derive(Debug)]
pub enum Modifiers {
    Alt,
    Crtl,
    Shift,
    Command,
    None,
}

impl Modifiers {
    fn is_active(&self, mods: &egui::Modifiers) -> bool {
        match self {
            Modifiers::Alt => mods.alt,
            Modifiers::Crtl => mods.ctrl,
            Modifiers::Shift => mods.shift,
            Modifiers::Command => mods.command,
            Modifiers::None => false,
        }
    }
}

trait Id {
    fn id(&self) -> usize;
    fn new(id: usize) -> Self;
}

#[derive(Default, Debug)]
struct ObjectPool<T> {
    pool: Vec<T>,
    in_use: Vec<bool>,
    free: Vec<usize>,
    map: HashMap<usize, usize>,
}

impl<T> ObjectPool<T> {
    fn find(&self, id: usize) -> Option<usize> {
        self.map.get(&id).copied()
    }
    fn reset(&mut self) {
        self.in_use.iter_mut().for_each(|x| *x = false);
    }
}

impl<T: Id> ObjectPool<T> {
    fn update(&mut self) {
        self.free.clear();
        for (i, (in_use, obj)) in self.in_use.iter().zip(self.pool.iter()).enumerate() {
            if !*in_use {
                self.map.remove(&obj.id());
                self.free.push(i);
            }
        }
    }

    fn find_or_create_index(&mut self, id: usize) -> usize {
        let index = {
            if let Some(index) = self.find(id) {
                index
            } else {
                let index = if let Some(index) = self.free.pop() {
                    self.pool[index] = T::new(id);
                    index
                } else {
                    self.pool.push(T::new(id));
                    self.in_use.push(false);
                    self.pool.len() - 1
                };
                self.map.insert(id, index);
                index
            }
        };
        self.in_use[index] = true;
        index
    }
}

impl Context {
    fn node_pool_update(&mut self) {
        self.nodes.free.clear();
        for (i, (in_use, node)) in
            self.nodes.in_use.iter_mut().zip(self.nodes.pool.iter_mut()).enumerate()
        {
            if *in_use {
                node.pin_indices.clear();
            } else {
                if self.nodes.map.contains_key(&node.id) {
                    self.node_depth_order.retain(|x| *x != i);
                }
                self.nodes.map.remove(&node.id);
                self.nodes.free.push(i);
            }
        }
    }

    fn node_pool_find_or_create_index(&mut self, id: usize, origin: Option<egui::Pos2>) -> usize {
        let index = {
            if let Some(index) = self.nodes.find(id) {
                index
            } else {
                let mut new_node = NodeData::new(id);
                if let Some(origin) = origin {
                    new_node.origin = self.screen_space_to_grid_space(origin);
                }
                let index = if let Some(index) = self.nodes.free.pop() {
                    self.nodes.pool[index] = new_node;
                    index
                } else {
                    self.nodes.pool.push(new_node);
                    self.nodes.in_use.push(false);
                    self.nodes.pool.len() - 1
                };
                self.nodes.map.insert(id, index);
                self.node_depth_order.push(index);
                index
            }
        };
        self.nodes.in_use[index] = true;
        index
    }
}
