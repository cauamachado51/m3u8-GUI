# Descrição: cria um m3u8 com nomes e links dos vídeos de uma playlist, você pode abrir o arquivo/playlist em PotPlayer e MPC-HC do K-lite. (VLC não)
# Como usar:
# 1. instale yt-dlp (ele é app solto): https://github.com/yt-dlp/yt-dlp
# 2. instale Python: https://www.python.org/downloads
# 3. adicione ao path a pasta de yt-dlp.exe: no menu iniciar abra Editar as variáveis de ambiente do sistema, clique em váriaveis de ambiente..., em váriaveis do sistema clique em Path, adicione algo como E:\Program Files\apps soltos\
# 4. abra o terminal onde o .py esteja e cole python "criar m3u8 de playlist do YouTube.py".

import subprocess

# configuração
playlist_url = "https://youtube.com/playlist?list=PLNPlJgG3nx8IqxpMrCkkEJvAhZXVbFaH0&si=Dcp_ntl5Q-Cm9eaW"
nome_arquivo = "playlist.m3u8"

def gerar_m3u8(playlist_url, nome_arquivo):
    # Executa o comando e captura a saída
    comando = ["yt-dlp", "--flat-playlist", "--print", "url", "--get-title", playlist_url]
    resultado = subprocess.run(comando, capture_output=True, text=True, check=True)

    linhas = resultado.stdout.strip().split("\n")
    if len(linhas) % 2 != 0:
        print("A resposta do yt-dlp não é pares de linhas (URL, Título).")
        return
    
    # Cria o arquivo M3U8
    with open(nome_arquivo, "w", encoding="utf-8") as arquivo:
        arquivo.write("#EXTM3U\n")
        for i in range(0, len(linhas), 2):
            link = linhas[i]
            titulo = linhas[i + 1]
            arquivo.write(f"#EXTINF:-1, {titulo}\n{link}\n")
        
gerar_m3u8(playlist_url, nome_arquivo)
