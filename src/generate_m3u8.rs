// Descrição: cria um m3u8 com nomes e links dos vídeos de uma playlist, você pode abrir o arquivo/playlist em PotPlayer e MPC-HC do K-lite. (VLC não)
// Como usar:
// 1. instale yt-dlp (ele é app solto): https://github.com/yt-dlp/yt-dlp
// 2. instale Rust: https://rustup.rs/
// 3. adicione ao path a pasta de yt-dlp.exe: no menu iniciar abra Editar as variáveis de ambiente do sistema, clique em váriaveis de ambiente..., em váriaveis do sistema clique em Path, adicione algo como E:\Program Files\apps soltos\
// 4. compile e execute: cargo run

use std::process::Command;
use std::fs::File;
use std::io::Write;

pub fn generate_m3u8() {
    // configuração
    let playlist_url = "https://youtube.com/playlist?list=PLNPlJgG3nx8IqxpMrCkkEJvAhZXVbFaH0&si=Dcp_ntl5Q-Cm9eaW";
    let nome_arquivo = "playlist.m3u8";
    
    gerar_m3u8(playlist_url, nome_arquivo);
}

fn gerar_m3u8(playlist_url: &str, nome_arquivo: &str) {
    match executar_ytdlp(playlist_url, nome_arquivo) {
        Ok(_) => println!("Playlist M3U8 gerada com sucesso: {}", nome_arquivo),
        Err(e) => println!("Erro ao gerar a playlist: {}", e),
    }
}

fn executar_ytdlp(playlist_url: &str, nome_arquivo: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Comando yt-dlp para extrair os links dos vídeos e seus títulos
    let output = Command::new("yt-dlp")
        .args(&["--flat-playlist", "--print", "url", "--get-title", playlist_url])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("yt-dlp falhou: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    let stdout = String::from_utf8(output.stdout)?;
    let linhas: Vec<&str> = stdout.trim().split('\n').collect();
    
    if linhas.len() % 2 != 0 {
        return Err("A resposta do yt-dlp não está no formato esperado (URL, Título).".into());
    }
    
    // Cria o arquivo M3U8
    let mut arquivo = File::create(nome_arquivo)?;
    arquivo.write_all(b"#EXTM3U\n")?;
    
    for i in (0..linhas.len()).step_by(2) {
        let link = linhas[i];
        let titulo = linhas[i + 1];
        arquivo.write_all(format!("#EXTINF:-1, {}\n{}\n", titulo, link).as_bytes())?;
    }
    
    Ok(())
}