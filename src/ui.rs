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
    }
}
pub fn gui<'b,'a>(ref mut ui: conrod::UiCell, ids: &Ids, display: &'b glium::Display, app: &mut AppState<'b>){
    #![allow(unused_imports)]
    use conrod::{color, widget, Colorable, Labelable, Positionable, Scalar, Sizeable, Widget};
    let path = std::path::Path::new("data/");


    match app.gui_data.gui_display {
        GuiDisplay::FileOpen =>
        {
            widget::Canvas::new()
                .color(conrod::color::DARK_CHARCOAL)
                .x_y(660.0,200.0)
                .w_h(440.0,600.0)
                .set(ids.canvas, ui);

            for _press in widget::Button::new()
                .label("Open File")
                .align_middle_x_of(ids.canvas)
                .down(40.0)
                .w_h(400.0, 50.0)
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
                        let wfwidth: u32=1320;
                        let wfheight: u32=200;
                        for i in 0..openbci_file.channels{
                        app.waveform_drawers.push( WaveformDrawer::new( display,
                            WaveformDrawerSettings{
                                    x: -300,
                                    y: -50 - 250*i as i32 - wfheight as i32/2 + ui.win_h as i32/2,
                                    width: wfwidth,
                                    height: wfheight,
                                    milliseconds_per_pixel: 5.0,
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
                .down(40.0)
                .w_h(400.0, 50.0)
                .set(ids.btn_useportaudio, ui)
                {

                    pastuff::pa_read_from_mic(app);

                    println!("Initialising waveform drawer.");
                    app.waveform_drawers.clear();
                    let wfwidth: u32=1320;
                    let wfheight: u32=900;
                    app.waveform_drawers.push( WaveformDrawer::new( display,
                        WaveformDrawerSettings{
                                x: -200,
                                y: 0 as i32 - wfheight as i32/2 + ui.win_h as i32/2,
                                width: wfwidth,
                                height: wfheight,
                                milliseconds_per_pixel: 5.0,
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
                .x_y(660.0,000.0)
                .w_h(600.0,1000.0)
                .set(ids.settings_canvas, ui);
            let ref mut fd = app.filter_data;

            for (x, y) in widget::XYPad::new(fd.red.0, fd.min_red.0, fd.max_red.0,
                                                fd.red.1, fd.min_red.1, fd.max_red.1)
                .label("Red Channel")
                .w_h(200.0,200.0)
                .y(300.0)
                .align_middle_x_of(ids.settings_canvas)
                .parent(ids.settings_canvas)
                .set(ids.red_xy_pad, ui)
                {fd.red = (x, y);}

            for (x, y) in widget::XYPad::new(fd.green.0, fd.min_green.0, fd.max_green.0,
                                                fd.green.1, fd.min_green.1, fd.max_green.1)
                .label("Green Channel")
                .w_h(200.0,200.0)
                .y(50.0)
                .align_middle_x_of(ids.settings_canvas)
                .parent(ids.settings_canvas)
                .set(ids.green_xy_pad, ui)
                {fd.green = (x, y);}

            for (x, y) in widget::XYPad::new(fd.blue.0, fd.min_blue.0, fd.max_blue.0,
                                                fd.blue.1, fd.min_blue.1, fd.max_blue.1)
                .label("Blue Channel")
                .w_h(200.0,200.0)
                .y(-200.0)
                .align_middle_x_of(ids.settings_canvas)
                .parent(ids.settings_canvas)
                .set(ids.blue_xy_pad, ui)
                {fd.blue = (x, y);}

            for manamp in widget::Toggle::new(fd.amp_manual)
                .label("Manual Amp")
                .label_color(if fd.amp_manual { conrod::color::WHITE } else { conrod::color::LIGHT_CHARCOAL })
                .align_middle_x_of(ids.settings_canvas)
                .y(-350.0)
                .w_h(300.0, 40.0)
                .set(ids.toggle_manamp, ui)
            {fd.amp_manual=manamp;}

            if fd.amp_manual {
                for value in widget::Slider::new(fd.amp,fd.amp_min,fd.amp_max)
                    .y(-390.0)
                    .align_middle_x_of(ids.settings_canvas)
                    .w_h(300.0, 40.0)
                    .label("Amplification")
                    .set(ids.sldier_amplification, ui)
                    {fd.amp=value;}
            }

        }
        _=>()
    }


}
