```
 ________
< 疲労! >
 --------
        \   ^__^
         \  (><)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
```

# Hirou Compiler
I would have made a C LLVM front for learning, but it no longer looks like C at all. So, I have no choice but to try to make my own language.

# Requirements

## Mac, Linux
1. Install Rust
2. Install LLVM 16

Here is example installation of LLVM 16 on macOS
```
brew install llvm@16
echo export LLVM_SYS_160_PREFIX=/opt/homebrew/Cellar/llvm@16/16.0.6 >> ~/.bashrc
# If you use fish shell, use this
set -Ux LLVM_SYS_160_PREFIX /opt/homebrew/Cellar/llvm@16/16.0.6
```

## ロードマップ
- 基本的な言語機能の実装 <- 今ココ
    - リージョンベースのメモリ管理
- VSCodeでのシンタックスハイライト
- テストフレームワークの実装
- 標準ライブラリの実装

## TODOリスト（やる順）
- トレイト（アロケーターの実装のために必要（stack, heapで挙動が違うため))
- 名前空間（モジュールシステム）
- メモリ管理
- Rustで標準ライブラリ作れるようにする
- string型の実装
- 使用箇所からのジェネリクス引数の推論
- リテラル
    - ベクタリテラル []
    - マップリテラル {} (Structとの相互変換を実装したい)
    - セットリテラル #{}

以下は細かいの
- Statementいらなかったので削除
- 関数定義のアノテーションがなかったらvoid型
- indexがintであるかの検証
- structのフィールドにVoidを入れることは出来ないことの検証
- panic!, todo!, unreachable!の実装
- annotationをOption<&ResolvedType>にできるか検討
- リージョンって実は推論できるかも cf. https://github.com/melsman/mlkit

変数定義こうしようと思ってる
```
(:= a 1)
(:= v  : Vec<i32> [1, 2, 3]
    v2 : Vec<u32> [])
```

## 言語仕様メモ　（まだまだ考え中）
- structは常に値をコピーして良いものに使う。Structの参照を取ることは出来ない。必ずコピーとなる。
- recordは常にリージョンのアロケーターを使ってメモリを確保する。recordは常に参照となるが、それを型で明示的に示すことはない。
- リージョンは、アロケーターと、そのアロケーターが管理するメモリの範囲のことである。

リージョンのおかげで以下のようなコードが書ける
```
fn return_slice(): str {
    (:= s "hello world")
    return (s[0..5]) // "hello"
}

// alloc fn は値を返すことが出来ない
ac fn use_return_slice() {
    (println "%s" (:= s (return_slice)))
}

sac fn use_return_slice() {
    (println "%s" (:= s (return_slice)))
}
```

- （要検討）再帰関数は必ずループに変換される（複雑なパーサーなどでその最適化が行えるか検証する）
  - （再起処理のパスを洗い出してインライン化すればできるかも...？）
- alloc scopeでスタックメモリに確保するリージョンを作れる。
- allocで確保するリージョンは、静的解析によって、スタックオーバーフローが起こることを検知し、エラーを出すことができる。
```
alloc {
・・・
}
```
- heap alloc scopeでヒープメモリに確保するリージョンを作れる。
```
heap_alloc {
    ...
}
```
- スレッドごとに必ずルートアロケータが存在する。リージョンはネストすることができ、スレッドごとのスタック構造になる
- リージョンのアロケーターはスレッドごとに管理される。
- 他のスレッドに record を渡すときは必ずコピーとなる。
- クロージャーは常にリージョンを持つ。リージョンはクロージャーが作られたスコープのリージョンとなる。

- Interface
```
interface as_bool<T>(self: T): bool

impl as_bool<i32>(self): bool {

}
```
