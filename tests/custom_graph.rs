use protobuf;
use std::collections::HashMap;

use wonnx::*;
// Indicates a f32 overflow in an intermediate Collatz value
// Args Management
async fn run() {
    let steps = execute_gpu().await.unwrap();
    let n = 32;

    assert_eq!(steps[0..5], [0.0, 1.0, 2.0, 3.0, 4.0]);
    for i in 0..n {
        println!("steps: {:?}", &steps[n / 2 * i..n / 2 * (i + 1)]);
    }
    #[cfg(target_arch = "wasm32")]
    log::info!("steps[0..5]: {:#?}", &steps[0..5]);
}

// Hardware management
async fn execute_gpu() -> Option<Vec<f32>> {
    // USER INPUT

    let n: usize = 32;
    let mut input_data = HashMap::new();

    let data: Vec<f32> = (0..n * n / 2).map(|x| x as f32).collect();
    let dims = vec![n as i64, (n / 2) as i64];

    input_data.insert("X".to_string(), (data.as_slice(), dims.as_slice()));

    // ONNX INPUTS
    let mut shape_dims = vec![];
    for dim in dims.iter() {
        let mut shape_tensor_proto_dim = onnx::TensorShapeProto_Dimension::new();
        shape_tensor_proto_dim.set_dim_value(*dim);
        shape_dims.push(shape_tensor_proto_dim)
    }

    let mut shape_tensor_proto = onnx::TensorShapeProto::new();
    shape_tensor_proto.set_dim(protobuf::RepeatedField::from(shape_dims));

    let mut type_proto_tensor = crate::onnx::TypeProto_Tensor::new();
    type_proto_tensor.set_elem_type(1);
    type_proto_tensor.set_shape(shape_tensor_proto);

    let mut type_proto = crate::onnx::TypeProto::new();
    type_proto.set_tensor_type(type_proto_tensor);

    let mut input = crate::onnx::ValueInfoProto::new();
    input.set_name("X".to_string());
    input.set_field_type(type_proto.clone());

    let mut attribute_y = crate::onnx::ValueInfoProto::new();
    attribute_y.set_name("Y".to_string());
    attribute_y.set_field_type(type_proto.clone());

    let mut output = crate::onnx::ValueInfoProto::new();
    output.set_name("Z".to_string());
    output.set_field_type(type_proto.clone());

    let mut node1 = crate::onnx::NodeProto::new();
    node1.set_op_type("Transpose".to_string());
    node1.set_name("node".to_string());
    node1.set_input(protobuf::RepeatedField::from(vec!["X".to_string()]));
    node1.set_output(protobuf::RepeatedField::from(vec!["Y".to_string()]));

    let mut perm = onnx::AttributeProto::new();
    perm.set_name("perm".to_string());
    perm.set_ints(vec![1, 0]);

    node1.set_attribute(protobuf::RepeatedField::from(vec![perm.clone()]));

    let mut node2 = crate::onnx::NodeProto::new();
    node2.set_op_type("Transpose".to_string());
    node2.set_name("node".to_string());
    node2.set_input(protobuf::RepeatedField::from(vec!["Y".to_string()]));
    node2.set_output(protobuf::RepeatedField::from(vec!["Z".to_string()]));
    node2.set_attribute(protobuf::RepeatedField::from(vec![perm]));

    let mut graph = wonnx::onnx::GraphProto::new();
    graph.set_node(protobuf::RepeatedField::from(vec![node1, node2]));
    graph.set_input(protobuf::RepeatedField::from(vec![input]));
    graph.set_output(protobuf::RepeatedField::from(vec![output]));
    graph.set_value_info(protobuf::RepeatedField::from(vec![attribute_y]));

    let mut model = crate::onnx::ModelProto::new();
    model.set_graph(graph);

    // LOGIC

    let mut session = wonnx::Session::from_model(model)
        .await
        .expect("Session did not create");

    session.run(input_data).await
}

#[test]
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run());
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run());
    }
}
