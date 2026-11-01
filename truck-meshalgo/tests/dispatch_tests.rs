//! Tests for the intelligent kernel dispatcher, startup micro-benchmark, and persistent caching.

use truck_base::cgmath64::*;
use truck_meshalgo::dispatch::cache::*;
use truck_meshalgo::dispatch::metadata::*;
use truck_meshalgo::dispatch::microbench::*;
use truck_meshalgo::dispatch::profile::*;
use truck_meshalgo::dispatch::*;
use truck_meshalgo::prelude::*;

fn create_cube() -> PolygonMesh {
    let positions = vec![
        Point3::new(-1.0, -1.0, -1.0),
        Point3::new(1.0, -1.0, -1.0),
        Point3::new(1.0, 1.0, -1.0),
        Point3::new(-1.0, 1.0, -1.0),
        Point3::new(-1.0, -1.0, 1.0),
        Point3::new(1.0, -1.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(-1.0, 1.0, 1.0),
    ];
    let faces = Faces::from_iter(vec![
        [0, 2, 1], [0, 3, 2], // -z
        [4, 5, 6], [4, 6, 7], // +z
        [0, 1, 5], [0, 5, 4], // -y
        [2, 3, 7], [2, 7, 6], // +y
        [0, 4, 7], [0, 7, 3], // -x
        [1, 2, 6], [1, 6, 5], // +x
    ]);
    PolygonMesh::new(StandardAttributes { positions, ..Default::default() }, faces)
}

#[test]
fn test_metadata_detection_and_serialization() {
    let metadata = SystemMetadata::current();
    println!("Detected CPU: {} ({} threads, {})", metadata.cpu.model_name, metadata.cpu.logical_cores, metadata.cpu.architecture);
    assert!(!metadata.cpu.model_name.is_empty());
    assert!(metadata.cpu.logical_cores > 0);

    #[cfg(feature = "cuda")]
    if let Some(ref cuda) = metadata.gpu.cuda {
        println!("Primary CUDA GPU: {} (SM {}, {} SMs, {} MB)", cuda.name, cuda.compute_capability, cuda.multi_processor_count, cuda.total_memory_bytes / 1024 / 1024);
        assert!(!cuda.name.is_empty());
    }

    #[cfg(feature = "cuda")]
    {
        let all_cuda = truck_meshalgo::gpu::cuda::CudaMeshAnalyzer::get_all_devices();
        println!("All detected CUDA devices ({}):", all_cuda.len());
        for dev in &all_cuda {
            println!("  CUDA #{}: {} (SM {}, {} SMs, {} MB)", dev.device_id, dev.name, dev.compute_capability, dev.multi_processor_count, dev.total_memory_bytes / 1024 / 1024);
        }
    }

    #[cfg(feature = "gpu")]
    if let Some(ref vulkan) = metadata.gpu.vulkan {
        println!("Detected Vulkan GPU: {} ({})", vulkan.name, vulkan.backend);
        assert!(!vulkan.name.is_empty());
    }

    {
        use truck_platform::wgpu::*;
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            flags: InstanceFlags::from_build_config(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let adapters = pollster::block_on(instance.enumerate_adapters(Backends::VULKAN));
        for (i, adapter) in adapters.into_iter().enumerate() {
            let info = adapter.get_info();
            println!("  Found wgpu adapter #{}: {} ({:?}, driver: {}, info: {})", i, info.name, info.backend, info.driver, info.driver_info);
        }
    }

    // Verify serialization roundtrip
    let json = serde_json::to_string_pretty(&metadata).expect("Failed to serialize metadata");
    let deserialized: SystemMetadata = serde_json::from_str(&json).expect("Failed to deserialize metadata");
    assert_eq!(metadata, deserialized);
}

#[test]
fn test_cache_invalidation_lifecycle() {
    let temp_dir = std::env::temp_dir();
    let temp_cache_file = temp_dir.join(format!(".truck_test_cache_{}.json", std::process::id()));

    // Clean up if leftover
    let _ = std::fs::remove_file(&temp_cache_file);

    let profile = DispatchProfile::default();
    save_cached_profile_to_path(&profile, &temp_cache_file).expect("Failed to save cache");
    assert!(temp_cache_file.exists());

    // 1. Should successfully load when matching
    let loaded = load_cached_profile_from_path(&temp_cache_file);
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap(), profile);

    // 2. Invalidate via Cache Version Change
    {
        let content = std::fs::read_to_string(&temp_cache_file).unwrap();
        let mut raw: serde_json::Value = serde_json::from_str(&content).unwrap();
        raw["cache_version"] = serde_json::Value::from(999);
        std::fs::write(&temp_cache_file, serde_json::to_string(&raw).unwrap()).unwrap();

        let invalidated = load_cached_profile_from_path(&temp_cache_file);
        assert!(invalidated.is_none(), "Cache should be invalidated when cache_version changes");
    }

    // 3. Invalidate via CPU Metadata Change
    {
        let mut cache = KernelCache::new(profile.clone());
        cache.hardware.cpu.model_name = "Completely Different Quantum CPU 9000".to_string();
        std::fs::write(&temp_cache_file, serde_json::to_string(&cache).unwrap()).unwrap();

        let invalidated = load_cached_profile_from_path(&temp_cache_file);
        assert!(invalidated.is_none(), "Cache should be invalidated when CPU changes");
    }

    // 4. Invalidate via GPU Metadata Change
    #[cfg(feature = "cuda")]
    {
        let mut cache = KernelCache::new(profile.clone());
        if let Some(ref mut cuda) = cache.hardware.gpu.cuda {
            cuda.name = "NVIDIA GeForce RTX 9090 Super".to_string();
        } else {
            cache.hardware.gpu.cuda = Some(CudaDeviceInfo {
                device_id: 0,
                name: "Fake GPU".to_string(),
                compute_capability: "9.9".to_string(),
                multi_processor_count: 128,
                total_memory_bytes: 32 * 1024 * 1024 * 1024,
            });
        }
        std::fs::write(&temp_cache_file, serde_json::to_string(&cache).unwrap()).unwrap();

        let invalidated = load_cached_profile_from_path(&temp_cache_file);
        assert!(invalidated.is_none(), "Cache should be invalidated when GPU changes");
    }

    let _ = std::fs::remove_file(&temp_cache_file);
}

#[test]
fn test_microbench_calibration() {
    println!("Running scaling microbenchmark calibration...");
    let profile = run_scaling_microbenchmark();

    println!("Calibrated Thresholds:");
    println!("  PointMeshDistance   : 1T <= {}, Rayon <= {}, Preferred GPU: {:?}",
        profile.point_mesh_distance.cpu_single_max,
        profile.point_mesh_distance.cpu_rayon_max,
        profile.point_mesh_distance.preferred_gpu
    );
    println!("  InOutContainment    : 1T <= {}, Rayon <= {}, Preferred GPU: {:?}",
        profile.in_out_containment.cpu_single_max,
        profile.in_out_containment.cpu_rayon_max,
        profile.in_out_containment.preferred_gpu
    );
    println!("  MeshNormals         : 1T <= {}, Rayon <= {}, Preferred GPU: {:?}",
        profile.mesh_normals.cpu_single_max,
        profile.mesh_normals.cpu_rayon_max,
        profile.mesh_normals.preferred_gpu
    );
    println!("  MeshInterference    : 1T <= {}, Rayon <= {}, Preferred GPU: {:?}",
        profile.mesh_interference.cpu_single_max,
        profile.mesh_interference.cpu_rayon_max,
        profile.mesh_interference.preferred_gpu
    );
    println!("  TrimmingLoopInclude : 1T <= {}, Rayon <= {}, Preferred GPU: {:?}",
        profile.trimming_loop_include.cpu_single_max,
        profile.trimming_loop_include.cpu_rayon_max,
        profile.trimming_loop_include.preferred_gpu
    );
    println!("  BsplineEvaluation   : 1T <= {}, Rayon <= {}, Preferred GPU: {:?}",
        profile.bspline_evaluation.cpu_single_max,
        profile.bspline_evaluation.cpu_rayon_max,
        profile.bspline_evaluation.preferred_gpu
    );

    assert!(profile.point_mesh_distance.cpu_single_max <= profile.point_mesh_distance.cpu_rayon_max);
    assert!(!profile.point_mesh_distance.sample_timings.is_empty());
}

#[test]
fn test_dispatcher_routing_and_parity() {
    let dispatcher = Dispatcher::new_with_profile(Dispatcher::global().profile().clone());
    let cube = create_cube();

    // 1. PointMeshDistance
    let small_pts = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0)];
    let target_small = dispatcher.select_kernel(Operation::PointMeshDistance, small_pts.len());
    // Problem sizes below GPU crossover route to CPU execution target (single-threaded or Rayon depending on core count)
    assert!(matches!(target_small, KernelTarget::CpuSingle | KernelTarget::CpuRayon));

    let d_small = dispatcher.distance2_point_cloud(&cube, &small_pts);
    assert_eq!(d_small.len(), 2);
    assert!((d_small[0] - 1.0).abs() < 1e-5); // Center to cube face is 1.0 (squared: 1.0)
    assert!((d_small[1] - 1.0).abs() < 1e-5); // (2,0,0) to cube face at x=1 is 1.0 (squared: 1.0)

    let large_pts: Vec<Point3> = (0..5000)
        .map(|i| {
            let t = i as f64 * 0.01;
            Point3::new(t.sin() * 0.5, t.cos() * 0.5, 0.0)
        })
        .collect();
    let target_large = dispatcher.select_kernel(Operation::PointMeshDistance, large_pts.len());
    if SystemMetadata::current().has_gpu() {
        #[cfg(feature = "cuda")]
        assert!(matches!(
            target_large,
            KernelTarget::GpuPtxSm86 | KernelTarget::GpuVulkan | KernelTarget::GpuCuda
        ));
    } else {
        let _ = target_large;
    }

    let d_large = dispatcher.distance2_point_cloud(&cube, &large_pts);
    assert_eq!(d_large.len(), 5000);

    // Verify exact parity with CPU
    dispatcher.set_override(Some(KernelTarget::CpuSingle));
    let d_cpu = dispatcher.distance2_point_cloud(&cube, &large_pts);
    dispatcher.reset_override();

    for i in 0..100 {
        assert!(
            (d_large[i] - d_cpu[i]).abs() < 1e-4,
            "Distance mismatch at index {}: GPU {}, CPU {}",
            i, d_large[i], d_cpu[i]
        );
    }

    // 2. In-Out Containment
    let inside_res = dispatcher.inside(&cube, &small_pts);
    assert_eq!(inside_res, vec![true, false]);

    // 3. Manual Override
    dispatcher.set_override(Some(KernelTarget::CpuSingle));
    assert_eq!(dispatcher.select_kernel(Operation::PointMeshDistance, 100000), KernelTarget::CpuSingle);
    dispatcher.reset_override();
    if SystemMetadata::current().has_gpu() {
        #[cfg(feature = "cuda")]
        assert!(matches!(
            dispatcher.select_kernel(Operation::PointMeshDistance, 100000),
            KernelTarget::GpuPtxSm86 | KernelTarget::GpuVulkan | KernelTarget::GpuCuda
        ));
    }
}

