# Descrição: cria um m3u8 com nomes e links dos vídeos de uma playlist, você pode abrir o arquivo/playlist em PotPlayer e MPC-HC do K-lite. (VLC não)
# Como usar:
# 1. instale yt-dlp (ele é app solto): https://github.com/yt-dlp/yt-dlp
# 2. adicione ao path a pasta de yt-dlp.exe: no menu iniciar abra Editar as variáveis de ambiente do sistema, clique em váriaveis de ambiente..., em váriaveis do sistema clique em Path, adicione algo como E:\Program Files\apps soltos\
# 3. abra o PowerShell onde o .ps1 esteja e execute: .\nome_do_arquivo.ps1

# configuração
$playlist_url = "https://youtube.com/playlist?list=PLNPlJgG3nx8IqxpMrCkkEJvAhZXVbFaH0&si=Dcp_ntl5Q-Cm9eaW"
$nome_arquivo = "playlist.m3u8"

function gerar_m3u8($playlist_url, $nome_arquivo) {
    if (-not (Get-Command "yt-dlp" -ErrorAction SilentlyContinue)) { throw "yt-dlp não foi encontrado no path." }

    # Executa o comando e captura a saída
    $resultado = & "yt-dlp" @("--flat-playlist", "--print", "url", "--get-title", $playlist_url)
    if ($resultado.Count % 2 -ne 0) { throw "A resposta do yt-dlp não é pares de linhas (URL, Título)." }
    
    # Cria o arquivo M3U8
    $conteudo = "#EXTM3U`n"
    for ($i = 0; $i -lt $resultado.Count; $i += 2) {
        $link = $resultado[$i]
        $titulo = $resultado[$i + 1]
        $conteudo += "#EXTINF:-1, $titulo`n$link`n"
    }
    Set-Content -Path $nome_arquivo -Value $conteudo -Encoding UTF8
}

gerar_m3u8 $playlist_url $nome_arquivo