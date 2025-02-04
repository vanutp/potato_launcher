pub fn error(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::RED
    } else {
        egui::Color32::from_rgb(128, 0, 0)
    }
}

pub fn partial_error(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::BLUE
    } else {
        egui::Color32::from_rgb(0, 0, 128)
    }
}

pub fn offline(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::YELLOW
    } else {
        egui::Color32::from_rgb(128, 128, 0)
    }
}

pub fn in_progress(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::LIGHT_BLUE
    } else {
        egui::Color32::from_rgb(0, 128, 128)
    }
}

pub fn timeout(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(255, 0, 255)
    } else {
        egui::Color32::from_rgb(128, 0, 128)
    }
}

pub fn action(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::LIGHT_YELLOW
    } else {
        egui::Color32::from_rgb(128, 128, 0)
    }
}

pub fn ok(_dark_mode: bool) -> egui::Color32 {
    egui::Color32::PLACEHOLDER
}
