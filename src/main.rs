#![windows_subsystem = "windows"] // iniciar o programa sem abrir uma janela de terminal.
use eframe::{egui, App, CreationContext, Frame};
use egui::{Color32, Sense, TextureHandle, Vec2};
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt; // para usar api do windows. no momento serve para abrir arquivos sem abrir o terminal.
mod gerar_m3u8;
use gerar_m3u8::gerar_m3u8;

// Estruturas para armazenar informações na RAM
struct VideoEntry {
    title: String,                  // Título do vídeo extraído do arquivo M3U
    url: String,                    // URL completa do vídeo
    id: String,                     // ID único do vídeo (do YouTube ou hash MD5 para outras fontes)
    texture: Option<TextureHandle>, // Thumbnail do vídeo carregada do cache (None se ainda não carregada)
}
struct M3UViewer {
    m3u_path: Option<PathBuf>, // Armazena o caminho do arquivo M3U atual (opcional)
    search_query: String,      // Armazena o texto de pesquisa digitado pelo usuário
    videos: Vec<VideoEntry>,   // Lista de todos os vídeos carregados do arquivo M3U
    filtered_videos: Vec<usize>, // Índices dos vídeos que correspondem à pesquisa atual
    pending_downloads: Vec<String>, // IDs dos vídeos que precisam ter thumbnails baixadas
    selected_videos: Vec<usize>,  // Índices dos vídeos selecionados pelo usuário
    zoom_factor: f32,           // Fator de zoom para os thumbnails
    grid_width_factor: f32,     // Fator de largura da grade
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
            selected_videos: Vec::new(),
            zoom_factor: 1.0,           // Valor inicial do zoom
            grid_width_factor: 0.9,     // Valor inicial da largura da grade (90%)
        }
    }

    // Função para abrir arquivo .m3u
    fn open_m3u_file(&mut self, path: PathBuf) {
        self.m3u_path = Some(path.clone());
        self.videos.clear();
        self.selected_videos.clear();

        // Ler arquivo .m3u
        if let Ok(file) = File::open(&path) {
            let reader = io::BufReader::new(file);
            let youtube_id_regex = Regex::new(
                r"(?:youtu\.be/|youtube\.com/(?:embed/|v/|watch\?v=|watch\?.+&v=))([^?&/]+)",
            )
            .unwrap();

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
                        let image_data =
                            egui::ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
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

    // Função para alternar a seleção de um vídeo
    fn toggle_video_selection(&mut self, filtered_index: usize) {
        if let Some(&video_index) = self.filtered_videos.get(filtered_index) {
            if let Some(position) = self.selected_videos.iter().position(|&x| x == video_index) {
                // Se já estiver selecionado, remova da seleção
                self.selected_videos.remove(position);
            } else {
                // Caso contrário, adicione à seleção
                self.selected_videos.push(video_index);
            }
        }
    }

    // Função para reproduzir vídeos selecionados
    fn play_selected_videos(&self) {
        if self.selected_videos.is_empty() {
            // Se nenhum vídeo estiver selecionado, não faça nada
            return;
        }

        // Criar arquivo temporário .m3u
        if let Ok(mut file) = File::create("temp.m3u") {
            // Escrever o cabeçalho M3U
            let _ = file.write_all(b"#EXTM3U\n");
            
            // Adicionar cada vídeo selecionado ao arquivo
            for &video_index in &self.selected_videos {
                if let Some(video) = self.videos.get(video_index) {
                    let entry = format!("#EXTINF:-1, {}\n{}\n", video.title, video.url);
                    let _ = file.write_all(entry.as_bytes());
                }
            }
            
            // Abrir com o aplicativo padrão
            #[cfg(target_os = "windows")]
            {
                const SW_HIDE: u32 = 0;
                Command::new("rundll32")
                    .args(&["url.dll,FileProtocolHandler", "temp.m3u"])
                    .creation_flags(SW_HIDE)
                    .spawn()
                    .ok();
            }

            #[cfg(target_os = "linux")]
            {
                Command::new("xdg-open").arg("temp.m3u").spawn().ok();
            }

            #[cfg(target_os = "macos")]
            {
                Command::new("open").arg("temp.m3u").spawn().ok();
            }
        }
    }

    // Função para reproduzir um único vídeo (mantida para compatibilidade)
    fn play_video(&self, index: usize) {
        // Se não houver vídeos selecionados, selecione apenas este
        if self.selected_videos.is_empty() {
            if let Some(&video_index) = self.filtered_videos.get(index) {
                // Criar arquivo temporário .m3u
                if let Ok(mut file) = File::create("temp.m3u") {
                    // Escrever o cabeçalho e informações do vídeo no formato M3U correto
                    if let Some(video) = self.videos.get(video_index) {
                        let m3u_content = format!("#EXTM3U\n#EXTINF:-1, {}\n{}", video.title, video.url);
                        
                        if file.write_all(m3u_content.as_bytes()).is_ok() {
                            // Abrir com o aplicativo padrão
                            #[cfg(target_os = "windows")]
                            {
                                const SW_HIDE: u32 = 0;
                                Command::new("rundll32")
                                    .args(&["url.dll,FileProtocolHandler", "temp.m3u"])
                                    .creation_flags(SW_HIDE)
                                    .spawn()
                                    .ok();
                            }

                            #[cfg(target_os = "linux")]
                            {
                                Command::new("xdg-open").arg("temp.m3u").spawn().ok();
                            }

                            #[cfg(target_os = "macos")]
                            {
                                Command::new("open").arg("temp.m3u").spawn().ok();
                            }
                        }
                    }
                }
            }
        } else {
            // Se já houver vídeos selecionados, reproduza todos
            self.play_selected_videos();
        }
    }
}

impl App for M3UViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Carregar texturas para vídeos que ainda não têm
        self.load_textures(ctx);

        // Processar eventos de scroll para zoom e ajuste de grade
        ctx.input(|input| {
            // Verificar se há eventos de scroll
            let scroll_delta = input.raw_scroll_delta.y;
            if scroll_delta != 0.0 {
                let scroll_direction = scroll_delta.signum();
                
                // Ctrl+Scroll para zoom nos videos
                if input.modifiers.ctrl {
                    // Ajustar o fator de zoom (aumentar/diminuir em 5% por scroll)
                    let zoom_change = 0.05 * scroll_direction;
                    self.zoom_factor = (self.zoom_factor + zoom_change).clamp(0.5, 2.0);
                }
                
                // Alt+Scroll para ajustar largura da grade
                if input.modifiers.alt {
                    // Ajustar o fator de largura da grade (aumentar/diminuir em 5% por scroll)
                    let width_change = 0.05 * scroll_direction;
                    self.grid_width_factor = (self.grid_width_factor + width_change).clamp(0.25, 1.0);
                }
            }
        });

        // Iniciar downloads pendentes
        if !self.pending_downloads.is_empty() {
            let pending = self.pending_downloads.clone();
            let pending_clone = pending.clone();

            let future = async move {
                let client = reqwest::Client::new();
                for (i, id) in pending_clone.iter().enumerate() {
                    if i >= 50 { break; } // Limitar a 50 downloads simultâneos

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
                ui.menu_button("Opções", |ui| {
                    ui.set_min_width(150.0); // Definir largura mínima do menu, 1 = 1,33 pixels numa tela 1920x1080
                    
                    if ui.button("Abrir M3U...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("M3U Playlist", &["m3u", "m3u8"])
                            .pick_file()
                        {
                            self.open_m3u_file(path);
                            ui.close_menu();
                        }
                    }

                    if ui.button("gerar m3u8...").clicked() {
                        gerar_m3u8();
                        ui.close_menu();
                    }
                    
                    if !self.selected_videos.is_empty() {
                        if ui.button("Reproduzir Selecionados").clicked() {
                            self.play_selected_videos();
                            ui.close_menu();
                        }
                        
                        if ui.button("Limpar Seleção").clicked() {
                            self.selected_videos.clear();
                            ui.close_menu();
                        }
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.text_edit_singleline(&mut self.search_query).changed() {
                        self.update_filtered_videos();
                    }
                    ui.label("Pesquisar:");
                    
                    // Exibir informações sobre os controles com tooltips
                    ui.label(format!("Zoom: {:.0}%", self.zoom_factor * 100.0))
                        .on_hover_text("Ctrl+Scroll para ajustar o zoom dos videos");
                    ui.label(format!("Largura: {:.0}%", self.grid_width_factor * 100.0))
                        .on_hover_text("Alt+Scroll para ajustar a largura da grade");
                });
            });
        });

        // Área principal com a lista de vídeos
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.m3u_path.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label("Selecione um arquivo M3U/M3U8 no menu Opções");
                });
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Configurar espaçamento
                    ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);

                    // Exibir vídeos em grade
                    let available_width = ui.available_width();
                    
                    // Calcular largura efetiva usando o fator de largura da grade
                    let effective_width = available_width * self.grid_width_factor;
                    
                    // Calcular margem lateral
                    let side_margin = (available_width - effective_width) / 2.0;
                    
                    // Aplicar o fator de zoom ao tamanho base do thumbnail
                    let base_thumbnail_width = 320.0 * self.zoom_factor;
                    let base_thumbnail_height = 180.0 * self.zoom_factor;
                    
                    // Calcular quantos itens cabem por linha usando a largura efetiva
                    let items_per_row = (effective_width / base_thumbnail_width).floor() as usize;
                    let items_per_row = items_per_row.max(1);
                    
                    // Calcular a largura ideal para cada thumbnail para ocupar toda a largura efetiva
                    // Considerando o espaçamento entre itens (10.0 pixels)
                    let spacing_total = (items_per_row - 1) as f32 * 10.0;
                    let thumbnail_width = (effective_width - spacing_total) / items_per_row as f32;
                    
                    // Manter a proporção da altura
                    let aspect_ratio = base_thumbnail_height / base_thumbnail_width;
                    let thumbnail_height = thumbnail_width * aspect_ratio;

                    let mut i = 0;
                    while i < self.filtered_videos.len() {
                        ui.horizontal(|ui| {
                            // Adicionar margem à esquerda para centralizar
                            ui.add_space(side_margin);
                            
                            for j in 0..items_per_row {
                                let idx = i + j;
                                if idx >= self.filtered_videos.len() {
                                    break;
                                }

                                let video_idx = self.filtered_videos[idx];
                                // Obter apenas as informações necessárias do vídeo antes do closure
                                let title = self.videos[video_idx].title.clone();
                                let texture_option = self.videos[video_idx].texture.clone();
                                let is_selected = self.selected_videos.contains(&video_idx);

                                ui.vertical(|ui| {
                                    // Exibir thumbnail
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(thumbnail_width, thumbnail_height),
                                        Sense::click(),
                                    );

                                    // Desenhar borda de seleção se o vídeo estiver selecionado
                                    if is_selected {
                                        ui.painter().rect_stroke(
                                            rect.expand(3.0),
                                            0.0,
                                            egui::Stroke::new(3.0, Color32::from_rgb(0, 120, 215)),
                                            egui::StrokeKind::Outside,
                                        );
                                    }

                                    if let Some(texture) = &texture_option {
                                        ui.painter().image(
                                            texture.id(),
                                            rect,
                                            egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0),
                                                egui::pos2(1.0, 1.0),
                                            ),
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
                                    let response = ui.interact(rect, ui.id().with(idx), Sense::click());
                                    
                                    // Verificar se Ctrl está pressionado
                                    let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);
                                    
                                    if response.clicked() {
                                        if ctrl_pressed {
                                            // Se Ctrl estiver pressionado, alterne a seleção
                                            self.toggle_video_selection(idx);
                                        } else {
                                            // Caso contrário, limpe a seleção e reproduza apenas este vídeo
                                            if !is_selected {
                                                self.selected_videos.clear();
                                            }
                                            self.play_video(idx);
                                        }
                                    }

                                    // Título do vídeo com quebra de linha
                                    ui.set_max_width(thumbnail_width);
                                    ui.label(&title);
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
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Visualizador m3u8",
        options,
        Box::new(|cc| Ok(Box::new(M3UViewer::new(cc)))),
    )
}