#[test]
fn test_cache_hit_vs_miss_timing() {
    let temp_cache_file = std::env::temp_dir().join(format!(".truck_timing_cache_{}.json", std::process::id()));
    let _ = std::fs::remove_file(&temp_cache_file);

    // Initial cache miss -> Run benchmark and save
    let t0 = std::time::Instant::now();
    let profile = run_scaling_microbenchmark();
    save_cached_profile_to_path(&profile, &temp_cache_file).unwrap();
    let miss_elapsed = t0.elapsed();

    // Cache hit -> Read file
    let t1 = std::time::Instant::now();
    let loaded = load_cached_profile_from_path(&temp_cache_file);
    let hit_elapsed = t1.elapsed();

    assert!(loaded.is_some());
    println!("Cache Miss + Calibration time : {:?}", miss_elapsed);
    println!("Cache Hit load time           : {:?}", hit_elapsed);
    assert!(hit_elapsed < std::time::Duration::from_millis(50));
    assert!(miss_elapsed > hit_elapsed * 10);

    let _ = std::fs::remove_file(&temp_cache_file);
}

#[test]
fn test_dispatcher_nurbs_surface_evaluation() {
    let dispatcher = Dispatcher::global();
    let uknot_vec = vec![0.0, 0.0, 1.0, 1.0];
    let vknot_vec = vec![0.0, 0.0, 1.0, 1.0];
    let ctrl_pts = vec![
        vec![Vector4::new(0.0, 0.0, 0.0, 1.0), Vector4::new(0.0, 1.0, 0.0, 1.0)],
        vec![Vector4::new(1.0, 0.0, 0.0, 1.0), Vector4::new(1.0, 1.0, 1.0, 2.0)],
    ];
    let mut uv_points = Vec::new();
    for &u in &[0.25, 0.5, 0.75] {
        for &v in &[0.25, 0.5, 0.75] {
            uv_points.push(Point2::new(u, v));
        }
    }
    let (pts, normals) = dispatcher.evaluate_nurbs_surface(&uknot_vec, &vknot_vec, 1, 1, &ctrl_pts, &uv_points);
    assert_eq!(pts.len(), 9);
    assert_eq!(normals.len(), 9);

    dispatcher.set_override(Some(KernelTarget::CpuSingle));
    let (cpu_pts, cpu_normals) = dispatcher.evaluate_nurbs_surface(&uknot_vec, &vknot_vec, 1, 1, &ctrl_pts, &uv_points);
    dispatcher.reset_override();

    for i in 0..9 {
        assert!((pts[i] - cpu_pts[i]).magnitude2() < 1e-10);
        assert!((normals[i] - cpu_normals[i]).magnitude2() < 1e-10);
    }
}

