extern crate regex;
extern crate num;
#[macro_use] extern crate conrod;
mod support;
extern crate glium;

extern crate portaudio;
extern crate find_folder;
extern crate rustfft;

//use glium::DisplayBuild;
use glium::Surface;

//use glium::{DisplayBuild, Surface};

mod city2d;

pub mod waveformdrawer;
use waveformdrawer::{WaveformDrawer};

pub mod appstate;
use appstate::{AppState, Ticker, AppData, GuiData, GuiDisplay, FilterData};

pub mod openbci_file;

pub mod pastuff;

pub mod ui;

pub mod dftwindower;


pub fn main() {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1000;

    println!("Building the window.");
    /*let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("STFT Viewer")
        .build()
        .expect("Unable to create OpenGL Window.");*/
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Spectrum Analyser")
        .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_multisampling(0);
        //.with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    println!("Constructing UI.");
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).theme(support::theme()).build();



    //OpenGL stuff###################################




    println!("Initialising internal data.");

    let mut app = AppState{
        filter_data: FilterData::default(),
        gui_data: GuiData{
            gui_display: GuiDisplay::FileOpen,
            file_selection: None},
        waveform_drawers: Vec::<WaveformDrawer>::new(),
        app_data: std::sync::Arc::new(std::sync::Mutex::new(AppData{
            data_source: appstate::DataSource::NoSource,
            wave_data: None,
            streaming_data: None})),
        ticker: Ticker::default()
    };


    // end waveform create

    // A unique identifier for each widget.
    let ids = ui::Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let noto_sans = assets.join("fonts/NotoSans");


    // Specify the default font to use when none is specified by the widget.
    ui.theme.font_id = Some(ui.fonts.insert_from_file(noto_sans.join("NotoSans-Regular.ttf")).unwrap());

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    println!("Starting main event loop.");
    let mut frame_rater=support::FrameRater::new(0);
    let mut event_loop = support::EventLoop::new();
    //################################################################################################################################################################################################
    'main: loop {

        // Handle all events.
        for event in event_loop.next(&mut events_loop) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                ui.handle_event(event);
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }


        let mut ticks;
        loop{
            ticks=app.ticker.ticks();
            for wfd in &mut app.waveform_drawers {wfd.update_stft(ticks, &app.app_data, &app.filter_data);}
            //frame_rater.fps(ticks);
            std::thread::sleep(std::time::Duration::from_millis(1));
            if frame_rater.elapsed_ms(ticks, 16) {break;}
        }




        ui::gui(ui.set_widgets(), &ids, &display, &mut app);

        // Render the `Ui` and then display it on the screen.
        let mut target = display.draw();
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
        }
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw(&display, &mut target, &image_map).unwrap();


            //###### MY DRAWING GOES HERE ######

            //gliumtexdraw.draw(&mut target,&textures[i as usize],0.0,wy(400.0-250.0*i as f64),wx(1600.0),wy(192.0));
            for wfd in &mut app.waveform_drawers {
                //gliumtexdraw.draw(&mut target,&waveform_textures[i as usize],0.0,wy(400.0-250.0*i as f64),wx(1600.0),wy(192.0));

                wfd.generate_and_draw_texture(&mut target);
            }

            //###### MY DRAWING ENDS HERE ######
        target.finish().unwrap();

        event_loop.needs_update(); //force update
    }
    //#####################################################################################################################################################################################
}
