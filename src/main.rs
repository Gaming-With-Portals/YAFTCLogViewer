
use std::io::{self, BufRead, BufReader};
use eframe::egui;
use rfd::FileDialog;
use std::fs;
use std::collections::HashMap;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Yet Another FTC Log Viewer", 
        eframe::NativeOptions::default(), 
    Box::new(|cc| {
        Ok(Box::new(App::default()))
    }),
    )
}

#[derive(Clone, Debug)]
pub enum EventType {
    Verbose,
    Info,
    Warning,
    Error,
    Debug
}

#[derive(Clone)]
pub struct LogEvent {
    pub line_idx: usize,
    pub date: String,
    pub time: String,
    pub code_a : String, // PID
    pub code_b : String, // TID?
    pub tp: EventType,
    pub caller: String,
    pub info : String,

}


#[derive(Default)]
pub struct LogFile {
    pub lines: Vec<String>,
    events: Vec<LogEvent>,
    pub event_map: HashMap<usize, LogEvent>,
    pub op_starts: Vec<usize>,
}

impl LogFile {
    fn parse_event(line: &str, line_idx: usize) -> Option<LogEvent> {
        

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < 6 { return None; }

        let date  = tokens[0];
        let time  = tokens[1];
        let pid   = tokens[2];
        let tid   = tokens[3];
        let level = tokens[4];
        let caller = tokens[5].trim_end_matches(':');
        let message = tokens[6..].join(" ");

        let tp = match level.as_bytes().first() {
            Some(b'I') => EventType::Info,
            Some(b'W') => EventType::Warning,
            Some(b'E') => EventType::Error,
            Some(b'D') => EventType::Debug,
            _          => EventType::Verbose,
        };

        Some(LogEvent {
            line_idx,
            date:   date.to_string(),
            time:   time.to_string(),
            code_a: pid.to_string(),
            code_b: tid.to_string(),
            tp,
            caller: caller.to_string(),
            info:   message,
        })
    }

    pub fn open(path: &String) -> io::Result<Self> {

        let contents = fs::read_to_string(path)?;
        let lines: Vec<String> = contents.lines().map(str::to_string).collect();

        let events: Vec<LogEvent> = lines.iter().enumerate()
            .filter_map(|(i, line)| Self::parse_event(line, i))
            .collect();
        
        let mut op_starts : Vec<usize> = events.iter()
            .filter(|e| e.info.contains("Robot Controller starting OpMode") || e.info.contains("START - OPMODE"))
            .filter(|e| !e.info.contains("$Stop$Robot$")) // what is this
            .map(|e| e.line_idx)
            .collect();

        op_starts.insert(0, 0);

        let event_map = events.iter()
            .map(|e| (e.line_idx, e.clone()))
            .collect();
        
        Ok(Self { lines, events, event_map, op_starts })
    }

}


#[derive(Default)]
struct App {
    log_path: String,
    log_file: LogFile,
    show_verbose : bool,
    show_info : bool,
    show_debug :bool,
    show_warning : bool,
    show_error : bool,
    filtered_lines: Vec<usize>,
    goto : Option<usize>,
    op_start_seek : usize,
    highlighted_line: Option<usize>,
    goto_freeze : u8, // hack and a half
    selected_opmode: usize,
    text_search : String,
    last_text_search : String,
}

impl App {
    fn open_log_file(&mut self, path : String){
        self.show_debug = true;
        self.show_error = true;
        self.show_verbose = true;
        self.show_info = true;
        self.show_warning = true;
        self.op_start_seek = 0;
        self.text_search = "".to_string();
        self.last_text_search = "".to_string();

        println!("File Read Start!");
        match LogFile::open(&path) {
            Ok(lf) => {
                self.log_file = lf;
                self.log_path = path;
                println!("File Read Finished!"); 
            }
            Err(e) => eprintln!("Failed to open: {e}"),
        }
        self.rebuild_filter();
    }

    fn rebuild_filter(&mut self) {
        let min_line = self.log_file.op_starts
            .get(self.selected_opmode)
            .copied()
            .unwrap_or(0);

        let filter = self.text_search.to_lowercase();

        self.filtered_lines = self.log_file.lines.iter().enumerate()
            .filter(|(i, line)| {
                if *i < min_line { return false; }

                if !filter.is_empty() && !line.to_lowercase().contains(&filter) {
                    return false;
                }

                match self.log_file.event_map.get(i) {
                    Some(e) => match e.tp {
                        EventType::Verbose => self.show_verbose,
                        EventType::Info    => self.show_info,
                        EventType::Warning => self.show_warning,
                        EventType::Error   => self.show_error,
                        EventType::Debug   => self.show_debug,
                    },
                    None => self.show_verbose,
                }
            })
            .map(|(i, _)| i)
            .collect();
    }