#[test]
fn test_multi_gpu_concurrent_execution() {
    let metadata = SystemMetadata::current();
    println!("System GPU device count: {}", metadata.gpu.device_count());

    #[cfg(feature = "cuda")]
    {
        let count = truck_meshalgo::gpu::cuda::CudaMeshAnalyzer::get_device_count();
        println!("Detected {} CUDA device(s):", count);
        for i in 0..count {
            if let Some(info) = truck_meshalgo::gpu::cuda::CudaMeshAnalyzer::get_device_info_at(i as i32) {
                println!("  Device {}: {} (SM {}, {} SMs, {} MB)", i, info.name, info.compute_capability, info.multi_processor_count, info.total_memory_bytes / 1024 / 1024);
            }
        }

        if count >= 2 {
            let mesh = create_cube();
            let mut points = Vec::new();
            for i in 0..2048 {
                let x = (i as f64 * 0.01).sin() * 2.0;
                let y = (i as f64 * 0.013).cos() * 2.0;
                let z = (i as f64 * 0.017).sin() * 2.0;
                points.push(Point3::new(x, y, z));
            }

            // Run concurrently on dev 0 and dev 1
            let (dist0, dist1) = rayon::join(
                || {
                    let a0 = truck_meshalgo::gpu::cuda::CudaMeshAnalyzer::with_device(0);
                    a0.distance2_point_cloud(&mesh, &points)
                },
                || {
                    let a1 = truck_meshalgo::gpu::cuda::CudaMeshAnalyzer::with_device(1);
                    a1.distance2_point_cloud(&mesh, &points)
                },
            );

            assert_eq!(dist0.len(), points.len());
            assert_eq!(dist1.len(), points.len());
            for i in 0..points.len() {
                assert!(
                    (dist0[i] - dist1[i]).abs() < 1e-4,
                    "Mismatch at point {}: dev0={}, dev1={}", i, dist0[i], dist1[i]
                );
            }

            // Now test through Dispatcher multi-GPU automatic splitting
            let dispatcher = Dispatcher::global();
            dispatcher.set_override(Some(KernelTarget::GpuPtxSm86));
            let multi_dist = dispatcher.distance2_point_cloud(&mesh, &points);
            let multi_inside = dispatcher.inside(&mesh, &points);
            dispatcher.reset_override();

            // Compare against CPU Rayon
            dispatcher.set_override(Some(KernelTarget::CpuRayon));
            let cpu_dist = dispatcher.distance2_point_cloud(&mesh, &points);
            let cpu_inside = dispatcher.inside(&mesh, &points);
            dispatcher.reset_override();

            assert_eq!(multi_dist.len(), points.len());
            assert_eq!(multi_inside.len(), points.len());
            for i in 0..points.len() {
                assert!(
                    (multi_dist[i] - cpu_dist[i]).abs() < 1e-3,
                    "Multi-GPU dist mismatch at {}: multi={}, cpu={}", i, multi_dist[i], cpu_dist[i]
                );
                assert_eq!(
                    multi_inside[i], cpu_inside[i],
                    "Multi-GPU inside mismatch at point {}", i
                );
            }
            println!("Multi-GPU concurrent execution passed across {} CUDA devices!", count);
        } else {
            println!("Skipping multi-GPU test: only {} CUDA device detected", count);
        }
    }

    #[cfg(feature = "gpu")]
    {
        let vulkan_devices = truck_meshalgo::gpu::get_all_vulkan_devices();
        println!("Detected {} Vulkan device(s):", vulkan_devices.len());
        for v in &vulkan_devices {
            println!("  Vulkan #{}: {} ({})", v.adapter_index, v.name, v.driver);
        }
    }
}

#[test]
fn test_explicit_warmup() {
    let t0 = std::time::Instant::now();
    truck_meshalgo::warmup();
    let elapsed = t0.elapsed();
    println!("Explicit warmup completed in {:?}", elapsed);

    // After warmup, running a distance query executes with zero cold-start hitch
    let mesh = create_cube();
    let pts = vec![Point3::new(0.5, 0.5, 0.5)];
    let t1 = std::time::Instant::now();
    let d = Dispatcher::global().distance2_point_cloud(&mesh, &pts);
    let first_query_elapsed = t1.elapsed();
    println!("Post-warmup first query latency: {:?}", first_query_elapsed);
    assert_eq!(d.len(), 1);
    assert!(first_query_elapsed < std::time::Duration::from_millis(50));
}
