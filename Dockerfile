# 最新のubuntuのバージョンを指定
FROM ubuntu:latest

# apt updateはパッケージのリポジトリから、パッケージの名前やバージョン、依存関係を取得し、有効でインストール可能なパッケージの一覧を更新するコマンド
RUN apt update

# DEBIAN_FRONTEND=noninteractiveによりDockerfile内でapt installする際に、対話式の命令をスキップするように設定することができる
# ENV DEBIAN_FRONTEND=noninteractive のように用いると、そのdocker imageを利用したdocker containerやdocker imageの環境変数も影響を受けてしまうので、一時的になるように以下のような書き方になっている
RUN DEBIAN_FRONTEND=noninteractive apt install -y gcc make git binutils libc6-dev gdb sudo

# adduser: 新規userの作成
# --disabled-password: パスワードを使用できなくする
# --gecos 設定内容:	GECOSフィールド（/etc/passwdに保存されるユーザーの所属やフルネームなど）の内容を設定する
# 対話式でメールアカウントとか聞かれないように-gecos ''指定
# userはユーザネーム
RUN adduser --disabled-password --gecos '' user

# /etc/sudoers: UNIX系のOS（Linuxとか）で使われる、sudoコマンドで変更できるユーザと実行できるコマンドを記述する設定ファイル
# /etc/sudoers.dディレクトリ以下のファイルはsudoersファイルの #includedir /etc/sudoers.d によってsudoersから読み込まれ、sudoersとして有効になる。sudoersファイルを編集するよりも/etc/sudoers.d/以下にカスタマイズ用ファイルを置く方が、カスタマイズ部分が明確にわかるという点でより良い。
# sudoersの基本構文:   誰が どのホストで = (誰として) 何を
RUN echo 'user ALL=(root) NOPASSWD:ALL' > /etc/sudoers.d/user

# 各種命令の実行時のユーザの指定
USER user

# 各種命令の実行時のcurrent directoryを指定
WORKDIR /home/user