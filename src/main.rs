use eframe::{egui, App, Frame, CreationContext};
use egui::{Color32, TextureHandle, Vec2, Sense};
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;

// Estrutura para armazenar informações de cada vídeo
struct VideoEntry {
    title: String,
    url: String,
    id: String,
    texture: Option<TextureHandle>,
}

// Estrutura principal do aplicativo
struct M3UViewer {
    m3u_path: Option<PathBuf>,
    search_query: String,
    videos: Vec<VideoEntry>,
    filtered_videos: Vec<usize>,
    pending_downloads: Vec<String>,
}

impl M3UViewer {
    fn new(_cc: &CreationContext<'_>) -> Self {
        // Criar diretório de cache se não existir
        fs::create_dir_all("cache_m3u").unwrap_or_else(|_| {
            println!("Não foi possível criar o diretório de cache");
        });
        
        Self {
            m3u_path: None,
            search_query: String::new(),
            videos: Vec::new(),
            filtered_videos: Vec::new(),
            pending_downloads: Vec::new(),
        }
    }

// Função para abrir arquivo .m3u
fn open_m3u_file(&mut self, path: PathBuf) {
    self.m3u_path = Some(path.clone());
    self.videos.clear();
    
    // Ler arquivo .m3u
    if let Ok(file) = File::open(&path) {
        let reader = io::BufReader::new(file);
        let youtube_id_regex = Regex::new(r"(?:youtu\.be/|youtube\.com/(?:embed/|v/|watch\?v=|watch\?.+&v=))([^?&/]+)").unwrap();
        
        let mut lines = reader.lines();
        let mut current_title = String::new();
        
        while let Some(Ok(line)) = lines.next() {
            if line.starts_with("#EXTINF") {
                // Extrair título da linha EXTINF - pegar tudo após a primeira vírgula
                if let Some(pos) = line.find(',') {
                    current_title = line[pos + 1..].trim().to_string();
                }
            } else if !line.starts_with("#") && !line.trim().is_empty() {
                // Esta é uma linha de URL
                // Extrair ID do vídeo da URL
                let id = if let Some(captures) = youtube_id_regex.captures(&line) {
                    captures.get(1).unwrap().as_str().to_string()
                } else {
                    // Gerar um ID baseado no hash da URL se não for do YouTube
                    format!("{:x}", md5::compute(line.as_bytes()))
                };
                
                // Usar o título extraído ou a URL como fallback
                let title = if current_title.is_empty() {
                    line.split('/').last().unwrap_or(&line).to_string()
                } else {
                    current_title.clone()
                };
                
                self.videos.push(VideoEntry {
                    title,
                    url: line,
                    id: id.clone(),
                    texture: None,
                });
                
                // Verificar se a thumbnail existe, caso contrário adicionar à lista de downloads
                let cache_path = format!("cache_m3u/{}.jpg", id);
                if !Path::new(&cache_path).exists() {
                    self.pending_downloads.push(id);
                }
                
                // Resetar o título atual para a próxima entrada
                current_title = String::new();
            }
        }
    }
    
    // Atualizar lista filtrada
    self.update_filtered_videos();
}
    
    // Função para atualizar a lista filtrada com base na pesquisa
    fn update_filtered_videos(&mut self) {
        self.filtered_videos.clear();
        
        let query = self.search_query.to_lowercase();
        for (i, video) in self.videos.iter().enumerate() {
            if query.is_empty() || video.title.to_lowercase().contains(&query) {
                self.filtered_videos.push(i);
            }
        }
    }
    
    // Função para carregar texturas
    fn load_textures(&mut self, ctx: &egui::Context) {
        for video in &mut self.videos {
            if video.texture.is_none() {
                let cache_path = format!("cache_m3u/{}.jpg", video.id);
                if Path::new(&cache_path).exists() {
                    if let Ok(image) = image::open(&cache_path) {
                        let image = image.to_rgba8();
                        let size = [image.width() as _, image.height() as _];
                        let image_data = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            &image.into_raw(),
                        );
                        let texture = ctx.load_texture(
                            &video.id,
                            image_data,
                            egui::TextureOptions::default(),
                        );
                        video.texture = Some(texture);
                    }
                }
            }
        }
    }
    
    // Função para baixar thumbnails pendentes
    async fn download_thumbnails(&mut self) {
        let client = reqwest::Client::new();
        let mut completed = Vec::new();
        
        for (i, id) in self.pending_downloads.iter().enumerate() {
            if i >= 5 {  // Limitar a 5 downloads simultâneos
                break;
            }
            
            let cache_path = format!("cache_m3u/{}.jpg", id);
            let url = format!("https://img.youtube.com/vi/{}/mqdefault.jpg", id);
            
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(bytes) = response.bytes().await {
                            if let Ok(mut file) = File::create(&cache_path) {
                                if file.write_all(&bytes).is_ok() {
                                    completed.push(id.clone());
                                }
                            }
                        }
                    }
                },
                Err(_) => {
                    // Em caso de erro, criar uma imagem em branco como fallback
                    let img = image::RgbaImage::new(320, 180);
                    if img.save(&cache_path).is_ok() {
                        completed.push(id.clone());
                    }
                }
            }
        }
        
        // Remover downloads concluídos da lista pendente
        for id in completed {
            if let Some(pos) = self.pending_downloads.iter().position(|x| x == &id) {
                self.pending_downloads.remove(pos);
            }
        }
    }
    
    // Função para reproduzir um vídeo
    fn play_video(&self, index: usize) {
        if let Some(&video_index) = self.filtered_videos.get(index) {
            if let Some(video) = self.videos.get(video_index) {
                // Criar arquivo temporário .m3u
                if let Ok(mut file) = File::create("temp.m3u") {
                    if file.write_all(video.url.as_bytes()).is_ok() {
                        // Abrir com o aplicativo padrão
                        #[cfg(target_os = "windows")]
                        {
                            Command::new("cmd")
                                .args(&["/C", "start", "temp.m3u"])
                                .spawn()
                                .ok();
                        }
                        
                        #[cfg(target_os = "linux")]
                        {
                            Command::new("xdg-open")
                                .arg("temp.m3u")
                                .spawn()
                                .ok();
                        }
                        
                        #[cfg(target_os = "macos")]
                        {
                            Command::new("open")
                                .arg("temp.m3u")
                                .spawn()
                                .ok();
                        }
                    }
                }
            }
        }
    }
}

