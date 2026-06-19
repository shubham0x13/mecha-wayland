use criterion::{Criterion, criterion_group, criterion_main};
use glow::HasContext;
use lock_screen::lock_ui::LockUi;
use lock_screen::render;
use renderer::{DmaBuf, Renderer, commands::*};
use utils::Color;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const COLOR_LOCK_BG: Color = Color::rgb(0.05, 0.05, 0.07);

fn bench_lock_screen_session(c: &mut Criterion) {
    let mut renderer =
        Renderer::new().expect("Renderer::new failed — needs /dev/dri/renderD* and EGL");

    let surface = renderer
        .create_surface::<DmaBuf>(WIDTH, HEIGHT)
        .expect("create_surface failed");

    // Initialize command queues
    renderer.init_command_queue::<ClearColor>();
    renderer.init_command_queue::<DrawRect>();
    renderer.init_command_queue::<DrawQuad>();
    renderer.init_command_queue::<DrawMonochromeSprite>();
    renderer.init_command_queue::<DrawText>();

    // Upload the UI atlas
    renderer.upload_atlas(&lock_screen::atlas::UI).expect("atlas upload");

    // Set up LockUi (using dummy surface IDs)
    let mut lock_ui = LockUi::new(0, 0);
    lock_ui.surface.size = (WIDTH as i32, HEIGHT as i32);

    // Recompute layout once before benchmarking rendering
    lock_ui.recompute_layout();

    let mut group = c.benchmark_group("lock_screen_session");

    // Benchmark 1: Layout only
    group.bench_function("layout_recompute_1920x1080", |b| {
        b.iter(|| {
            lock_ui.recompute_layout();
        });
    });

    // Benchmark 2: Full frame render time (recompute layout + command collection + GPU draw & finish)
    group.bench_function("frame_render_1920x1080", |b| {
        b.iter(|| {
            renderer.active_surface(&surface);
            lock_ui.recompute_layout();
            let cmds = lock_ui.render_commands();
            unsafe {
                renderer.gl.clear_depth_f32(0.0);
                renderer.gl.clear(glow::DEPTH_BUFFER_BIT);
                render::render_frame(&mut renderer, cmds, COLOR_LOCK_BG);
                renderer.gl.finish(); // measures true GPU time
            }
        });
    });

    group.finish();
    renderer.destroy_surface(surface);
}

criterion_group! {
    name = lock_screen_benches;
    config = Criterion::default();
    targets = bench_lock_screen_session
}

criterion_main!(lock_screen_benches);
