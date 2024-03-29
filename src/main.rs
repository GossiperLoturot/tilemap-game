use winit::event::Event;
use winit::event::WindowEvent;

mod assets;
mod game_loop;
mod renderer;

fn main() {
    let assets = assets::Assets::new("assets/assets.json");

    let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let window = std::sync::Arc::new(window);

    let mut game_loop = game_loop::GameLoop::new();
    let mut renderer = pollster::block_on(renderer::Renderer::new_async(&assets, window.clone()));
    let mut input = winit_input_helper::WinitInputHelper::new();
    let mut instant = std::time::Instant::now();

    event_loop
        .run(move |event, control_flow| {
            let _ = input.update(&event);

            match event {
                Event::AboutToWait => {
                    window.request_redraw();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::RedrawRequested => {
                        let tick =
                            std::mem::replace(&mut instant, std::time::Instant::now()).elapsed();

                        let window_size = (window.inner_size().width, window.inner_size().height);
                        let cx = game_loop::Context {
                            assets: &assets,
                            input: &input,
                            tick: &tick,
                            window_size: &window_size,
                        };
                        game_loop.update(&cx);
                        let extract = game_loop.extract(&cx);

                        renderer.render(&assets, &extract);
                    }
                    WindowEvent::Resized(new_inner_size) => {
                        renderer.resize(new_inner_size);
                    }
                    WindowEvent::CloseRequested => {
                        control_flow.exit();
                    }
                    _ => (),
                },
                _ => (),
            }
        })
        .unwrap();
}
