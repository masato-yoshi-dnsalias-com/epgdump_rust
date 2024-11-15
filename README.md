# epgdump rust版
## Rust で記述された  epgdump UNA版互換の、MPEG2-TSストリームからEPGデータを取得するプログラムです。
UNA版本家：https://katauna.hatenablog.com/archive/category/epgdump%20UNA

## epgdump：録画コマンド
    epgdump --BS|--CS|<id> tsFile outfile [ ( [--pf] [--sid n] ) | [--cut n1,n2] ]
詳しいオプションは「epgdump --help」を参照してください。  

# ビルド
ビルドするには Rust が必要です。  
Rust がインストールされていない場合は、Rustup をインストールしてください。  
## Ubuntu / Debian
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
上記のコマンドでRustupをインストールできます。  

## コンパイルとインストール
    bash install.sh

## 手動コンパイル

    cargo build --release

## 手動インストール
    install target/release/epgdump /usr/local/bin