    fn seek_to_line(&mut self, line : usize){
        
        
        if let Some(filtered_row) = self.filtered_lines.iter().position(|&i| i == line) {
            self.goto_freeze = 3;
            self.goto = Some(filtered_row);
            self.highlighted_line = Some(line);
        }
    }

}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
        egui::TopBottomPanel::top("tools").show(ctx, |ui|{
            ui.horizontal(|ui|{
                
                if (ui.button("Select File").clicked()){
                    let log_file_selector = FileDialog::new()
                        .add_filter("text", &["txt", "1", "2", "3", "4", "5"])
                        .set_directory("/")
                        .pick_file();
                    
                    match log_file_selector{
                        Some(path) => self.open_log_file(path.display().to_string()),
                        None => println!("Selection cancelled"),
                    }

                }

                ui.label(format!("Open file: {}", self.log_path));
            });
            ui.horizontal(|ui|{
                let changed =
                    ui.checkbox(&mut self.show_verbose, "Show Verbose").changed() |
                    ui.checkbox(&mut self.show_info,    "Show Debug").changed() |
                    ui.checkbox(&mut self.show_debug, "Show Info").changed() |
                    ui.checkbox(&mut self.show_warning, "Show Warning").changed() |
                    ui.checkbox(&mut self.show_error,   "Show Error").changed();
                if changed { self.rebuild_filter(); }
            });

            ui.horizontal(|ui|{
                ui.label("Start Log From: ");

                let op_starts: Vec<usize> = self.log_file.op_starts.clone();
                let mut labels: Vec<String> = op_starts.iter()
                    .map(|&line_idx| {
                        let info = self.log_file.event_map
                        .get(&line_idx)
                        .map(|e| e.info.as_str())
                        .unwrap_or("");

                    if info.contains("START - OPMODE"){
                        info.split("START - OPMODE")
                        .nth(1)
                        .unwrap_or("UNKNOWN")
                        .trim()
                        .trim_matches('*')
                        .trim()
                        .to_string()   
                    } else {
                        info.split(":")
                        .last()
                        .unwrap_or("UNKNOWN")
                        .trim().to_string()
                    }

                    })
                    .map(|name| name + " - OpMode Start")
                    .collect();

                if (labels.len() > 1){
                    labels.remove(0);
                }
                
                labels.insert(0, "File Start".to_string());
                

                egui::ComboBox::from_label("")
                    .selected_text(labels.get(self.selected_opmode).map(|s| s.as_str()).unwrap_or("File Start"))
                    .show_ui(ui, |ui| {
                        for (i, (line_idx, label)) in op_starts.iter().zip(labels.iter()).enumerate() {
                            if ui.selectable_value(&mut self.selected_opmode, i, label).clicked() {
                                self.goto = Some(0);
                                self.goto_freeze = 3;
                                self.rebuild_filter();
                            }
                        }
                    });

            });
            ui.horizontal(|ui|{
                ui.label("Search: ");
                egui::TextEdit::singleline(&mut self.text_search).show(ui);
                if (self.text_search != self.last_text_search){
                    self.rebuild_filter();
                    self.last_text_search = self.text_search.clone();
                }
                ui.label("(Note: This searches raw lines, not what's displayed)")
            });

        });

        egui::CentralPanel::default().show(ctx, |ui|{
            if self.log_path.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a file");
                });
                return;
            }
                
            let line_height = 18.0;
            //ui.label(egui::RichText::new("MM-DD HH-MM-SS.MS   PID   TID  TYPE").monospace().size(13.0));

            let mut scroll_area = egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .id_salt("main_log");

            if let Some(target) = self.goto.take() {
                scroll_area = scroll_area.vertical_scroll_offset(target as f32 * line_height);
            }

            scroll_area.show_rows(ui, line_height, self.filtered_lines.len(), |ui, row_range| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    for row in row_range {
                        let i = self.filtered_lines[row];
                        let line = &self.log_file.lines[i];


                            
                        let color = match self.log_file.event_map.get(&i) {
                            Some(e) => match e.tp {
                                EventType::Error   => egui::Color32::from_rgb(255, 100, 100),
                                EventType::Warning => egui::Color32::from_rgb(255, 190, 80),
                                EventType::Info    => egui::Color32::from_rgb(150, 200, 255),
                                EventType::Debug    => egui::Color32::from_rgb(20, 200, 20),
                                EventType::Verbose => egui::Color32::GRAY,
                            },
                            None => egui::Color32::GRAY,
                        };
                        let type_text = match self.log_file.event_map.get(&i) {
                            Some(e) => match e.tp {
                                EventType::Error   => "[ERROR]",
                                EventType::Warning => "[WARN ]",
                                EventType::Info    => "[INFO ]",
                                EventType::Debug   => "[DEBUG]",
                                EventType::Verbose => "[VERB ]",
                            },
                            None => "[UNK  ]",
                        };

                        let is_highlighted = self.highlighted_line == Some(i);

                        let resp = ui.horizontal(|ui| {
                            if is_highlighted {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(
                                    rect,
                                    0.0,
                                    egui::Color32::from_rgba_premultiplied(255, 255, 100, 40),
                                );
                            }

                            ui.label(format!("{:05}", i));
                            

                            
                            if let Some(e) = self.log_file.event_map.get(&i) {
                                ui.label(egui::RichText::new(&e.date).monospace().size(13.0).color(egui::Color32::GRAY));
                                ui.label(egui::RichText::new(&e.time).monospace().size(13.0).color(egui::Color32::from_rgb(150, 100, 150)));

                                ui.label(egui::RichText::new(type_text).monospace().size(13.0).color(color));
                                ui.label(egui::RichText::new(&e.caller).monospace().size(13.0).color(egui::Color32::GRAY));
                                ui.label(&e.info);
                            }
                        });


                    }
                    
            });  
        });

    }


}