impl App for M3UViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Carregar texturas para vídeos que ainda não têm
        self.load_textures(ctx);
        
        // Iniciar downloads pendentes
        if !self.pending_downloads.is_empty() {
            let pending = self.pending_downloads.clone();
            let pending_clone = pending.clone();
            
            let future = async move {
                let client = reqwest::Client::new();
                for (i, id) in pending_clone.iter().enumerate() {
                    if i >= 5 {  // Limitar a 5 downloads simultâneos
                        break;
                    }
                    
                    let cache_path = format!("cache_m3u/{}.jpg", id);
                    let url = format!("https://img.youtube.com/vi/{}/mqdefault.jpg", id);
                    
                    if let Ok(response) = client.get(&url).send().await {
                        if response.status().is_success() {
                            if let Ok(bytes) = response.bytes().await {
                                if let Ok(mut file) = File::create(&cache_path) {
                                    let _ = file.write_all(&bytes);
                                }
                            }
                        }
                    }
                }
            };
            
            tokio::spawn(future);
            
            // Remover os primeiros 5 downloads da lista pendente
            let count = pending.len().min(5);
            for _ in 0..count {
                if !self.pending_downloads.is_empty() {
                    self.pending_downloads.remove(0);
                }
            }
        }
        
        // Barra superior com menu e pesquisa
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Arquivo", |ui| {
                    if ui.button("Abrir M3U...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("M3U Playlist", &["m3u", "m3u8"])
                            .pick_file() 
                        {
                            self.open_m3u_file(path);
                            ui.close_menu();
                        }
                    }
                    
                    if ui.button("Sair").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Pesquisar:");
                    if ui.text_edit_singleline(&mut self.search_query).changed() {
                        self.update_filtered_videos();
                    }
                });
            });
        });
        
        // Área principal com a lista de vídeos
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.m3u_path.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label("Selecione um arquivo M3U/M3U8 no menu Arquivo");
                });
                return;
            }
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);
                
                // Exibir vídeos em grade
                let available_width = ui.available_width();
                let thumbnail_width = 320.0;
                let thumbnail_height = 180.0;
                let padding = 10.0;
                let items_per_row = (available_width / (thumbnail_width + padding)).floor() as usize;
                let items_per_row = items_per_row.max(1);
                
                let mut i = 0;
                while i < self.filtered_videos.len() {
                    ui.horizontal(|ui| {
                        for j in 0..items_per_row {
                            let idx = i + j;
                            if idx >= self.filtered_videos.len() {
                                break;
                            }
                            
                            let video_idx = self.filtered_videos[idx];
                            let video = &self.videos[video_idx];
                            
                            ui.vertical(|ui| {
                                // Exibir thumbnail
                                let (rect, _) = ui.allocate_exact_size(Vec2::new(thumbnail_width, thumbnail_height), Sense::click());
                                
                                if let Some(texture) = &video.texture {
                                    ui.painter().image(
                                        texture.id(),
                                        rect,
                                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                        Color32::WHITE,
                                    );
                                } else {
                                    // Placeholder enquanto a imagem não carrega
                                    ui.painter().rect_filled(
                                        rect,
                                        0.0,
                                        Color32::from_rgb(50, 50, 50),
                                    );
                                    
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "Carregando...",
                                        egui::FontId::default(),
                                        Color32::WHITE,
                                    );
                                }
                                
                                // Detectar clique na thumbnail
                                if ui.interact(rect, ui.id().with(idx), Sense::click()).clicked() {
                                    self.play_video(idx);
                                }
                                
                                // Título do vídeo com quebra de linha
                                ui.set_max_width(thumbnail_width);
                                ui.label(&video.title);
                            });
                        }
                    });
                    
                    i += items_per_row;
                }
            });
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Visualizador M3U",
        options,
        Box::new(|cc| Ok(Box::new(M3UViewer::new(cc))))
    )
}