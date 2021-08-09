# egui_nodes

 A egui port of https://github.com/Nelarius/imnodes 
 
### Example
``` rust
pub fn example_graph(ctx: &mut Context, links: &mut Vec<(usize, usize)>, ui: &mut Ui) {
    // add nodes with attributes
    let nodes = vec![
        NodeConstructor::new(0, Default::default())
            .with_title(|ui| ui.label("Example Node A"))
            .with_input_attribute(0, Default::default(), |ui| ui.label("Input"))
            .with_static_attribute(1, |ui| ui.label("Can't Connect to Me"))
            .with_output_attribute(2, Default::default(), |ui| ui.label("Output")),
        NodeConstructor::new(1, Default::default())
            .with_title(|ui| ui.label("Example Node B"))
            .with_static_attribute(3, |ui| ui.label("Can't Connect to Me"))
            .with_output_attribute(4, Default::default(), |ui| ui.label("Output"))
            .with_input_attribute(5, Default::default(), |ui| ui.label("Input"))
    ];

    // add them to the ui
    ctx.show(
        nodes,
        links.iter().enumerate().map(|(i, (start, end))| (i, *start, *end, LinkArgs::default())),
        ui
    );
    
    // remove destroyed links
    if let Some(idx) = ctx.link_destroyed() {
        links.remove(idx);
    }

    // add created links
    if let Some((start, end, _)) = ctx.link_created() {
        links.push((start, end))
    }
}
```
 
 <img src="media/example.gif">