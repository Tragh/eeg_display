use std;
use conrod;
use glium;
use appstate;
use appstate::{AppState, WaveData, GuiDisplay};
use openbci_file::{OpenBCIFile};
use waveformdrawer::{WaveformDrawer,WaveformDrawerSettings};
use pastuff;

// Generate a unique const `WidgetId` for each widget.
widget_ids!{
    pub struct Ids {
        canvas,
        button,
        btn_useportaudio,
        file_navigator,
        settings_canvas,
        red_xy_pad,
        green_xy_pad,
        blue_xy_pad,
        sldier_amplification,
        toggle_manamp,
        drop_down_dft_window_shape,
        freq_line,
        freq_display,
        freq_display_bg,
    }
}
pub fn gui<'b,'a>(ref mut ui: conrod::UiCell, ids: &Ids, display: &'b glium::Display, app: &mut AppState<'b>){
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    use conrod::{color, widget, Colorable, Labelable, Positionable, Scalar, Sizeable, Widget};
    let path = std::path::Path::new("data/");

    //So the window width and framebuffer width are different!!!!
    //They may or may not correspond to the actual window width
    //This is all a bit confusing so there are some helper functions for this
    let win_w = ui.win_w;
    let win_h = ui.win_h;
    let X = |x: f64| x*win_w/100.0;
    let Y = |x: f64| x*win_h/100.0;
    let iX = |x: f32| x/win_w as f32*100.0;
    let iY = |x: f32| x/win_h as f32*100.0;

    /* let (fb_w,fb_h) = display.get_framebuffer_dimensions();
    let fbX = |x: f64| x*fb_w as f64/100.0;
    let fbY = |x: f64| x*fb_h as f64/100.0; */


    match app.gui_data.gui_display {
        GuiDisplay::FileOpen =>
        {
            widget::Canvas::new()
                .color(conrod::color::DARK_CHARCOAL)
                .x_y(X(37.5),Y(20.0))
                .w_h(X(25.0),Y(60.0))
                .set(ids.canvas, ui);

            for _press in widget::Button::new()
                .label("Open File")
                .align_middle_x_of(ids.canvas)
                .down(Y(4.0))
                .w_h(X(20.0), Y(5.0))
                .set(ids.button, ui)
                {
                    println!("Pressed!");
                    println!("{:?}", app.gui_data.file_selection);

                    if app.gui_data.file_selection.is_some() {
                        // ## load OPENBCI file
                        println!("Reading OpenBCI data file.");
                        let openbci_file=OpenBCIFile::new(app.gui_data.file_selection.take().unwrap().to_str().unwrap());
                        let wave_data = WaveData{
                            buffer: openbci_file.samples.clone(),
                            channels: openbci_file.channels,
                            sample_rate: 200,
                            buffer_length: openbci_file.samples[0].len()
                        };
                        let app_data_arc=app.app_data.clone();
                        let mut app_data = app_data_arc.lock().unwrap();
                        app_data.wave_data = Some(wave_data);
                        app_data.data_source = appstate::DataSource::WavBuffer;

                        println!("Initialising waveform drawer.");
                        app.waveform_drawers.clear();
                        for i in 0..openbci_file.channels{
                        app.waveform_drawers.push( WaveformDrawer::new( display,
                            WaveformDrawerSettings{
                                    x: -12.5,
                                    y: 37.5 - 25.0 *i as f32,
                                    width: 75.0,
                                    height: 20.0,
                                    milliseconds_per_pixel: 8.0,
                                    time_pixels: 1000,
                                    dtft_samples: 800,
                                    dtft_display_samples: 200,
                                    channel: i}))
                        }

                        let ticks=app.ticker.ticks();
                        for wfd in &mut app.waveform_drawers{
                            wfd.start(ticks);
                        }
                        app.gui_data.gui_display=GuiDisplay::FilterOptions;
                    }

                }

            for _press in widget::Button::new()
                .label("Use Portaudio mic for input.")
                .align_middle_x_of(ids.canvas)
                .down(Y(4.0))
                .w_h(X(20.0), Y(5.0))
                .set(ids.btn_useportaudio, ui)
                {

                    pastuff::pa_read_from_mic(app);

                    println!("Initialising waveform drawer.");
                    app.waveform_drawers.clear();
                    app.waveform_drawers.push( WaveformDrawer::new( display,
                        WaveformDrawerSettings{
                                x: -12.5,
                                y: 0.0,
                                width: 75.0,
                                height: 100.0,
                                milliseconds_per_pixel: 5.0,
                                time_pixels: 1600,
                                dtft_samples: 1800,
                                dtft_display_samples: 300,
                                channel: 0}));

                    let ticks=app.ticker.ticks();
                    for wfd in &mut app.waveform_drawers{
                        wfd.start(ticks);
                    app.gui_data.gui_display=GuiDisplay::FilterOptions;
                    }
                }

            // Navigate the conrod directory only showing `.rs` and `.toml` files.
            for event in widget::FileNavigator::new(&path,conrod::widget::file_navigator::Types::All)
                .color(conrod::color::LIGHT_BLUE)
                .font_size(16)
                .wh_of(ids.canvas)
                .middle_of(ids.canvas)
                //.show_hidden_files(true)  // Use this to show hidden files
                .set(ids.file_navigator, ui)
                {
                    use conrod::widget::file_navigator::Event;
                    match event {
                        Event::ChangeSelection(mut paths)=>
                            app.gui_data.file_selection= if paths.len()>0 {Some(paths.pop().unwrap())} else {None},
                            _ => ()
                    }
                    //println!("{:?}", event);
                }
        }
        GuiDisplay::FilterOptions =>
        {
            widget::Canvas::new()
                .color(conrod::color::DARK_CHARCOAL)
                .x_y(X(37.5),Y(0.0))
                .w_h(X(25.0),Y(100.0))
                .set(ids.settings_canvas, ui);
            let ref mut fd = app.filter_data;

            for (x, y) in widget::XYPad::new(fd.green.0, fd.min_green.0, fd.max_green.0,
                                                fd.green.1, fd.min_green.1, fd.max_green.1)
                .label("Green")
                .w_h(X(7.0),X(7.0))
                .y(Y(35.0))
                .align_middle_x_of(ids.settings_canvas)
                .parent(ids.settings_canvas)
                .set(ids.green_xy_pad, ui)
                {fd.green = (x, y);}

            for (x, y) in widget::XYPad::new(fd.red.0, fd.min_red.0, fd.max_red.0,
                                                fd.red.1, fd.min_red.1, fd.max_red.1)
                .label("Red")
                .w_h(X(7.0),X(7.0))
                .left_from(ids.green_xy_pad,Y(2.0))
                .parent(ids.settings_canvas)
                .set(ids.red_xy_pad, ui)
                {fd.red = (x, y);}


            for (x, y) in widget::XYPad::new(fd.blue.0, fd.min_blue.0, fd.max_blue.0,
                                                fd.blue.1, fd.min_blue.1, fd.max_blue.1)
                .label("Blue")
                .w_h(X(7.0),X(7.0))
                .right_from(ids.green_xy_pad,Y(2.0))
                .parent(ids.settings_canvas)
                .set(ids.blue_xy_pad, ui)
                {fd.blue = (x, y);}

            for manamp in widget::Toggle::new(fd.amp_manual)
                .label("Manual Amp")
                .label_color(if fd.amp_manual { conrod::color::WHITE } else { conrod::color::LIGHT_CHARCOAL })
                .align_middle_x_of(ids.settings_canvas)
                .w_h(X(20.0),X(3.0))
                .down(Y(5.0))
                .set(ids.toggle_manamp, ui)
            {fd.amp_manual=manamp;}

            if fd.amp_manual {
                for value in widget::Slider::new(fd.amp,fd.amp_min,fd.amp_max)
                    .align_middle_x_of(ids.settings_canvas)
                    .w_h(X(20.0),X(3.0))
                    .down(Y(0.0))
                    .label("Amplification")
                    .set(ids.sldier_amplification, ui)
                    {fd.amp=value;}
            }

            let list_items = [
                "Rectangular Window".to_string(),
                "Hann Window".to_string(),
                "Hamming Window".to_string(),
                "Nuttall Window".to_string(),
                "Sine Window".to_string(),
                "Kaiser Window".to_string(),
            ];

            for drop in widget::DropDownList::new(&list_items,Some(fd.window_shape as usize))
                .align_middle_x_of(ids.settings_canvas)
                .w_h(X(20.0),X(3.0))
                .down(Y(5.0))
                .set(ids.drop_down_dft_window_shape, ui)
                {fd.window_shape = drop as i32;}

            for wfd in &app.waveform_drawers {
                let x = iX(app.gui_data.cursor_xy.0);
                let y = iY(app.gui_data.cursor_xy.1);
                if  (x - wfd.settings.x - 50.0).abs() < wfd.settings.width/2.0 &&
                    (y - wfd.settings.y - 50.0).abs() < wfd.settings.height/2.0 {
                        //hf is how far above the bottom of the spectrum is the mouse as a proportion of the height of the spectrum.
                        let hf:f32 = 0.5 - (y - wfd.settings.y - 50.0)/ wfd.settings.height;
                        let freq:f32 = hf * wfd.settings.dtft_display_samples as f32 * pastuff::PA_SAMPLE_RATE as f32 / wfd.settings.dtft_samples as f32;
                        let freqs = format!("{:.2}", freq) + " Hz";
                        widget::Line::centred([0.0,0.0], [X(wfd.settings.width as f64),0.0])
                        .x_y(X(wfd.settings.x as f64),Y(50.0 - y as f64))
                        .set(ids.freq_line, ui);
                        let t = widget::Text::new(&freqs)
                        .y(Y(50.0 - y as f64))
                        .align_left_of(ids.freq_line)
                        .align_middle_y_of(ids.freq_line);
                        widget::Rectangle::fill([t.get_w(ui).unwrap()*1.05, t.get_h(ui).unwrap()*1.1]).align_left_of(ids.freq_line).y_relative_to(ids.freq_line,-2.0).set(ids.freq_display_bg, ui);
                        t.set(ids.freq_display, ui);
                    }

            }

        }
        _=>()
    }


}
