### melhorar desempenho de inicialização
quando inicializa ele verifica a existência de <id do video>.jpg na pasta cache_m3u, se não existir ele baixa, mas eu uso sempre o mesmo .m3u, então 99,9% das vezes não tem novas thumbnails para baixar, eu queria que a verficação fosse iniciada pelo botão "atualizar thumbnails", que ficaria no botão "opções" na barra superior. não tenho certeza, mas não descarto a possibilidade de talvez não melhorar o desempenho de inicialização por ainda precisar linkar todas as imagens na pasta cache_m3u na GUI.

### modo lista
quero adicionar o modo lista. faça o botão/menu "visualização" ao lado de "opções", dentro coloque "mudar para <lista/grade>". o modo lista é ter uma coluna de video, a imagem a esquerda e o titulo a direita.

### adicionar suporte a configurações.
no botão "opções" adicionar o botão "configurações", que ao apertar cria uma janela para as configurações, a esquerda deve ter abas e a direita as configurações da aba aberta. as configurações são guardadas em config.json

### adicionar suporte a menu de contexto
com o Click Direito, seja com vários videos selecionados ou nenhum (Click direto no alvo), abre um menu de contexto.

### adicionar o recurso de tags e playlists
no menu de contexto ter o botão "editar", que sumona uma janela que contém os campos:
```plaintext
<nome do video>

playlist: <escrever>
tag: <escrever> valor: <escrever>

<cancelar> <salvar>
```
ele deve me permitir adicionar multiplas playlists e tags, não deve obrigar a dar o valor da tag, também deve ter um botãozinho tipo <, que ao clicar ele verifica quais já existem pra aquele campo, mostra tipo o menu de contexto e me deixa clicar para inserir no campo. caso o video já tenha dados, ao clicar em "editar" eles serão mostrados nos campos. ao salvar é modificado/adicionado em m3u.json, exemplo:
```json
{
  "playlist": {
    "energia": {
      "videos": ["id do video", "id do video", "id do video"]
    },
    "relax": {
      "videos": ["id do video", "id do video"]
    }
  },
  "tag": {
    "nota": {
      "id do video": "top",
      "id do video": "10"
    },
    "calmo": {
      "id do video": "suave",
      "id do video": "relax total"
    }
  }
}
```
deve ter na Barra Superior ao lado do botão "opções" um botão "visualização", que abre uma janela de configuração própria, lá você pode selecionar qual playlist filtrar para só mostrar os videos dela, quais tags aparecem abaixo do titulo do video (valor incluso), também deve ter seleção de como é ordenado os videos, por ordem de inserção (é a atual, ordem em que processa o .m3u), ou por ordem alfabetica (titulo do video ou valor de X tag). também deve ter seleção de exibição em grade ou lista, na lista as thumbs ficam a esquerda e o nome do video e as tags a direita um abaixo do outro.

### outros
recurso de seleção aleatoria.
verificar por que abre o terminal ao executar o programa.