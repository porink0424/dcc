## docker で Linux 開発環境を整える

`docker build -t [タグ名] [Dockerfileのパス]`

で Docker は Dockerfile から命令を読み込み、自動的にイメージ（Docker はこのイメージを使ってコンテナを動作させる）を構築できる。Dockerfile を記述しておけば、上記のコマンドを実行するだけで同じ環境をすぐに構築することができる。

Dockerfile の書式は基本的に以下：

`命令 引数`

具体的には

`FROM <image>`

ベースとなるイメージの指定

`RUN <command>`

で FROM で指定したイメージ上で、コマンドを実行

#### docker イメージの作成

`docker build -t dcc .`

Dockerfile を置いたディレクトリで実行する。タグ名は dcc にしてみた

`docker images`

で確認できる。

`docker run --rm dcc ls /`

コンテナはバックグラウンドで動作し続けるようにすることも可能ですが、我々の使い方ではインタラクティブな使い方しか必要ないので、--rm オプションを与えることで、コマンドが終了し次第コンテナも終了するようにしました。従って、上記のコマンドを入力するたびにコンテナが作成されて破棄されることになります。

#### コンテナを使ったビルド、コンテナに新たなアプリケーションを追加

book に従う

#### つまりどうするか

イメージを作成したら

`docker run --rm -v <source>:<dest> -w <dest> dcc <command>`

でコンテナでコマンドを実行できる（source が dest として読み込まれ、current dir が dest になった状態で）

インタラクティブに使いたかったら

`docker run --rm -v $PWD:/dcc -w /dcc -it dcc`

↑ 毎回これを打とう！！
