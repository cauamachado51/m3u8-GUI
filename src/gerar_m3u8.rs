// Descrição: cria um m3u8 com nomes e links dos vídeos de uma playlist, você pode abrir o arquivo/playlist em PotPlayer e MPC-HC do K-lite. (VLC não)
// Como usar:
// 1. instale yt-dlp (ele é app solto): https://github.com/yt-dlp/yt-dlp
// 2. instale Rust: https://rustup.rs/
// 3. adicione ao path a pasta de yt-dlp.exe: no menu iniciar abra Editar as variáveis de ambiente do sistema, clique em váriaveis de ambiente..., em váriaveis do sistema clique em Path, adicione algo como E:\Program Files\apps soltos\
// 4. compile e execute: cargo run

use std::process::Command;
use std::fs::File;
use std::io::Write;

pub fn gerar_m3u8() {
    let playlist_url = "https://youtube.com/playlist?list=PLNPlJgG3nx8IqxpMrCkkEJvAhZXVbFaH0&si=Dcp_ntl5Q-Cm9eaW";
    let nome_arquivo = "playlist.m3u8";
    
    let output = Command::new("yt-dlp")
        .args(&["--flat-playlist", "--print", "url", "--get-title", playlist_url])
        .output()
        .expect("Falha ao executar yt-dlp");
    
    let stdout = String::from_utf8(output.stdout).expect("Saída inválida");
    let linhas: Vec<&str> = stdout.trim().split('\n').collect();
    
    if linhas.len() % 2 != 0 {
        panic!("Formato de resposta do yt-dlp inválido");
    }
    
    let mut arquivo = File::create(nome_arquivo).expect("Não foi possível criar o arquivo");
    arquivo.write_all(b"#EXTM3U\n").unwrap();
    
    for i in (0..linhas.len()).step_by(2) {
        let link = linhas[i];
        let titulo = linhas[i + 1];
        writeln!(arquivo, "#EXTINF:-1, {}\n{}", titulo, link).unwrap();
    }
    
    println!("Playlist M3U8 gerada: {}", nome_arquivo);
}