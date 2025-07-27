use std::process::Command;
use std::fs::{self, File};
use std::io::Write;
use powershell::{read_host, console_temporario};

/// pergunta ao usuario uma URL de playlist do YouTube e cria um m3u8 com nomes e links dos vídeos.
/// grava `last_playlist_url: <URL>` em config.json, ao dar enter sem digitar nada ele usa o ultimo. exemplo de m3u8:
/// ```m3u8
/// #EXTM3U
/// #EXTINF:-1, Rap: Eu Sou Um Deus (Madara, Kira..........) // Purificando Almas // TK RAPS
/// https://www.youtube.com/watch?v=9YOpBlxxAqs
/// #EXTINF:-1, Rap do Escanor ( Nanatsu no Taizai ) | WLO | {Prod.MK Beats}
/// https://www.youtube.com/watch?v=hnZkREIOkX4
/// ```
/// requer: yt-dlp no path https://github.com/yt-dlp/yt-dlp
pub fn gerar_m3u8() {
    let _console = console_temporario();
    println!("requer: yt-dlp no path https://github.com/yt-dlp/yt-dlp");
    let mut playlist_url = read_host("Digite a URL da playlist (enter para usar o ultimo): ");
    let nome_arquivo = "playlist.m3u8";

    if playlist_url.is_empty() {
        let content_configjson = fs::read_to_string("config.json").unwrap_or_default();
        let parse_content_configjson = jzon::parse(&content_configjson).unwrap_or_else(|_| jzon::object! {});
        playlist_url = parse_content_configjson["last_playlist_url"].as_str().unwrap_or("").to_string();
    } else {
        let save_config = jzon::object! {
            "last_playlist_url": playlist_url.clone()
        };
        let _ = fs::write("config.json", jzon::stringify(save_config));
    }

    if playlist_url.is_empty() {
        read_host("Nenhuma URL fornecida e nenhuma URL salva encontrada. aperte enter.");
        return;
    }

    gerar_m3u8_interno(&playlist_url, nome_arquivo);
}

/// cria um m3u8 com nomes e links dos vídeos de uma playlist do YouTube. exemplo:
/// ```m3u8
/// #EXTM3U
/// #EXTINF:-1, Rap: Eu Sou Um Deus (Madara, Kira..........) // Purificando Almas // TK RAPS
/// https://www.youtube.com/watch?v=9YOpBlxxAqs
/// #EXTINF:-1, Rap do Escanor ( Nanatsu no Taizai ) | WLO | {Prod.MK Beats}
/// https://www.youtube.com/watch?v=hnZkREIOkX4
/// ```
/// requer: yt-dlp no path https://github.com/yt-dlp/yt-dlp
pub fn gerar_m3u8_interno(playlist_url: &str, nome_arquivo: &str) {
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