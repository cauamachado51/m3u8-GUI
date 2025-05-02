# Motivo
Minha playlist no YouTube contém cerca de 900 músicas. Leva muito tempo para chegar ao final e, pior ainda, clicar em um vídeo e depois voltar faz com que seja necessário recarregar os vídeos.

# Solução
Meu aplicativo carrega toda a playlist de uma vez e nunca a descarrega.

# Uso
- Foi desenvolvido para ser usado com o melhor player do mundo, [PotPlayer](https://potplayer.daum.net/). Ele deve ser configurado como padrão para abrir arquivos .m3u/.m3u8.
- Gere o arquivo .m3u utilizando o "criar m3u8 de playlist do YouTube.py". leia o comentário dentro de dele para saber como usar.
[![Assista no YouTube](https://img.youtube.com/vi/DGp3KWYItNk/maxresdefault.jpg)](https://youtu.be/DGp3KWYItNk)

# Técnico
- O aplicativo baixa miniaturas para a pasta cache_m3u, caso necessário. Ao clicar para abrir um video, ele cria um temp.m3u para abrir no app padrão.
- Fiz usando IA até o v12, mas decidi parar um pouco para voltar a ler a documentação e aprender rust (quero programar sem IA), depois vou nas dependencias.
- Não faço ideia se o executavel do Linux funciona.