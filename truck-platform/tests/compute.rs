mod common;
use truck_platform::*;
use wgpu::*;

const COMPUTE_SHADER: &str = r#"
@group(0) @binding(0)
var<storage, read> input_data: array<f32>;

@group(0) @binding(1)
var<storage, read_write> output_data: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index < arrayLength(&input_data)) {
        let x = input_data[index];
        // Test cubic polynomial: 2.0*x^3 - 3.0*x^2 + 4.0*x - 5.0
        output_data[index] = 2.0 * x * x * x - 3.0 * x * x + 4.0 * x - 5.0;
    }
}
"#;

#[test]
fn test_compute_pipeline_vulkan() {
    common::os_alt_exec_test(|backends, _| {
        let handler = common::init_device(backends);
        let n: usize = 1024;
        let input: Vec<f32> = (0..n).map(|i| i as f32 * 0.1).collect();
        let expected: Vec<f32> = input
            .iter()
            .map(|&x| 2.0 * x * x * x - 3.0 * x * x + 4.0 * x - 5.0)
            .collect();

        let in_buf = BufferHandler::storage_from_slice(&input, handler.device());
        let out_buf = BufferHandler::storage_empty(
            handler.device(),
            (n * std::mem::size_of::<f32>()) as u64,
        );

        let bind_group_layout_entries = [
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];

        let engine = ComputeEngine::new(
            &handler,
            COMPUTE_SHADER,
            "main",
            &bind_group_layout_entries,
        );

        let bind_group = engine.create_bind_group([
            in_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((n as u32 + 63) / 64, 1, 1);
        engine.dispatch(&bind_group, workgroups);

        let result: Vec<f32> = out_buf.read_to_vec_block(&handler);
        assert_eq!(result.len(), n);
        for i in 0..n {
            let diff = (result[i] - expected[i]).abs();
            let rel_diff = diff / expected[i].abs().max(1.0);
            assert!(
                rel_diff < 1e-5,
                "Mismatch at index {}: got {}, expected {}, rel_diff {}",
                i,
                result[i],
                expected[i],
                rel_diff
            );
        }
    });
}
